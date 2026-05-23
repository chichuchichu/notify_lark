import { spawn } from "node:child_process"
import { platform } from "node:os"

const BIN = platform() === "win32" ? "notify_lark.exe" : "notify_lark"

function notify(title: string, body: string) {
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
        if (event.type === "session.idle") {
          notify("任务完成", "agent 已完成响应，请查看 opencode")
        }
        if (event.type === "permission.asked") {
          const detail = event?.detail ?? event?.description ?? "agent 请求权限，请查看 opencode 确认"
          notify("需要授权", String(detail).slice(0, 200))
        }
      },
    }
  },
}
