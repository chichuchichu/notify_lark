import type { Plugin } from "@opencode-ai/plugin"
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

export default (async () => {
  return {
    "permission.asked": async (input: any) => {
      const detail = input?.detail ?? input?.description ?? "agent 请求权限，请查看 opencode 确认"
      notify("需要授权", String(detail).slice(0, 200))
    },
    "session.idle": async () => {
      notify("任务完成", "agent 已完成响应，请查看 opencode")
    },
  }
}) satisfies Plugin
