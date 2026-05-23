use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod config;
mod lark;

const PLUGIN_TS: &str = r#"import type { Plugin } from "@opencode-ai/plugin"
import { spawnSync } from "node:child_process"
import { platform } from "node:os"

const BIN = platform() === "win32" ? "notify_lark.exe" : "notify_lark"
const ENV = process.env.LARK_WEBHOOK_URL ? "ok" : "missing"

function sendCard(title: string, body: string) {
  const result = spawnSync(BIN, ["--card-title", title, body.slice(0, 2000)], { timeout: 5000 })
  if (result.error) {
    console.error(`[notify-lark] ${BIN} not found (PATH?); LARK_WEBHOOK_URL=${ENV}`)
  } else if (result.status !== 0) {
    const stderr = result.stderr?.toString().trim() || ""
    console.error(`[notify-lark] exit ${result.status}${stderr ? ": " + stderr : ""}`)
  }
}

function extractText(content: any): string {
  if (typeof content === "string") return content
  if (Array.isArray(content)) return content.map((c: any) => c.text ?? c.content ?? "").join(" ")
  return String(content ?? "")
}

function preview(text: string, max: number): string {
  const s = text.replace(/\s+/g, " ").trim()
  return s.length > max ? s.slice(0, max) + "..." : s
}

export default (async () => {
  return {
    "permission.ask": async (input: any) => {
      const detail = input?.detail ?? input?.description ?? "agent 请求权限，请查看 opencode 确认"
      sendCard("需要授权", preview(String(detail), 200))
    },
    "chat.message": async (input: any) => {
      const text = extractText(input?.content ?? input?.message?.content)
      if (!text || text.length < 3) return
      sendCard("任务完成", preview(text, 500))
    },
  }
}) satisfies Plugin
"#;

#[derive(Parser)]
#[command(name = "notify_lark", about = "通过飞书机器人发送通知消息")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// 消息内容（可选，不提供时从 stdin 读取）
    message: Option<String>,

    /// 消息类型: text 或 interactive
    #[arg(short = 't', long = "type", default_value = "text")]
    msg_type: String,

    /// 发送消息卡片（需要 --card-title）
    #[arg(long = "card-title")]
    card_title: Option<String>,

    /// 卡片按钮跳转 URL（可选）
    #[arg(long = "card-url")]
    card_url: Option<String>,

    /// 卡片按钮文字（默认: 查看详情）
    #[arg(long = "card-button")]
    card_button: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// 安装 opencode 集成（自动创建 plugin 文件和配置）
    Setup,
    /// 测试配置是否正常（发送一条测试消息到飞书）
    Verify,
    /// 卸载 notify_lark 及 opencode 集成（plugin、skill、rules）
    Uninstall,
}

fn opencode_config_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default()).join(".config").join("opencode")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config").join("opencode")
    }
}

async fn send_message(cli: &Cli, message: String) -> Result<()> {
    let config = config::Config::from_env()?;
    let client = lark::LarkClient::new(&config)?;

    if let Some(title) = &cli.card_title {
        client.send_card(title, &message, cli.card_url.as_deref(), cli.card_button.as_deref()).await
    } else {
        match cli.msg_type.as_str() {
            "text" => client.send_text(&message).await,
            "post" => client.send_post_json(&message).await,
            "interactive" => client.send_interactive_json(&message).await,
            other => anyhow::bail!("不支持的消息类型: {}，可选值: text, post, interactive", other),
        }
    }
}

async fn run_setup() -> Result<()> {
    let cfg_dir = opencode_config_dir();
    std::fs::create_dir_all(cfg_dir.join("plugin")).context("创建 opencode plugin 目录失败")?;

    let plugin_path = cfg_dir.join("plugin").join("notify-lark.ts");
    std::fs::write(&plugin_path, PLUGIN_TS).context("写入 plugin 文件失败")?;
    println!("已创建 {}", plugin_path.display());

    let cfg_path = cfg_dir.join("opencode.jsonc");
    let new_entry = r#""plugin": ["./plugin/notify-lark.ts"]"#;

    if cfg_path.exists() {
        let content = std::fs::read_to_string(&cfg_path).context("读取 opencode.jsonc 失败")?;
        if !content.contains("notify-lark") {
            let body = content.trim().trim_end_matches('}').trim_end().trim_end_matches(',');
            let updated = format!("{},\n  {}\n}}\n", body, new_entry);
            std::fs::write(&cfg_path, updated).context("更新 opencode.jsonc 失败")?;
            println!("已更新 {}", cfg_path.display());
        } else {
            println!("opencode.jsonc 已包含 notify-lark 配置，跳过");
        }
    } else {
        let cfg = format!(
            r#"{{"$schema":"https://opencode.ai/config.json",
  {} }}
"#,
            new_entry
        );
        std::fs::write(&cfg_path, cfg).context("创建 opencode.jsonc 失败")?;
        println!("已创建 {}", cfg_path.display());
    }

    println!("\n配置完成。重启 opencode 后生效。");
    println!("验证安装: notify_lark verify");
    println!("卸载:      notify_lark uninstall && cargo uninstall notify_lark");
    Ok(())
}

async fn run_verify() -> Result<()> {
    println!("🔍 检查配置...");
    let config = config::Config::from_env()?;
    println!("  LARK_WEBHOOK_URL ✓");

    println!("📤 发送测试消息...");
    let client = lark::LarkClient::new(&config)?;
    client.send_text("notify_lark 安装配置验证通过 ✓").await?;
    println!("  ✓ 消息已发送，请检查飞书群聊");

    println!("\n所有验证通过。");
    Ok(())
}

fn remove_if_exists(path: &std::path::Path) {
    if path.is_dir() {
        std::fs::remove_dir_all(path).ok();
    } else if path.exists() {
        std::fs::remove_file(path).ok();
    }
}

fn clean_jsonc_entry(cfg_path: &std::path::Path, key: &str, value_pat: &str) {
    if !cfg_path.exists() { return; }
    let content = std::fs::read_to_string(cfg_path).unwrap_or_default();
    let mut lines: Vec<&str> = content.lines().collect();
    // remove the exact line containing value_pat
    lines.retain(|l| !l.contains(value_pat));
    // if the array is now empty (just `[...]` or `"key": [...]`), remove surrounding lines
    let mut cleaned = lines.join("\n");
    // remove empty plugin / instructions arrays
    let patterns = [
        (format!(r#""{}": []"#, key), ""),
        (format!(r#""{}": []"#, key), ""),
        (format!(r#""{}": [  ]"#, key), ""),
    ];
    for (old, new) in &patterns {
        cleaned = cleaned.replace(old.as_str(), new);
    }
    // remove trailing comma before the removed line
    cleaned = cleaned.replace(",\n  }", "\n}");
    cleaned = cleaned.replace(",\n}", "\n}");
    if cleaned != content {
        std::fs::write(cfg_path, cleaned).ok();
    }
}

async fn run_uninstall() -> Result<()> {
    let cfg_dir = opencode_config_dir();

    // 1. remove plugin
    let plugin = cfg_dir.join("plugin").join("notify-lark.ts");
    remove_if_exists(&plugin);
    println!("移除 plugin: {}", plugin.display());

    // 2. remove skill
    let skill = cfg_dir.join("skills").join("notify-lark");
    remove_if_exists(&skill);
    println!("移除 skill: {}", skill.display());

    // 3. remove instructions
    let instructions = cfg_dir.join("notify_lark.md");
    remove_if_exists(&instructions);
    println!("移除 rules: {}", instructions.display());

    // 4. clean opencode.jsonc entries
    let cfg_path = cfg_dir.join("opencode.jsonc");
    clean_jsonc_entry(&cfg_path, "plugin", "./plugin/notify-lark.ts");
    clean_jsonc_entry(&cfg_path, "instructions", "notify_lark.md");
    if cfg_path.exists() {
        println!("已清理 notify_lark 相关配置: {}", cfg_path.display());
    }

    // 5. remove empty plugin dir
    let plugin_dir = cfg_dir.join("plugin");
    if plugin_dir.exists() && plugin_dir.read_dir().map(|mut d| d.next().is_none()).unwrap_or(false) {
        std::fs::remove_dir(&plugin_dir).ok();
    }

    println!("\nnotify_lark 集成已从 opencode 移除。");
    println!("如需完全卸载，请执行:");
    println!("  cargo uninstall notify_lark");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Setup) => return run_setup().await,
        Some(Commands::Verify) => return run_verify().await,
        Some(Commands::Uninstall) => return run_uninstall().await,
        None => {}
    }

    let message = if let Some(msg) = &cli.message {
        msg.clone()
    } else {
        let mut input = String::new();
        std::io::stdin()
            .read_to_string(&mut input)
            .context("从 stdin 读取消息失败")?;
        input.trim().to_string()
    };

    if message.is_empty() {
        anyhow::bail!("消息内容不能为空");
    }

    send_message(&cli, message).await
}
