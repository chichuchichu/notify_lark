# notify_lark

跨平台飞书通知工具，agent 或 CI 任务中任何需要通知用户的场景，通过飞书机器人发送消息。

## 安装

### 前置条件

安装 [Rust](https://rustup.rs/) 工具链（任一平台）：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 一行安装

macOS / Linux / Windows 通用：

```bash
cargo install --git https://github.com/chichuchichu/notify_lark.git
```

安装后二进制位于 `~/.cargo/bin/notify_lark`（Windows: `%USERPROFILE%\.cargo\bin\notify_lark.exe`），该目录通常在 PATH 中。

## 配置飞书 Webhook

1. 飞书群设置 → 群机器人 → 添加自定义机器人
2. 复制 webhook 地址，设为环境变量：

```bash
# macOS / Linux / WSL
export LARK_WEBHOOK_URL="https://open.feishu.cn/open-apis/bot/v2/hook/xxxxx"

# Windows PowerShell
$env:LARK_WEBHOOK_URL = "https://open.feishu.cn/open-apis/bot/v2/hook/xxxxx"
```

建议设为**系统级环境变量**，或使用 `.env` 文件。

## opencode 集成（一键安装 hook）

```bash
notify_lark setup
```

自动创建 opencode plugin 和配置文件，实现：
- 任务完成 → 自动飞书通知
- 需要权限/用户操作 → 自动飞书通知
- 任何 agent 中断场景 → 自动飞书通知

重启 opencode 后生效。其他 agent 框架请参考 [`agent-rules.md`](agent-rules.md)。

## 使用

```bash
# 文本消息
notify_lark "Agent 任务已完成"

# 管道输入
echo "构建成功" | notify_lark

# 交互卡片
notify_lark -t interactive '{"elements":[{"tag":"div","text":{"content":"详情..."}}]}'

# 查看帮助
notify_lark --help
```

## 项目结构

```
notify_lark/
├── agent-rules.md              # AI Agent 通用集成规则
├── .opencode/
│   ├── plugin/
│   │   └── notify-lark.ts      # opencode 自动通知插件
│   └── skills/
│       └── notify-lark/        # opencode 技能
├── src/
│   ├── main.rs                 # CLI 入口（含 setup 子命令）
│   ├── config.rs               # 配置读取
│   └── lark.rs                 # 飞书 API 客户端
└── Cargo.toml
```

## 许可

MIT
