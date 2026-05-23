# notify_lark

跨平台飞书通知工具，agent 或 CI 任务中任何需要通知用户的场景，通过飞书机器人发送消息。

## 安装

### 前置条件

安装 [Rust](https://rustup.rs/) 工具链（任一平台）：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 一行安装

| 平台 | 命令 |
|---|---|
| **macOS / Linux / Windows (WSL)** | `cargo install --git https://github.com/chichuchichu/notify_lark.git` |
| **Windows (PowerShell)** | `cargo install --git https://github.com/chichuchichu/notify_lark.git` |

安装后二进制位于 `~/.cargo/bin/notify_lark`（Windows: `%USERPROFILE%\.cargo\bin\notify_lark.exe`），该目录通常在 PATH 中，无需额外配置。

## 配置飞书 Webhook

1. 打开飞书，进入目标群聊
2. 群设置 → 群机器人 → 添加自定义机器人
3. 复制 webhook 地址

```bash
# macOS / Linux / WSL
export LARK_WEBHOOK_URL="https://open.feishu.cn/open-apis/bot/v2/hook/xxxxx"

# Windows PowerShell
$env:LARK_WEBHOOK_URL = "https://open.feishu.cn/open-apis/bot/v2/hook/xxxxx"
```

建议设为**系统级环境变量**，或使用项目目录下的 `.env` 文件。

## 使用

```bash
# 文本消息
notify_lark "Agent 任务已完成：代码审查通过"

# 管道输入
echo "构建成功，12/12 测试通过" | notify_lark

# 交互卡片
notify_lark -t interactive '{"elements":[{"tag":"div","text":{"content":"详情..."}}]}'
```

## Agent 集成

将 [`agent-rules.md`](agent-rules.md) 中的规则加入你的 AI Agent 的系统提示词，agent 就会在任何需要通知用户的场景自动调用 `notify_lark`。

规则覆盖场景：
- 任务完成通知
- 需要用户决策/确认
- 进度汇报
- 错误提醒
- 请求额外信息

## 项目结构

```
notify_lark/
├── agent-rules.md        # AI Agent 通用集成规则
├── .opencode/
│   └── skills/
│       └── notify-lark/  # opencode Agent 技能
│           └── SKILL.md
├── src/
│   ├── main.rs           # CLI 入口
│   ├── config.rs         # 配置读取
│   └── lark.rs           # 飞书 API 客户端
└── Cargo.toml
```

## 许可

MIT
