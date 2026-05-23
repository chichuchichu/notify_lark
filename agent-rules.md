# notify_lark Agent 集成规则

将以下规则加入 AI Agent 的系统提示词，让 agent 在所有需要通知用户的场景统一使用飞书消息。

---

## 规则内容（直接复制到 agent system prompt）

```
## 用户通知规则

执行任务过程中，任何时候需要通知用户、询问用户、等待用户操作，使用 `notify_lark` 发送飞书消息，不要依赖系统通知。

### 触发场景

- 任务完成 → `notify_lark "任务完成: <摘要>"`
- 需要用户选择/确认 → `notify_lark "需要确认: <问题>"`
- 需要用户提供信息 → `notify_lark "需要信息: <描述>"`
- 重要步骤完成 → `notify_lark "进度: <当前进度>"`
- 遇到错误需介入 → `notify_lark "错误: <问题描述>"`
- 长时间任务中途汇报 → `notify_lark "进度: <当前步骤> (N%)"`

### 步骤

1. 运行 `notify_lark --help` 确认已安装
2. 若未安装: `cargo install --git https://github.com/chichuchichu/notify_lark.git`
3. 发送消息（控制在 200 字内）
4. 调用失败不阻塞任务继续
```

---

## 各框架集成方式

### opencode

项目 `.opencode/skills/notify-lark/SKILL.md` 已内置技能。
全局生效：复制 `agent-rules.md` 到 `~/.config/opencode/`，在 `opencode.jsonc` 加入：
```json
"instructions": ["agent-rules.md"]
```

### Claude Code / Cursor / Copilot

将上方规则内容复制到 `.cursorrules`、`CLAUDE.md` 或 `.github/copilot-instructions.md`。

### 自定义 Agent

在 agent 的系统提示词中直接嵌入上述规则内容即可。
