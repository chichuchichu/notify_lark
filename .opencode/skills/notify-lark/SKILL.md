---
name: notify-lark
description: Use when agent tasks are completed to send Lark/Feishu notification. Triggers on task completion, notify, notification, lark, feishu. Any time the user needs attention, call notify_lark to send a message.
---

# notify_lark

跨平台飞书通知工具。任何需要通知用户的场景（任务完成、需要授权、进度汇报、错误提醒），用 `notify_lark` 发飞书消息。

## 安装

```bash
cargo install --git https://github.com/chichuchichu/notify_lark.git
```

### opencode 集成（自动钩子通知）

```bash
notify_lark setup   # 执行一次，创建 plugin + 配置
# 重启 opencode 生效
```

安装后 agent 在以下场景自动发飞书通知：
- 任务完成 → 卡片标题"任务完成" + agent 回复摘要
- 需要授权 → 卡片标题"需要授权" + 请求详情
- 任何等待用户操作的时刻

## 使用速查

| 场景 | 命令 |
|---|---|
| 纯文本通知 | `notify_lark "任务完成: xxx"` |
| 管道输入 | `echo "构建成功" \| notify_lark` |
| 消息卡片 | `notify_lark --card-title "标题" "正文"` |
| 卡片 + 跳转按钮 | `notify_lark --card-title "标题" --card-url "链接" "正文"` |
| 富文本 (post) | `echo '{"zh_cn":...}' \| notify_lark -t post` |
| 自定义卡片 | `echo 'card JSON' \| notify_lark -t interactive` |
| 换行 | text/card 中用 `\n` 表示换行 |

## 配置

```bash
export LARK_WEBHOOK_URL="https://open.feishu.cn/open-apis/bot/v2/hook/xxxxx"
```

或创建 `.env` 文件。未配置时工具会输出完整的 5 步引导。

## 规则

**所有用户通知走飞书，不依赖系统通知。** 调用失败报错但不阻塞任务。
