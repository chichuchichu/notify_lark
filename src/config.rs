use anyhow::{Context, Result};

pub struct Config {
    pub webhook_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let webhook_url = std::env::var("LARK_WEBHOOK_URL")
            .context("LARK_WEBHOOK_URL 环境变量未设置，请配置飞书机器人的 webhook 地址")?;

        if webhook_url.is_empty() {
            anyhow::bail!("LARK_WEBHOOK_URL 不能为空");
        }

        Ok(Config { webhook_url })
    }
}
