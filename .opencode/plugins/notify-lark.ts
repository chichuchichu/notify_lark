import { spawn } from "node:child_process"
import { platform } from "node:os"

const BIN = platform() === "win32" ? "notify_lark.exe" : "notify_lark"

let lastMessageContent = ""

function notify(title: string, body: string) {
  try {
    spawn(BIN, ["--card-title", title, body.slice(0, 500)], {
      stdio: "ignore",
      timeout: 5000,
    }).on("error", () => {})
  } catch {}
}

function extractContent(src: any): string {
  if (!src) return ""
  if (typeof src.content === "string") return src.content
  if (Array.isArray(src.content)) {
    return src.content.map((c: any) =>
      typeof c === "string" ? c : c?.text ?? c?.text?.value ?? ""
    ).join(" ").trim()
  }
  return ""
}

function guessRole(src: any): string {
  return src?.role ?? src?.message?.role ?? src?.data?.role ?? ""
}

export default {
  id: "notify-lark",
  server: async () => {
    return {
      event: async ({ event }: any) => {
        if (event.type === "message.updated") {
          const role = guessRole(event) || guessRole(event.message) || guessRole(event.data)
          if (role === "assistant") {
            const text = extractContent(event) || extractContent(event.message) || extractContent(event.data)
            if (text) lastMessageContent = text
          }
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
