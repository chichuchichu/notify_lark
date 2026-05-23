use std::io::Read;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod config;
mod lark;

const PLUGIN_TS: &str = r#"import type { Plugin } from "@opencode-ai/plugin"
import { execSync } from "node:child_process"
import { platform } from "node:os"

const BIN = platform() === "win32" ? "notify_lark.exe" : "notify_lark"

function notify(msg: string) {
  try {
    execSync(`${BIN} "${msg.replace(/"/g, '\\"')}"`, {
      stdio: "ignore",
      timeout: 5000,
    })
  } catch {
  }
}

export default (async () => {
  return {
    "permission.ask": async () => {
      notify("需要确认: agent 请求权限，请查看 opencode 操作")
    },
    "chat.message": async (input) => {
      if (input?.role !== "assistant") return
      const text = (input?.content ?? "").trim()
      if (text.length < 5) return
      const preview = text.length > 150 ? text.slice(0, 150) + "..." : text
      notify(`任务完成: ${preview}`)
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
}

#[derive(Subcommand)]
enum Commands {
    /// 安装 opencode 集成（自动创建 plugin 文件和配置）
    Setup,
}

fn opencode_config_dir() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default()).join(".config").join("opencode")
    } else {
        PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".config").join("opencode")
    }
}

async fn send_message(message: String, msg_type: String) -> Result<()> {
    let config = config::Config::from_env()?;
    let client = lark::LarkClient::new(&config)?;

    match msg_type.as_str() {
        "text" => client.send_text(&message).await,
        "interactive" => client.send_interactive(&message).await,
        other => anyhow::bail!("不支持的消息类型: {}，可选值: text, interactive", other),
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
            let updated = if content.trim().ends_with('}') {
                let trimmed = content.trim_end();
                let inner = &trimmed[..trimmed.len() - 1];
                format!("{},\n  {}\n}}", inner, new_entry)
            } else {
                content
            };
            std::fs::write(&cfg_path, updated).context("更新 opencode.jsonc 失败")?;
            println!("已更新 {}", cfg_path.display());
        } else {
            println!("opencode.jsonc 已包含 notify-lark 配置，跳过");
        }
    } else {
        let cfg = format!(
            r#"{{
  "$schema": "https://opencode.ai/config.json",
  {}
}}
"#,
            new_entry
        );
        std::fs::write(&cfg_path, cfg).context("创建 opencode.jsonc 失败")?;
        println!("已创建 {}", cfg_path.display());
    }

    println!("\n配置完成。重启 opencode 后生效。");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Some(Commands::Setup) = cli.command {
        return run_setup().await;
    }

    let message = if let Some(msg) = cli.message {
        msg
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

    send_message(message, cli.msg_type).await
}
