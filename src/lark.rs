use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::config::Config;

const LARK_WEBHOOK_BASE: &str = "https://open.feishu.cn/open-apis/bot/v2/hook";

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
        let body = json!({
            "msg_type": "text",
            "content": { "text": text }
        });
        self.post(&body).await
    }

    pub async fn send_interactive(&self, card: Value) -> Result<()> {
        let body = json!({
            "msg_type": "interactive",
            "card": card
        });
        self.post(&body).await
    }

    pub async fn send_interactive_json(&self, card_json: &str) -> Result<()> {
        let card: Value = serde_json::from_str(card_json)
            .context("解析交互卡片 JSON 失败")?;
        self.send_interactive(card).await
    }

    pub async fn send_post(&self, lang: &str, title: &str, content: Value) -> Result<()> {
        let body = json!({
            "msg_type": "post",
            "content": {
                "post": {
                    lang: {
                        "title": title,
                        "content": content
                    }
                }
            }
        });
        self.post(&body).await
    }

    pub async fn send_post_json(&self, post_json: &str) -> Result<()> {
        let post_val: Value = serde_json::from_str(post_json)
            .context("解析富文本 JSON 失败")?;
        let body = json!({
            "msg_type": "post",
            "content": {
                "post": post_val
            }
        });
        self.post(&body).await
    }

    pub async fn send_card(&self, title: &str, body_text: &str, button_url: Option<&str>) -> Result<()> {
        let mut elements: Vec<Value> = vec![json!({
            "tag": "markdown",
            "content": body_text,
            "text_align": "left",
            "text_size": "normal_v2"
        })];

        if let Some(url) = button_url {
            elements.push(json!({
                "tag": "button",
                "text": {
                    "tag": "plain_text",
                    "content": "打开 opencode"
                },
                "type": "primary",
                "width": "default",
                "size": "medium",
                "behaviors": [{
                    "type": "open_url",
                    "default_url": url
                }],
                "margin": "8px 0px 0px 0px"
            }));
        }

        let card = json!({
            "schema": "2.0",
            "config": { "update_multi": true },
            "header": {
                "title": { "tag": "plain_text", "content": title },
                "template": "blue",
                "padding": "12px 12px 12px 12px"
            },
            "body": {
                "direction": "vertical",
                "padding": "12px 12px 12px 12px",
                "elements": elements
            }
        });

        self.send_interactive(card).await
    }

    async fn post(&self, body: &Value) -> Result<()> {
        let resp = self
            .client
            .post(&self.webhook_url)
            .json(body)
            .send()
            .await
            .context("发送飞书消息请求失败")?;

        let status = resp.status();
        let resp_body = resp.text().await.context("读取飞书响应失败")?;

        if !status.is_success() {
            anyhow::bail!("飞书 API 返回错误 ({}): {}", status.as_u16(), resp_body);
        }

        let parsed: Value =
            serde_json::from_str(&resp_body).context("解析飞书响应 JSON 失败")?;

        let code = parsed["code"].as_i64().unwrap_or(-1);
        if code != 0 {
            let msg = parsed["msg"].as_str().unwrap_or("未知错误");
            anyhow::bail!("飞书 API 返回错误码 {}: {}", code, msg);
        }

        Ok(())
    }
}
