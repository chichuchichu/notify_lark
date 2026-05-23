use anyhow::{Context, Result};
use serde::Serialize;

use crate::config::Config;

const LARK_WEBHOOK_BASE: &str = "https://open.feishu.cn/open-apis/bot/v2/hook";

#[derive(Serialize)]
struct TextContent {
    text: String,
}

#[derive(Serialize)]
struct LarkMessage {
    msg_type: String,
    content: TextContent,
}

pub struct LarkClient {
    client: reqwest::Client,
    webhook_url: String,
}

impl LarkClient {
    pub fn new(config: &Config) -> Result<Self> {
        let webhook_url = config.webhook_url.trim_end_matches('/').to_string();

        if !webhook_url.starts_with(LARK_WEBHOOK_BASE) {
            anyhow::bail!(
                "无效的飞书 webhook URL，期望以 {} 开头，实际: {}",
                LARK_WEBHOOK_BASE,
                webhook_url
            );
        }

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("创建 HTTP 客户端失败")?;

        Ok(LarkClient {
            client,
            webhook_url,
        })
    }

    pub async fn send_text(&self, text: &str) -> Result<()> {
        let msg = LarkMessage {
            msg_type: "text".to_string(),
            content: TextContent {
                text: text.to_string(),
            },
        };

        self.post(&msg).await
    }

    pub async fn send_interactive(&self, card_json: &str) -> Result<()> {
        let msg = LarkMessage {
            msg_type: "interactive".to_string(),
            content: TextContent {
                text: card_json.to_string(),
            },
        };

        self.post(&msg).await
    }

    async fn post(&self, msg: &LarkMessage) -> Result<()> {
        let resp = self
            .client
            .post(&self.webhook_url)
            .json(msg)
            .send()
            .await
            .context("发送飞书消息请求失败")?;

        let status = resp.status();
        let body = resp.text().await.context("读取飞书响应失败")?;

        if !status.is_success() {
            anyhow::bail!("飞书 API 返回错误 ({}): {}", status.as_u16(), body);
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&body).context("解析飞书响应 JSON 失败")?;

        let code = parsed["code"].as_i64().unwrap_or(-1);
        if code != 0 {
            let msg = parsed["msg"].as_str().unwrap_or("未知错误");
            anyhow::bail!("飞书 API 返回错误码 {}: {}", code, msg);
        }

        Ok(())
    }
}
