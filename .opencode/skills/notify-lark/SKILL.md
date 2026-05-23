---
name: notify-lark
description: Use when agent tasks are completed to send Lark/Feishu notification. Triggers on task completion, notify, notification, lark, feishu. Any time the user needs attention, call notify_lark to send a message.
---

# notify_lark

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

## 规则

**所有用户通知走飞书，不依赖系统通知。** 调用失败报错但不阻塞任务。
