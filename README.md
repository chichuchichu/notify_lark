# notify_lark

跨平台飞书通知工具，AI Agent 任务中任何需要通知用户的场景，通过飞书机器人发送消息。

## 安装

```bash
# 需要 Rust 工具链（https://rustup.rs/）
cargo install --git https://github.com/chichuchichu/notify_lark.git
```

安装后 `notify_lark` 位于 `~/.cargo/bin/`，通常在 PATH 中，可直接调用。

## 配置

```bash
export LARK_WEBHOOK_URL="https://open.feishu.cn/open-apis/bot/v2/hook/xxxxx"
```

也可创建 `.env` 文件，或设为系统环境变量。如果未配置，工具会提示完整操作步骤。

## 使用方式

### 快速消息（text — 最常用，默认）

```bash
notify_lark "任务完成: 代码审查通过，12/12 测试通过"
echo "构建成功" | notify_lark
```

### 富文本消息（post — 带标题、段落、链接）

```bash
echo '{"zh_cn":{"title":"构建通知","content":[[{"tag":"text","text":"构建成功"}]],"links":[[{"tag":"a","text":"查看报告","href":"https://ci.example.com"}]]}}' | notify_lark -t post
```

### 消息卡片（interactive — 可配置按钮跳转 URL）

```bash
echo '{"schema":"2.0","header":{"title":{"tag":"plain_text","content":"任务完成"}},"body":{"elements":[{"tag":"markdown","content":"构建通过，12/12 测试成功"}]}}' | notify_lark -t interactive

# 快捷方式：标题 + 正文 + 可选按钮
notify_lark --card-title "通知" "这是一条卡片消息"
notify_lark --card-title "构建通过" --card-url "https://github.com" --card-button "查看仓库" "12/12 测试通过"
```

> **换行**：text 和 card 消息中可用 `\n` 表示换行（所有平台通用）
>
> **Windows PowerShell**：传递复杂 JSON 时双引号会被剥离，请用 stdin 管道（`echo '...' | notify_lark -t post`）；Linux/macOS 可直接传参数。

### 集成 opencode（自动钩子通知）

```bash
notify_lark setup    # 安装成功后执行一次
```

自动创建 opencode plugin，在所有 agent 的以下场景自动发送飞书通知：
- 任务完成
- 需要授权/确认
- 任何等待用户操作的时刻

重启 opencode 生效。

### 配置管理

```bash
notify_lark setup    # 创建/更新 opencode plugin 及配置文件（幂等）
```

## 使用方式速查表

| 场景 | 命令 | 效果 |
|---|---|---|
| 最简通知 | `notify_lark "消息"` | 纯文本 |
| 管道输入 | `echo "消息" \| notify_lark` | 纯文本 |
| 消息卡片 | `notify_lark --card-title "标题" "正文"` | 带标题的卡片 |
| 卡片+按钮 | `notify_lark --card-title "T" --card-url "U" "正文"` | 标题+正文+跳转按钮 |
| 富文本 | `echo 'post JSON' \| notify_lark -t post` | 分段+链接 |
| 交互卡片 | `echo 'interactive JSON' \| notify_lark -t interactive` | 自定义卡片 |

## 消息类型对比

| 类型 | `-t` 参数 | 能力 | 常见用法 |
|---|---|---|---|
| `text` | 默认 | 纯文本 | 任务完成、错误提醒、进度汇报 |
| `post` | `-t post` | 段落、链接、图片、@人 | 复杂汇报、多段消息 |
| `interactive` | `-t interactive` | 标题、按钮、选择器、Markdown | 需要用户跳转操作的场景 |

## 环境变量参考

| 变量 | 必填 | 说明 |
|---|---|---|
| `LARK_WEBHOOK_URL` | 是 | 飞书自定义机器人 webhook 地址 |

## 项目结构

```
notify_lark/
├── agent-rules.md              # AI Agent 通用集成规则（用于非 opencode 框架）
├── .opencode/
│   ├── plugin/
│   │   └── notify-lark.ts      # opencode 自动通知插件
│   └── skills/
│       └── notify-lark/        # opencode 技能
├── src/
│   ├── main.rs                 # CLI 入口（含 setup 子命令）
│   ├── config.rs               # 配置读取（含引导提示）
│   └── lark.rs                 # 飞书 API 客户端
└── Cargo.toml
```

## License

MIT — 详见 [LICENSE](LICENSE)
