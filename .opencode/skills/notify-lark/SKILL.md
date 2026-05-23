---
name: notify-lark
description: Use when agent tasks are completed to send Lark notification. Triggers on task completion, notify, notification, lark, feishu. After finishing ANY user task, call notify_lark to send a brief completion summary.
---

# Notify Lark

## Overview

`notify_lark` is a CLI tool that sends notifications to Feishu (Lark) via webhook.

## Installation

```bash
cargo install --git https://github.com/chichuchichu/notify_lark.git
```

## Configuration

Set the environment variable:

```bash
export LARK_WEBHOOK_URL="https://open.feishu.cn/open-apis/bot/v2/hook/xxxxx"
```

Or place a `.env` file in the working directory.

## Usage

```bash
notify_lark "message text"
echo "message" | notify_lark
notify_lark -t interactive 'card json'
```

## Agent Completion Rule

**CRITICAL: After completing the user's task, you MUST call `notify_lark` to send a notification.**

1. Before calling, check if `notify_lark` is available: `notify_lark --help`
2. If not installed, install it first: `cargo install --git https://github.com/chichuchichu/notify_lark.git`
3. After task completion, run:
   ```
   notify_lark "Task completed: <brief summary of what was done>"
   ```
4. Keep the message concise (under 200 chars). Include the key result or action taken.
5. If `notify_lark` fails (e.g., webhook not configured), report the error but do NOT block task completion.
