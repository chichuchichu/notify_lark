use anyhow::{Context, Result};
use serde_json::{json, Value};

use crate::config::Config;

const LARK_WEBHOOK_PATH: &str = "/open-apis/bot/v2/hook/";

fn resolve(text: &str) -> String {
    text.replace("\\n", "\n")
}

pub struct LarkClient {
    client: reqwest::Client,
    webhook_url: String,
}

impl LarkClient {
    pub fn new(config: &Config) -> Result<Self> {
        let webhook_url = config.webhook_url.trim_end_matches('/').to_string();

        if !webhook_url.contains(LARK_WEBHOOK_PATH) {
            anyhow::bail!(
                "无效的飞书 webhook URL，路径必须包含 {}，当前: {}",
                LARK_WEBHOOK_PATH,
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
            "content": { "text": resolve(text) }
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

    #[allow(dead_code)]
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

    pub async fn send_card(&self, title: &str, body_text: &str, button_url: Option<&str>, button_text: Option<&str>) -> Result<()> {
        let body = resolve(body_text);
        let mut elements: Vec<Value> = vec![json!({
            "tag": "markdown",
            "content": body,
            "text_align": "left",
            "text_size": "normal_v2"
        })];

        if let Some(url) = button_url {
            let btn_label = button_text.unwrap_or("查看详情");
            elements.push(json!({
                "tag": "button",
                "text": {
                    "tag": "plain_text",
                    "content": btn_label
                },
                "type": "default",
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use wiremock::matchers::{method, path};

    fn make_url(base: &str, token: &str) -> String {
        format!("{}/open-apis/bot/v2/hook/{}", base.trim_end_matches('/'), token)
    }

    // --- URL validation ---

    #[test]
    fn test_valid_production_url() {
        let cfg = Config { webhook_url: "https://open.feishu.cn/open-apis/bot/v2/hook/abc123".into() };
        assert!(LarkClient::new(&cfg).is_ok());
    }

    #[test]
    fn test_valid_mock_url() {
        let cfg = Config { webhook_url: "http://127.0.0.1:9999/open-apis/bot/v2/hook/test".into() };
        assert!(LarkClient::new(&cfg).is_ok());
    }

    #[test]
    fn test_invalid_url_wrong_host() {
        let cfg = Config { webhook_url: "https://evil.com/hook/abc".into() };
        assert!(LarkClient::new(&cfg).is_err());
    }

    #[test]
    fn test_invalid_url_no_path() {
        let cfg = Config { webhook_url: "https://example.com/not-lark".into() };
        assert!(LarkClient::new(&cfg).is_err());
    }

    // --- resolve() ---

    #[test]
    fn test_resolve_plain() {
        assert_eq!(resolve("hello"), "hello");
    }

    #[test]
    fn test_resolve_newline() {
        assert_eq!(resolve("a\\nb"), "a\nb");
    }

    #[test]
    fn test_resolve_multiple() {
        assert_eq!(resolve("x\\ny\\nz"), "x\ny\nz");
    }

    #[test]
    fn test_resolve_empty() {
        assert_eq!(resolve(""), "");
    }

    #[test]
    fn test_resolve_actual_newline() {
        assert_eq!(resolve("a\nb"), "a\nb");
    }

    #[test]
    fn test_resolve_mixed() {
        assert_eq!(resolve("a\\n\nb"), "a\n\nb");
    }

    // --- JSON body structure ---

    #[test]
    fn test_text_body_structure() {
        let body = json!({
            "msg_type": "text",
            "content": { "text": resolve("hello") }
        });
        assert_eq!(body["msg_type"], "text");
        assert_eq!(body["content"]["text"], "hello");
    }

    #[test]
    fn test_text_body_newline_resolved() {
        let body = json!({
            "msg_type": "text",
            "content": { "text": resolve("a\\nb") }
        });
        assert_eq!(body["content"]["text"], "a\nb");
    }

    #[test]
    fn test_card_body_structure() {
        let title = "测试";
        let body_text = "内容";
        let body = resolve(body_text);
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
                "elements": [{
                    "tag": "markdown",
                    "content": body,
                    "text_align": "left",
                    "text_size": "normal_v2"
                }]
            }
        });
        assert_eq!(card["header"]["title"]["content"], "测试");
        assert_eq!(card["body"]["elements"][0]["content"], "内容");
    }

    #[test]
    fn test_card_with_button() {
        let title = "测试";
        let body_text = "内容";
        let body = resolve(body_text);
        let btn_url = "https://example.com";
        let btn_label = "查看";
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
                "elements": [
                    json!({
                        "tag": "markdown",
                        "content": body,
                        "text_align": "left",
                        "text_size": "normal_v2"
                    }),
                    json!({
                        "tag": "button",
                        "text": { "tag": "plain_text", "content": btn_label },
                        "type": "default",
                        "width": "default",
                        "size": "medium",
                        "behaviors": [{ "type": "open_url", "default_url": btn_url }],
                        "margin": "8px 0px 0px 0px"
                    })
                ]
            }
        });
        assert_eq!(card["body"]["elements"].as_array().unwrap().len(), 2);
        assert_eq!(card["body"]["elements"][1]["behaviors"][0]["default_url"], "https://example.com");
    }

    #[test]
    fn test_post_body_structure() {
        let lang = "zh_cn";
        let title = "通知";
        let content = json!([[{"tag": "text", "text": "hi"}]]);
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
        assert_eq!(body["msg_type"], "post");
        assert_eq!(body["content"]["post"][lang]["title"], "通知");
        assert_eq!(body["content"]["post"][lang]["content"][0][0]["tag"], "text");
    }

    // --- HTTP mock tests ---

    #[tokio::test]
    async fn test_send_text_success() {
        let mock_server = MockServer::start().await;
        let url = make_url(&mock_server.uri(), "test-token");

        Mock::given(method("POST"))
            .and(path("/open-apis/bot/v2/hook/test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0, "msg": "success"
            })))
            .mount(&mock_server)
            .await;

        let cfg = Config { webhook_url: url };
        let client = LarkClient::new(&cfg).unwrap();
        client.send_text("hello").await.unwrap();
    }

    #[tokio::test]
    async fn test_send_text_rejects_business_error() {
        let mock_server = MockServer::start().await;
        let url = make_url(&mock_server.uri(), "test");

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 19024, "msg": "Key Words Not Found"
            })))
            .mount(&mock_server)
            .await;

        let cfg = Config { webhook_url: url };
        let client = LarkClient::new(&cfg).unwrap();
        let err = client.send_text("no keyword").await.unwrap_err();
        assert!(err.to_string().contains("19024"));
    }

    #[tokio::test]
    async fn test_send_text_rejects_http_error() {
        let mock_server = MockServer::start().await;
        let url = make_url(&mock_server.uri(), "test");

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(400).set_body_string("bad request"))
            .mount(&mock_server)
            .await;

        let cfg = Config { webhook_url: url };
        let client = LarkClient::new(&cfg).unwrap();
        let err = client.send_text("test").await.unwrap_err();
        assert!(err.to_string().contains("400"));
    }

    #[tokio::test]
    async fn test_send_card_triggers_post() {
        let mock_server = MockServer::start().await;
        let url = make_url(&mock_server.uri(), "test");

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0, "msg": "success"
            })))
            .mount(&mock_server)
            .await;

        let cfg = Config { webhook_url: url };
        let client = LarkClient::new(&cfg).unwrap();
        client.send_card("Title", "Body", None, None).await.unwrap();
    }

    #[tokio::test]
    async fn test_send_card_with_button() {
        let mock_server = MockServer::start().await;
        let url = make_url(&mock_server.uri(), "test");

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0, "msg": "success"
            })))
            .mount(&mock_server)
            .await;

        let cfg = Config { webhook_url: url };
        let client = LarkClient::new(&cfg).unwrap();
        client.send_card("Title", "Body", Some("https://example.com"), Some("详情")).await.unwrap();
    }

    #[tokio::test]
    async fn test_actual_http_body() {
        let mock_server = MockServer::start().await;
        let url = make_url(&mock_server.uri(), "assert-body");

        let expected_body = json!({
            "msg_type": "text",
            "content": { "text": "hello\nworld" }
        });

        Mock::given(method("POST"))
            .and(wiremock::matchers::body_json(expected_body))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0, "msg": "success"
            })))
            .mount(&mock_server)
            .await;

        let cfg = Config { webhook_url: url };
        let client = LarkClient::new(&cfg).unwrap();
        client.send_text("hello\\nworld").await.unwrap();
    }

    // --- send_post_json ---

    #[tokio::test]
    async fn test_send_post_json_success() {
        let mock_server = MockServer::start().await;
        let url = make_url(&mock_server.uri(), "test");

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "code": 0, "msg": "success"
            })))
            .mount(&mock_server)
            .await;

        let cfg = Config { webhook_url: url };
        let client = LarkClient::new(&cfg).unwrap();
        let post = r#"{"zh_cn":{"title":"测试","content":[[{"tag":"text","text":"hi"}]]}}"#;
        client.send_post_json(post).await.unwrap();
    }

    #[tokio::test]
    async fn test_send_post_json_invalid() {
        let mock_server = MockServer::start().await;
        let url = make_url(&mock_server.uri(), "test");

        let cfg = Config { webhook_url: url };
        let client = LarkClient::new(&cfg).unwrap();
        let err = client.send_post_json("not json").await.unwrap_err();
        assert!(err.to_string().contains("解析富文本 JSON 失败"));
    }

    // --- send_interactive_json ---

    #[tokio::test]
    async fn test_send_interactive_json_invalid() {
        let mock_server = MockServer::start().await;
        let url = make_url(&mock_server.uri(), "test");

        let cfg = Config { webhook_url: url };
        let client = LarkClient::new(&cfg).unwrap();
        let err = client.send_interactive_json("{bad json}").await.unwrap_err();
        assert!(err.to_string().contains("解析交互卡片 JSON 失败"));
    }
}
