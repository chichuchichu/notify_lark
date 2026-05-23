use anyhow::{Context, Result};

pub struct Config {
    pub webhook_url: String,
}

const HELP: &str = r#"使用方法:
  1. 飞书群设置 → 群机器人 → 添加自定义机器人，获取 webhook URL
  2. 设置环境变量:
     export LARK_WEBHOOK_URL="https://open.feishu.cn/open-apis/bot/v2/hook/xxxxx"
  3. 或创建 .env 文件:
     echo 'LARK_WEBHOOK_URL=https://open.feishu.cn/open-apis/bot/v2/hook/xxxxx' > .env
  4. 完成后验证:
     notify_lark "配置测试"
  5. (推荐) opencode 用户集成自动通知:
     notify_lark setup
     然后重启 opencode，所有 agent 任务完成/中断/权限请求都会自动发飞书通知"#;

impl Config {
    pub fn from_env() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let webhook_url = std::env::var("LARK_WEBHOOK_URL")
            .context(format!("LARK_WEBHOOK_URL 环境变量未设置\n{}", HELP))?;

        if webhook_url.is_empty() {
            anyhow::bail!("LARK_WEBHOOK_URL 不能为空");
        }

        Ok(Config { webhook_url })
    }
}
