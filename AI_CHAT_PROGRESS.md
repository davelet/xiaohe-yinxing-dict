# 小鹤音形词典 2.0 — AI 对话功能实现进度

### 已实现功能：
1. **子窗口架构** - 500x650 独立 OS 窗口，跟随主窗口移动
2. **对话功能** - 气泡消息、流式响应、中断生成
3. **工具集成** - search_text（汉字查编码）、search_code（编码查汉字）
4. **设置表单** - 提供商选择、API Key 管理、模型配置、快速配置
5. **隐私保护** - 钥匙串存储、首次使用隐私声明
6. **上下文管理** - 按轮数裁剪历史、token 预算估算


### 已修复问题：
1. ~~**多提供商支持不完整**~~ ✅ 已修复 - `create_client` 现按提供商分派：OpenAI/DeepSeek/Ollama/Custom 使用 OpenAI 兼容客户端并设置对应 base_url；Anthropic/Gemini 暂不支持时返回明确错误提示（`src/ai/client.rs`）
2. ~~**tokio runtime 创建失败无提示**~~ ✅ 已修复 - 改用 `expect` 在启动时给出明确报错，不再静默失败（`src/main.rs`）
3. ~~**上下文裁剪逻辑可能过度裁剪**~~ ✅ 已修复 - `trim_by_token_budget` 现保留至少最近 2 轮对话，并用 `saturating_sub` 防止下溢（`src/ai/chat.rs`）

### 遗留问题：
1. **真正的流式输出** - 当前仍为模拟逐字输出（先获取完整响应再逐字发送）。rig-core 0.39 的流式 API（`StreamingPromptRequest` / `MultiTurnStreamItem`，含多轮工具调用）较复杂，暂列为后续版本改进项（`src/ai/client.rs` `send_message_stream`）

