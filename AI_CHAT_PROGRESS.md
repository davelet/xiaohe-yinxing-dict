# 小鹤音形词典 2.0 - AI 对话功能实现进度

### 已实现功能：
1. **子窗口架构** - 独立 OS 窗口，跟随主窗口移动
2. **对话功能** - 气泡消息、后台异步响应（不阻塞 UI）、中断生成
3. **工具集成** - search_text（汉字查编码）、search_code（编码查汉字）、get_help（帮助文档）、list_categories（分类列表）、get_category_stats（分类统计）
4. **设置表单** - 提供商选择、API Key 管理（含清除）、模型配置
5. **隐私保护** - 钥匙串存储、首次使用隐私声明
6. **上下文管理** - 按轮数裁剪历史、token 预算估算
7. **非阻塞测试连接** - 后台执行，按钮显示 loading 状态
8. **URL 校验** - 保存时校验 http/https 格式
9. **Markdown 渲染** - AI 消息使用 egui_commonmark 渲染，支持标题、列表、代码块等
10. **快捷操作按钮** - 复制编码、在词典中查看（跳转词典视图）


### 已修复问题：
1. ~~**多提供商支持不完整**~~ ✅ 已修复 - 所有内置厂商走 OpenAI 兼容协议（`src/ai/client.rs`）
2. ~~**tokio runtime 创建失败无提示**~~ ✅ 已修复 - 改用 `expect` 在启动时明确报错（`src/main.rs`）
3. ~~**上下文裁剪逻辑可能过度裁剪**~~ ✅ 已修复 - `trim_by_token_budget` 保留至少最近 2 轮（`src/ai/chat.rs`）
4. ~~**UI 线程阻塞**~~ ✅ 已修复 - `send_message` 改为 `runtime.spawn` 后台任务 + 每帧 `try_recv` 轮询，不再 `block_on`（`src/ui/chat_viewport.rs`）
5. ~~**测试连接阻塞 UI**~~ ✅ 已修复 - 改为后台 `oneshot` channel + `request_repaint` 通知（`src/ui/chat_viewport.rs`）
6. ~~**max_tokens 上限过高**~~ ✅ 已修复 - 默认值 10000→2048，滑块范围 100000→8192（`src/ai/config.rs` + `src/ui/chat_viewport.rs`）
7. ~~**假流式输出**~~ ✅ 已修复 - 移除逐字 sleep 模拟，改为一次性返回完整响应（`src/ai/client.rs`）
8. ~~**无清除 API Key 按钮**~~ ✅ 已修复 - 设置页新增"清除 API Key"按钮（`src/ui/chat_viewport.rs`）
9. ~~**无 URL 校验**~~ ✅ 已修复 - 保存时校验 http/https 前缀 + 空格检测（`src/ui/chat_viewport.rs`）
10. ~~**Markdown 渲染缺失**~~ ✅ 已修复 - 替换 ui.label 为 CommonMarkViewer（`src/ui/chat_viewport.rs`）
11. ~~**无快捷操作按钮**~~ ✅ 已修复 - 添加复制编码、词典中查看按钮（`src/ui/chat_viewport.rs`）
12. ~~**缺少 3 个工具**~~ ✅ 已修复 - 实现 get_help、list_categories、get_category_stats（`src/ai/tools.rs`）
13. ~~**help_manager 不可跨线程共享**~~ ✅ 已修复 - 改为 Arc<HelpManager>（`src/main.rs`）
14. ~~**真正的流式输出**~~ ✅ 已修复 - 使用 rig-core stream_prompt 逐字流式输出（`src/ai/client.rs` + `src/ai/chat.rs` + `src/ui/chat_viewport.rs`）
15. ~~**请求频率控制**~~ ✅ 已修复 - send_message 增加 500ms 最小间隔限制（`src/ui/chat_viewport.rs` + `src/main.rs`）
16. ~~**多会话存储**~~ ✅ 已修复 - 新增 history.rs，对话持久化到 conversations/ 目录，工具栏支持新建/导出/清空，自动保存（`src/ai/history.rs` + `src/ui/chat_viewport.rs`）


### 遗留问题：
1. ~~**真正的流式输出**~~ ✅ 已修复
2. **search_external_dict 工具** - 外部词典查询工具未实现
3. ~~**多会话存储**~~ ✅ 已修复
4. **重新生成按钮** - 未实现
5. **复制回复按钮** - 未实现（当前仅有“复制编码”，整段回复复制未实现）
6. **三栏布局** - 当前单栏，缺少对话历史侧栏和工具详情面板
7. ~~**请求频率控制**~~ ✅ 已修复
8. **系统代理状态显示** - 未实现
9. **工具降级** - 模型不支持 Function Calling 时的降级处理未实现
10. **日志调试** - 关键事件日志未实现
11. ~~**流式输出不实时刷新**~~ ✅ 已修复 - `poll_chat_stream` 接收 `ctx`，收到 `TextDelta` 时调用 `request_repaint()` 逐字刷新；并持续请求重绘直到流式结束，避免停顿期间 UI 停止刷新（`src/ui/chat_viewport.rs`）
12. **主窗口垂直居中边界** ⚠️ 待确认 - 首次定位用 `monitor_size.y - size.y` 计算居中位置，若窗口高度超过显示器高度会出现负坐标（`src/ui/mod.rs`）
13. **多显示器首次定位重试** ⚠️ 已知行为 - 首帧 `viewport().outer_rect` 可能尚未建立（返回 None），`main_window_positioned` 会保持 false 并每帧重试发命令，直到拿到尺寸后才置位；属预期重试逻辑，但需确认无遗漏（`src/ui/mod.rs`）