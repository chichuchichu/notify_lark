import { spawn } from "node:child_process"
import { platform } from "node:os"

const BIN = platform() === "win32" ? "notify_lark.exe" : "notify_lark"

let lastNotifyTime = 0
let lastMessageContent = ""

function notify(title: string, body: string) {
  const now = Date.now()
  if (now - lastNotifyTime < 3000) return
  lastNotifyTime = now
  try {
    spawn(BIN, ["--card-title", title, body.slice(0, 500)], {
      stdio: "ignore",
      timeout: 5000,
    }).on("error", () => {})
  } catch {}
}

function extractContent(msg: any): string {
  if (!msg?.content) return ""
  if (typeof msg.content === "string") return msg.content
  if (Array.isArray(msg.content)) {
    return msg.content.map((c: any) => c.text ?? "").join(" ").trim()
  }
  return ""
}

export default {
  id: "notify-lark",
  server: async () => {
    return {
      event: async ({ event }: any) => {
        if (event.type === "message.updated" && event?.message?.role === "assistant") {
          const text = extractContent(event.message)
          if (text) lastMessageContent = text
        }
        if (event.type === "session.idle") {
          const preview = lastMessageContent
            ? lastMessageContent.trim().slice(0, 200)
            : "agent 已完成响应，请查看 opencode"
          notify("任务完成", preview)
          lastMessageContent = ""
        }
        if (event.type === "permission.asked") {
          const detail = event?.detail ?? event?.description ?? "agent 请求权限，请查看 opencode 确认"
          notify("需要授权", String(detail).slice(0, 200))
        }
      },
    }
  },
}
