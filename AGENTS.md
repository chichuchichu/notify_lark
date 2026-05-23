# notify_lark 通知规则

## 任务完成后必须通知

当你完成用户的任何任务（代码编写、问题修复、文件操作、构建等），**必须**调用 `notify_lark` 发送飞书通知。

### 步骤

1. 确认 notify_lark 已安装：运行 `notify_lark --help`
2. 若未安装，先执行：`cargo install --git https://github.com/chichuchichu/notify_lark.git`
3. 任务完成后，发送通知：
   ```
   notify_lark "任务完成: <一句话总结做了什么>"
   ```
4. 消息控制在 200 字以内
5. 如果调用失败（如未配置 webhook），报错但不阻塞任务完成
