import type { Plugin } from "@opencode-ai/plugin"
import { execSync } from "node:child_process"
import { platform } from "node:os"

const BIN = platform() === "win32" ? "notify_lark.exe" : "notify_lark"

function notify(msg: string) {
  try {
    execSync(`${BIN} "${msg.replace(/"/g, '\\"')}"`, {
      stdio: "ignore",
      timeout: 5000,
    })
  } catch {
    // 失败不阻塞
  }
}

export default (async () => {
  return {
    // 需要权限时（中断agent执行等待用户操作）
    "permission.ask": async () => {
      notify("需要确认: agent 请求权限，请查看 opencode 操作")
    },
    // agent 完成工具调用，输出回复后（任务完成或中断，交还控制权给用户）
    "chat.message": async (input) => {
      // 只在 assistant 发消息时通知（用户发消息或交互流程中跳过）
      if (input?.role !== "assistant") return
      // 极短消息跳过（空响应或无意义内容）
      const text = (input?.content ?? "").trim()
      if (text.length < 5) return
      const preview = text.length > 150 ? text.slice(0, 150) + "..." : text
      notify(`任务完成: ${preview}`)
    },
  }
}) satisfies Plugin
