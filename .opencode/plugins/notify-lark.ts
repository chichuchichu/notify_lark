import { spawn } from "node:child_process"
import { platform } from "node:os"

const BIN = platform() === "win32" ? "notify_lark.exe" : "notify_lark"

const textParts = new Map<string, string>()
let currentMsgId = ""
let lastNotifyTime = 0
let sessionNotified = false

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

export default {
  id: "notify-lark",
  server: async () => {
    return {
      event: async ({ event }: any) => {
        if (event.type === "message.updated") {
          const info = event?.properties?.info
          if (info?.role === "assistant" && info.id !== currentMsgId) {
            currentMsgId = info.id
            textParts.clear()
            sessionNotified = false
          }
        }
        if (event.type === "message.part.updated") {
          const part = event?.properties?.part
          if (part?.type === "text" && part?.messageID === currentMsgId && part.text) {
            textParts.set(part.id, part.text)
          }
        }
        if (event.type === "session.idle") {
          if (sessionNotified) return
          sessionNotified = true
          const text = [...textParts.values()].join(" ").trim()
          const preview = text ? text.slice(0, 200) : "agent 已完成响应，请查看 opencode"
          notify("任务完成", preview)
          textParts.clear()
        }
        if (event.type === "permission.asked") {
          const detail = event?.detail ?? event?.description ?? "agent 请求权限，请查看 opencode 确认"
          notify("需要授权", String(detail).slice(0, 200))
        }
      },
    }
  },
}
