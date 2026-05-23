use std::io::Read;

use anyhow::{Context, Result};
use clap::Parser;

mod config;
mod lark;

#[derive(Parser)]
#[command(name = "notify_lark", about = "通过飞书机器人发送通知消息")]
struct Cli {
    /// 消息内容（可选，不提供时从 stdin 读取）
    message: Option<String>,

    /// 消息类型: text 或 interactive
    #[arg(short = 't', long = "type", default_value = "text")]
    msg_type: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

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

    let config = config::Config::from_env()?;
    let client = lark::LarkClient::new(&config)?;

    match cli.msg_type.as_str() {
        "text" => client.send_text(&message).await?,
        "interactive" => client.send_interactive(&message).await?,
        other => anyhow::bail!("不支持的消息类型: {}，可选值: text, interactive", other),
    }

    Ok(())
}
