use crate::ai::config::{ModelConfig, Provider};
use crate::dict::DictEntry;
use crate::help::HelpManager;
use crate::manager::ManagerState;
use crate::search::SearchEngine;
use crate::types::ChatTab;
use crate::ui::chat_state::{ChatAction, ChatUiState};
use eframe::egui::{self, Color32, RichText};
use std::sync::Arc;

/// 渲染 AI 对话内容（在主窗口内全屏显示）
///
/// 返回 `ChatAction` 让 `ui/mod.rs` 统一处理跨模块副作用（关闭 AI 子窗口、
/// 跳转到词典等），chat 模块自身只持有自己的状态，不直接写 DictApp。
#[allow(clippy::too_many_arguments)]
pub fn render_chat_viewport(
    ui: &mut egui::Ui,
    chat: &mut ChatUiState,
    engine: &Arc<SearchEngine<DictEntry>>,
    help_manager: &Arc<HelpManager>,
    manager: &ManagerState,
) -> Option<ChatAction> {
    // 确保样式正确
    {
        let style = ui.style_mut();
        style.visuals = egui::Visuals::light();
        crate::ui::styles::DictViewStyle::apply(style);
    }

    // 轮询后台 AI 响应（非阻塞）
    let ctx = ui.ctx().clone();
    poll_chat_stream(chat, &ctx);
    poll_chat_test(chat);

    let mandatory_privacy =
        !chat.ai_config.privacy_acknowledged && chat.tab == ChatTab::Conversation;
    if mandatory_privacy || chat.show_privacy_dialog {
        render_privacy_dialog(ui, chat);
    }

    let mut action: Option<ChatAction> = None;

    egui::CentralPanel::default().show_inside(ui, |ui| {
        // 顶部 tab 栏
        ui.horizontal(|ui| {
            let conv_btn = ui.selectable_label(chat.tab == ChatTab::Conversation, "AI 对话");
            let settings_btn = ui.selectable_label(chat.tab == ChatTab::Settings, "AI 设置");

            if conv_btn.clicked() {
                chat.tab = ChatTab::Conversation;
            }
            if settings_btn.clicked() {
                chat.tab = ChatTab::Settings;
            }

            // 添加关闭按钮
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✖ 关闭").clicked() {
                    action = Some(ChatAction::Close);
                }
            });
        });
        ui.separator();

        // 根据当前 tab 渲染内容
        let tab_action = match chat.tab {
            ChatTab::Conversation => {
                render_conversation_tab(ui, chat, engine, help_manager, manager)
            }
            ChatTab::Settings => {
                render_settings_tab(ui, chat);
                None
            }
        };
        if let Some(a) = tab_action {
            action = Some(a);
        }
    });

    action
}

/// 渲染隐私提示弹窗
fn render_privacy_dialog(ui: &mut egui::Ui, chat: &mut ChatUiState) {
    egui::Window::new("隐私声明")
        .collapsible(false)
        .resizable(false)
        .show(ui.ctx(), |ui| {
            ui.add_space(8.0);
            ui.heading("AI 对话功能隐私声明");
            ui.add_space(12.0);

            ui.label("使用 AI 对话功能前，请了解以下信息：");
            ui.add_space(8.0);

            egui::Frame::new()
                .fill(egui::Color32::from_rgb(245, 245, 245))
                .corner_radius(4.0)
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.label("1. 您的对话内容将发送至所选的 AI 服务提供商（如深度求索、智谱等）");
                    ui.add_space(4.0);
                    ui.label("2. API Key 存储在您的操作系统钥匙串中，不会被本应用上传");
                    ui.add_space(4.0);
                    ui.label("3. 本地词库数据不会自动发送至外部服务");
                    ui.add_space(4.0);
                    ui.label("4. 如果启用了外部词典工具，查询时可能发送部分词库内容");
                });

            ui.add_space(16.0);

            ui.vertical_centered(|ui| {
                if ui.button("我已了解，开始使用").clicked() {
                    chat.ai_config.privacy_acknowledged = true;
                    chat.ai_config.save().ok();
                    chat.show_privacy_dialog = false;
                }
            });
        });
}

/// 渲染对话标签页
#[allow(clippy::too_many_arguments)]
fn render_conversation_tab(
    ui: &mut egui::Ui,
    chat: &mut ChatUiState,
    engine: &Arc<SearchEngine<DictEntry>>,
    help_manager: &Arc<HelpManager>,
    manager: &ManagerState,
) -> Option<ChatAction> {
    // 对话工具栏
    render_conversation_toolbar(ui, chat);

    // 未配置模型时显示提示条
    if !chat.ai_config.configured || chat.ai_config.active_model().is_none() {
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(255, 245, 230))
            .corner_radius(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("⚠ 请先在");
                    if ui.link("AI 设置").clicked() {
                        chat.tab = ChatTab::Settings;
                    }
                    ui.label("中添加模型并配置 API Key 后开始对话");
                });
            });
        ui.add_space(4.0);
    }

    // 工具栏和提示条已渲染，available_height 已扣除它们的高度，只需再预留输入框区域
    let input_reserved = 80.0;
    let scroll_height = (ui.available_height() - input_reserved).max(0.0);

    let mut action: Option<ChatAction> = None;
    egui::ScrollArea::vertical()
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(8.0);

            // 渲染消息（按时间顺序交替渲染）
            let msg_count = chat.state.messages.len();
            for idx in 0..msg_count {
                let is_last = idx == msg_count - 1;
                if chat.state.messages[idx].role == crate::ai::chat::Role::User {
                    let content = chat.state.messages[idx].content.clone();
                    render_user_message(ui, &content);
                    ui.add_space(4.0);
                } else {
                    let ai_act =
                        render_ai_message(ui, chat, idx, is_last, engine, help_manager, manager);
                    if ai_act.is_some() {
                        action = ai_act;
                    }
                    ui.add_space(4.0);
                }
            }

            // 正在生成时显示流式内容
            if chat.state.is_generating {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(255, 255, 255))
                        .corner_radius(8.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.set_max_width(350.0);
                            ui.spinner();
                            if let crate::ai::chat::StreamState::Generating(content) =
                                &chat.state.stream_state
                            {
                                if !content.is_empty() {
                                    ui.label(content);
                                } else {
                                    ui.label("AI 正在思考...");
                                }
                            } else {
                                ui.label("AI 正在思考...");
                            }
                        });
                });
            }

            ui.add_space(8.0);
        });

    // 输入区域（与消息区之间的视觉分隔由输入框边框提供）
    render_input_area(ui, chat, engine, help_manager, manager);

    action
}

/// 渲染用户消息气泡
fn render_user_message(ui: &mut egui::Ui, content: &str) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(220, 220, 220))
            .corner_radius(8.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_max_width(350.0);
                // Frame 内部用左对齐布局，避免继承外层 right_to_left 导致文字右对齐
                ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                    ui.label(content);
                });
            });
    });
}

/// 渲染 AI 消息气泡
#[allow(clippy::too_many_arguments)]
fn render_ai_message(
    ui: &mut egui::Ui,
    chat: &mut ChatUiState,
    msg_idx: usize,
    is_last: bool,
    engine: &Arc<SearchEngine<DictEntry>>,
    help_manager: &Arc<HelpManager>,
    manager: &ManagerState,
) -> Option<ChatAction> {
    let msg = &chat.state.messages[msg_idx];
    let content = msg.content.clone();
    let tool_calls = msg.tool_calls.clone();
    let is_interrupted = msg.is_interrupted;

    let mut action: Option<ChatAction> = None;

    ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(255, 255, 255))
            .corner_radius(8.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_max_width(380.0);

                if is_interrupted {
                    ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "⚠ 此回复已被中断");
                    ui.add_space(4.0);
                }

                if is_interrupted && content.is_empty() {
                } else {
                    egui_commonmark::CommonMarkViewer::new().show(ui, &mut chat.md_cache, &content);
                }

                if is_interrupted && !content.is_empty() {
                    ui.add_space(2.0);
                    ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "⚠ 已中断");
                }
            });

        // 快捷操作按钮（仅在非生成中的最后一条消息显示，放在气泡下方）
        if is_last && !chat.state.is_generating {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                let has_codes = !tool_calls.is_empty();
                ui.add_enabled_ui(has_codes, |ui| {
                    if ui.small_button("📋 复制编码").clicked() {
                        let codes: Vec<String> = tool_calls
                            .iter()
                            .filter_map(|tc| {
                                serde_json::from_str::<serde_json::Value>(&tc.result)
                                    .ok()
                                    .and_then(|v| {
                                        v.get("results")?
                                            .as_array()?
                                            .iter()
                                            .filter_map(|r| {
                                                r.get("code")?.as_str().map(String::from)
                                            })
                                            .reduce(|a, b| format!("{}, {}", a, b))
                                    })
                            })
                            .collect();
                        if !codes.is_empty() {
                            ui.ctx().copy_text(codes.join("; "));
                        }
                    }
                });

                ui.add_enabled_ui(has_codes, |ui| {
                    if ui.small_button("🔍 词典中查看").clicked()
                        && let Some(tc) = tool_calls.first()
                        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&tc.arguments)
                        && let Some(q) = v
                            .get("query")
                            .or_else(|| v.get("code"))
                            .and_then(|s| s.as_str())
                    {
                        action = Some(ChatAction::JumpToDict(q.to_string()));
                    }
                });

                if ui.small_button("📋 复制回复").clicked() {
                    ui.ctx().copy_text(content.clone());
                }

                if ui.small_button("🔄 重新生成").clicked()
                    && let Some(user_msg) = chat.state.prepare_regenerate()
                {
                    chat.input = user_msg;
                    send_message(chat, engine, help_manager, manager, &ui.ctx().clone());
                }
            });
        }
    });

    action
}

/// 渲染输入区域
#[allow(clippy::too_many_arguments)]
fn render_input_area(
    ui: &mut egui::Ui,
    chat: &mut ChatUiState,
    engine: &Arc<SearchEngine<DictEntry>>,
    help_manager: &Arc<HelpManager>,
    manager: &ManagerState,
) {
    let ctx = ui.ctx().clone();
    let row_content_height = 67.0; // 输入框内容高度，减小以降低整体高度
    let mut text_edit_has_focus = false;

    ui.horizontal_top(|ui| {
        let btn_width = 72.0;
        let spacing = ui.spacing().item_spacing.x;
        let frame_h_margin = 12.0; // inner_margin 左右各 6
        let input_width = (ui.available_width() - btn_width - spacing - frame_h_margin).max(80.0);

        // 输入框带边框 - 固定高度，内容超出时内部滚动
        let input_frame = egui::Frame::NONE
            .stroke(egui::Stroke::new(
                1.5_f32,
                egui::Color32::from_rgb(130, 130, 145),
            ))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(6, 4));
        let response = input_frame.show(ui, |ui| {
            ui.set_min_height(row_content_height);
            ui.set_max_height(row_content_height);
            ui.set_max_width(input_width);
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut chat.input)
                            .hint_text("输入消息...（Enter 发送，Shift+Enter 换行）")
                            .font(egui::TextStyle::Body)
                            .desired_width(input_width)
                            .desired_rows(3)
                            .frame(egui::Frame::NONE),
                    )
                })
                .inner
        });
        text_edit_has_focus = response.inner.has_focus();
        // 用输入框实际渲染高度作为按钮高度，保证上下严格对齐
        let row_height = response.response.rect.height();

        // 发送/停止按钮（与输入框等高）
        let send_btn = if chat.state.is_generating {
            ui.add_sized([btn_width, row_height], egui::Button::new("⏹ 停止"))
        } else {
            ui.add_sized([btn_width, row_height], egui::Button::new("发送"))
        };
        if send_btn.clicked() {
            if chat.state.is_generating {
                chat.state.interrupt_generation();
            } else if !chat.input.trim().is_empty() {
                send_message(chat, engine, help_manager, manager, &ctx);
            }
        }
    });

    // Enter 发送（Shift+Enter 换行不触发）
    if text_edit_has_focus && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
        send_message(chat, engine, help_manager, manager, &ctx);
    }
}

/// 轮询后台 AI 流式响应（非阻塞，每帧调用）
fn poll_chat_stream(chat: &mut ChatUiState, ctx: &egui::Context) {
    let Some(rx) = &mut chat.rx else { return };
    let mut need_repaint = false;
    loop {
        match rx.try_recv() {
            Ok(crate::ai::client::StreamMessage::TextDelta(delta)) => {
                chat.state.append_stream(delta);
                need_repaint = true;
            }
            Ok(crate::ai::client::StreamMessage::ToolCall {
                tool_name,
                arguments,
                result,
            }) => {
                // 流式结束时从 FinalResponse 提取的带 result 的 tool call
                chat.state.record_tool_call(tool_name, arguments, result);
                need_repaint = true;
            }
            Ok(crate::ai::client::StreamMessage::Complete) => {
                crate::ai_log!("生成完成");
                chat.state.finish_generation();
                save_current_conversation(chat);
                chat.rx = None;
                ctx.request_repaint();
                break;
            }
            Ok(crate::ai::client::StreamMessage::Error(e)) => {
                crate::ai_log!("请求失败: {}", e);
                chat.state.add_ai_message(format!("请求失败: {e}"));
                chat.state.is_generating = false;
                chat.state.abort_handle = None;
                chat.rx = None;
                ctx.request_repaint();
                break;
            }
            Err(_) => break, // 暂无更多消息
        }
    }
    if need_repaint {
        // 流式输出期间主动请求重绘，让 UI 逐字实时刷新
        ctx.request_repaint();
    } else if chat.rx.is_some() {
        // 仍在流式接收中但本帧无新增量时，也请求重绘以保持轮询存活，
        // 避免模型思考/网络停顿期间 UI 因无输入事件而停止刷新
        ctx.request_repaint();
    }
}

/// 轮询测试连接结果（非阻塞，每帧调用）
fn poll_chat_test(chat: &mut ChatUiState) {
    let Some(rx) = &mut chat.test_rx else {
        return;
    };
    match rx.try_recv() {
        Ok(msg) => {
            chat.test_response = msg;
            chat.test_in_progress = false;
            chat.test_rx = None;
        }
        Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
        Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
            chat.test_response = "❌ 测试请求异常终止".to_string();
            chat.test_in_progress = false;
            chat.test_rx = None;
        }
    }
}

/// 保存当前对话到磁盘（仅在 persist_conversations 开启时生效）
fn save_current_conversation(chat: &mut ChatUiState) {
    if chat.state.messages.is_empty() {
        return;
    }
    // 仅在用户开启"记录对话历史"时持久化
    if !chat.ai_config.persist_conversations {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let id = match chat.state.current_conversation_id.clone() {
        Some(id) => id,
        None => {
            let id = crate::ai::history::generate_conversation_id();
            chat.state.current_conversation_id = Some(id.clone());
            id
        }
    };
    let title = crate::ai::history::Conversation::title_from_messages(&chat.state.messages);
    let conv = crate::ai::history::Conversation {
        id,
        title,
        created_at: now,
        updated_at: now,
        messages: chat.state.messages.clone(),
    };
    let _ = chat.conversation_store.save_conversation(&conv);
    chat.conversation_store.enforce_limits(
        chat.ai_config.max_conversations,
        chat.ai_config.max_conversation_disk_mb,
    );
}

/// 渲染对话工具栏（新建、导出、历史切换、删除）
fn render_conversation_toolbar(ui: &mut egui::Ui, chat: &mut ChatUiState) {
    let persist_enabled = chat.ai_config.persist_conversations;

    ui.horizontal_wrapped(|ui| {
        if ui.button("📄 新建对话").clicked() {
            // 保存当前对话再清空（仅 persist 开启时保存）
            save_current_conversation(chat);
            chat.state.clear();
            chat.state.current_conversation_id = None;
        }

        // 显示当前激活的模型
        if let Some(active) = chat.ai_config.active_model() {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("当前模型: ")
                        .size(12.0)
                        .color(egui::Color32::DARK_GRAY),
                );
                ui.label(
                    egui::RichText::new(active.name.to_string())
                        .size(12.0)
                        .color(egui::Color32::DARK_GREEN),
                );
                ui.label(
                    egui::RichText::new(format!("[{} ({})]", active.model, active.provider))
                        .size(12.0)
                        .color(egui::Color32::from_gray(160)),
                );
            });
        }

        ui.separator();

        if ui
            .add_enabled(
                !chat.state.messages.is_empty(),
                egui::Button::new("💾 导出回复"),
            )
            .clicked()
        {
            let conv = crate::ai::history::Conversation {
                id: chat
                    .state
                    .current_conversation_id
                    .clone()
                    .unwrap_or_default(),
                title: crate::ai::history::Conversation::title_from_messages(&chat.state.messages),
                created_at: 0,
                updated_at: 0,
                messages: chat.state.messages.clone(),
            };
            let md = crate::ai::history::ConversationStore::export_markdown(&conv);
            ui.ctx().copy_text(md);
        }

        // 仅在开启持久化时显示历史切换
        if persist_enabled {
            ui.separator();

            // 历史对话切换下拉
            let conversations = chat.conversation_store.list_conversations();
            let current_title = if let Some(id) = &chat.state.current_conversation_id {
                conversations
                    .iter()
                    .find(|c| &c.id == id)
                    .map(|c| c.title.clone())
                    .unwrap_or_else(|| "当前对话".to_string())
            } else {
                "当前对话".to_string()
            };

            egui::ComboBox::from_id_salt("conv_history_combo")
                .width(170.0)
                .selected_text(egui::RichText::new(format!("📂 {current_title}")))
                .show_ui(ui, |ui| {
                    for meta in &conversations {
                        let is_current = chat
                            .state
                            .current_conversation_id
                            .as_ref()
                            .map(|id| id == &meta.id)
                            .unwrap_or(false);
                        let label = if is_current {
                            format!("● {}", meta.title)
                        } else {
                            meta.title.clone()
                        };
                        if ui.selectable_label(is_current, label).clicked() && !is_current {
                            // 加载选中的对话
                            if let Ok(conv) = chat.conversation_store.load_conversation(&meta.id) {
                                chat.state.messages = conv.messages;
                                chat.state.current_conversation_id = Some(conv.id);
                                chat.state.is_generating = false;
                            }
                        }
                    }
                });

            // 删除当前对话记录
            if chat.state.current_conversation_id.is_some()
                && ui.small_button("🗑 删除此记录").clicked()
                && let Some(id) = chat.state.current_conversation_id.take()
            {
                let _ = chat.conversation_store.delete_conversation(&id);
                chat.state.clear();
            }
        }
    });
    ui.separator();
}

/// 发送消息（后台异步，不阻塞 UI 线程）
fn send_message(
    chat: &mut ChatUiState,
    engine: &Arc<SearchEngine<DictEntry>>,
    help_manager: &Arc<HelpManager>,
    manager: &ManagerState,
    ctx: &egui::Context,
) {
    let message = chat.input.trim().to_string();
    if message.is_empty() || chat.state.is_generating {
        return;
    }

    // 请求频率控制：最小间隔 500ms，防止误触连发
    const MIN_INTERVAL_MS: u128 = 500;
    if let Some(last) = chat.last_send_time {
        let elapsed = last.elapsed().as_millis();
        if elapsed < MIN_INTERVAL_MS {
            return;
        }
    }
    chat.last_send_time = Some(std::time::Instant::now());

    // 添加用户消息
    chat.state.add_user_message(message.clone());
    chat.input.clear();
    chat.state.start_generation();

    // 从内存配置中获取当前激活模型
    let ai_config = &chat.ai_config;
    let Some(model_config) = ai_config.active_model().cloned() else {
        chat.state.is_generating = false;
        chat.state
            .add_ai_message("无法发送消息：请先添加并激活一个模型配置".to_string());
        return;
    };
    let api_key = match model_config.get_api_key() {
        Ok(k) => k,
        Err(e) => {
            chat.state.is_generating = false;
            chat.state.add_ai_message(format!("无法发送消息：{e}"));
            return;
        }
    };

    let Some(runtime) = chat.tokio_runtime.as_ref() else {
        chat.state.is_generating = false;
        chat.state
            .add_ai_message("无法发送消息：后台运行时未初始化".to_string());
        return;
    };

    // 准备外部词典数据（仅在有外部词典且用户启用时）
    let external_dict_data =
        if !manager.external_entries.is_empty() && ai_config.enable_external_dict_tool {
            Some(crate::ai::tools::ExternalDictData::from_manager(manager))
        } else {
            None
        };

    let engine_clone = engine.clone();
    let help_mgr_clone = help_manager.clone();

    // 按配置的轮数裁剪历史（仅用于发送给 AI，不修改内存中的完整消息列表）
    let mut trimmed_history = chat
        .state
        .clone_trimmed_history(ai_config.history_rounds as usize);

    // 按 token 预算二次裁剪，确保总 token 不超过模型上下文限制
    // 保守假设模型上下文窗口 = 8 * max_tokens，留出余量给模型输出
    let token_budget = (model_config.max_tokens as usize) * 8;
    crate::ai::chat::ChatState::trim_messages_by_token_budget(&mut trimmed_history, token_budget);

    // 收集多轮上下文（裁剪后的历史，不含当前消息）
    let chat_history = crate::ai::chat::ChatState::messages_to_rig_history(&trimmed_history);

    // 创建 channel 接收响应
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::ai::client::StreamMessage>();
    chat.rx = Some(rx);

    // 在后台 tokio 任务中执行整个异步流程
    let ctx_clone = ctx.clone();
    let ai_config_clone = ai_config.clone();
    let model_config_clone = model_config.clone();
    let message_clone = message.clone();

    crate::ai_log!(
        "发送消息: model={}, history_len={}",
        model_config.model,
        chat_history.len()
    );

    let handle = runtime.spawn(async move {
        let client = match crate::ai::client::create_client(&model_config_clone, &api_key) {
            Ok(c) => c,
            Err(e) => {
                crate::ai_log!("创建客户端失败: {}", e);
                let _ = tx.send(crate::ai::client::StreamMessage::Error(e));
                ctx_clone.request_repaint();
                return;
            }
        };
        let agent = crate::ai::client::build_agent(
            client,
            &model_config_clone,
            &ai_config_clone,
            engine_clone,
            help_mgr_clone,
            external_dict_data,
        );
        crate::ai_log!("Agent 创建成功，开始流式请求");
        let _ =
            crate::ai::client::send_message_stream(&agent, &message_clone, chat_history, tx).await;
        crate::ai_log!("流式请求完成");
        ctx_clone.request_repaint();
    });

    chat.state.abort_handle = Some(handle.abort_handle());
}

/// 渲染设置标签页
fn render_settings_tab(ui: &mut egui::Ui, chat: &mut ChatUiState) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let config = &mut chat.ai_config;
        // 捕获可用宽度，所有区块都约束在此宽度内，避免内容溢出窗口外/重叠
        ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 4.0);

        // 当前激活模型 + 新增按钮（放在列表上方）
        ui.horizontal(|ui| {
            if let Some(active) = config.active_model() {
                ui.label(egui::RichText::new(format!("当前激活: {}（供应商：{} 模型：{}）", active.name, active.provider, active.model)).color(egui::Color32::from_rgb(100, 100, 100)));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕ 新增模型").clicked() {
                    chat.model_editor.show_dialog = true;
                    chat.model_editor.editing_id = None;
                    chat.model_editor.name = String::new();
                    chat.model_editor.provider = Provider::default();
                    chat.model_editor.api_url = String::new();
                    chat.model_editor.model = String::new();
                    chat.model_editor.api_key = String::new();
                    chat.model_editor.temperature = 0.7;
                    chat.model_editor.max_tokens = 2048;
                    chat.model_editor.error_message = String::new();
                    chat.test_response.clear();
                    chat.test_in_progress = false;
                }
            });
        });

        // 模型操作结果提示
        if !chat.save_response.is_empty() {
            let color = if chat.save_response.starts_with("❌") {
                egui::Color32::from_rgb(220, 50, 50)
            } else {
                egui::Color32::from_rgb(34, 150, 80)
            };
            ui.label(egui::RichText::new(&chat.save_response).color(color));
        }

        // 模型列表（限制最大高度，超出滚动）
        ui.group(|ui| {
            if config.models.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    ui.label("暂无保存的模型配置");
                    ui.add_space(16.0);
                });
            } else {
                let active_id = config.active_model_id.clone();
                let mut sorted_models: Vec<ModelConfig> = config.models.clone();
                sorted_models.sort_by(|a, b| {
                    let a_active = Some(&a.id) == active_id.as_ref();
                    let b_active = Some(&b.id) == active_id.as_ref();
                    if a_active && !b_active {
                        std::cmp::Ordering::Less
                    } else if !a_active && b_active {
                        std::cmp::Ordering::Greater
                    } else {
                        b.created_at.cmp(&a.created_at)
                    }
                });
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for (idx, model) in sorted_models.iter().enumerate() {
                            let is_active = config.active_model_id.as_deref() == Some(&model.id);
                            let is_last = idx == sorted_models.len() - 1;
                            // 第一行：激活标记 + 名称 + 右侧操作按钮
                            ui.horizontal(|ui| {
                                if is_active {
                                    ui.label(egui::RichText::new("●").color(egui::Color32::from_rgb(34, 150, 80)));
                                } else {
                                    ui.label(" ");
                                }
                                let name_text = if is_active {
                                    egui::RichText::new(&model.name).strong()
                                } else {
                                    egui::RichText::new(&model.name)
                                };
                                ui.add(egui::Label::new(name_text).truncate());

                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if !is_active {
                                        if ui.small_button(RichText::new("◎")).on_hover_text("激活此模型").clicked() {
                                            if !chat.state.is_generating {
                                                config.active_model_id = Some(model.id.clone());
                                                let _ = config.save();
                                            } else {
                                                chat.save_response = "⚠️ 当前正在生成，请完成后再切换".to_string();
                                            }
                                        }
                                    } else {
                                        ui.add_enabled(false, egui::Button::new(RichText::new("◉").color(Color32::GREEN)));
                                    }
                                    if ui.small_button(RichText::new("\u{270F}").color(Color32::BLUE)).on_hover_text("编辑").clicked() { // ✏️
                                        chat.model_editor.show_dialog = true;
                                        chat.model_editor.editing_id = Some(model.id.clone());
                                        chat.model_editor.name = model.name.clone();
                                        chat.model_editor.provider = model.provider.clone();
                                        chat.model_editor.api_url = model.api_url.clone();
                                        chat.model_editor.model = model.model.clone();
                                        chat.model_editor.api_key = String::new();
                                        chat.model_editor.temperature = model.temperature;
                                        chat.model_editor.max_tokens = model.max_tokens;
                                        chat.model_editor.error_message = String::new();
                                        chat.test_response.clear();
                                        chat.test_in_progress = false;
                                    }
                                    if ui.small_button(RichText::new("🗑").color(Color32::RED)).on_hover_text("删除").clicked() {
                                        chat.confirm_delete = Some((model.id.clone(), model.name.clone()));
                                    }
                                    if ui.small_button(RichText::new("📋").color(Color32::ORANGE)).on_hover_text("复制此配置").clicked()
                                        && let Some(_new_model) = config.duplicate_model(&model.id) {
                                            config.active_model_id = Some(_new_model.id.clone());
                                            let _ = config.save();
                                        }
                                });
                            });

                            // 第二行：提供商 | 模型名 | 时间
                            ui.horizontal_wrapped(|ui| {
                                ui.small(model.provider.to_string());
                                ui.small("|");
                                ui.small(&model.model);
                                let time_str = model.formatted_time_display();
                                if !time_str.is_empty() {
                                    ui.small("|");
                                    ui.small(
                                        egui::RichText::new(time_str)
                                            .color(egui::Color32::from_gray(140)),
                                    );
                                }
                            });
                            if !is_last {
                                ui.separator();
                            }
                        }
                    });
            }
        });

        ui.add_space(12.0);

        // 高级选项
        ui.group(|ui| {
            // 外部词典工具开关（控制 AI 是否可查询本地 Rime 词典）
            if ui
                .add(egui::Checkbox::new(
                    &mut config.enable_external_dict_tool,
                    "启用外部词典工具",
                ))
                .on_hover_text("启用后 AI 可以搜索您加载的外部 Rime 词典内容。查询时会发送本地词库片段到 AI 服务商")
                .changed()
            {
                let _ = config.save();
            }
            ui.horizontal_wrapped(|ui| {
                ui.small("（查询会上传本地词库片段，见");
                if ui
                    .link(egui::RichText::new("隐私声明").text_style(egui::TextStyle::Small))
                    .clicked()
                {
                    chat.show_privacy_dialog = true;
                }
                ui.small("）");
            });

            ui.add_space(4.0);

            egui::Grid::new("ai_global_settings_grid")
                .spacing(egui::vec2(8.0, 8.0))
                .show(ui, |ui| {
                    ui.label("上下文保留轮数:");
                    let history_resp = ui.add(egui::Slider::new(&mut config.history_rounds, 4..=30));
                    history_resp.surrender_focus();
                    ui.end_row();

                    ui.label("");
                    ui.label("AI 在一次对话中最多使用的最近对话记录回合数，超过会被截断");
                    ui.end_row();

                    ui.label("最大工具调用轮次:");
                    let turns_resp = ui.add(egui::Slider::new(&mut config.max_tool_turns, 1..=30));
                    turns_resp.surrender_focus();
                    ui.end_row();

                    ui.label("");
                    ui.label("AI 在一次对话中最多连续调用工具的轮次，防止无限循环消耗额度");
                    ui.end_row();

                    // 拖动滑块松开鼠标后持久化，避免拖动过程中频繁写文件
                    if history_resp.drag_stopped() || turns_resp.drag_stopped() {
                        let _ = config.save();
                    }
                });

            ui.add_space(4.0);

            // 对话历史持久化开关
            ui.separator();
            if ui
                .add(egui::Checkbox::new(
                    &mut config.persist_conversations,
                    "记录对话历史",
                ))
                .on_hover_text("开启后，对话内容会在每次 AI 回复完成后保存到本地，并可在工具栏中切换和删除历史对话。关闭后不再记录新对话，但不会删除已有记录")
                .changed()
            {
                let _ = config.save();
            }

            // 对话保留策略（仅 persist 开启时显示）
            if config.persist_conversations {
                ui.add_space(4.0);
                let mut should_save = false;
                ui.indent("retention_policy", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("最大对话数:");
                        let count_resp = ui.add(egui::Slider::new(
                            &mut config.max_conversations,
                            1..=1000,
                        ));
                        count_resp.surrender_focus();
                        if count_resp.drag_stopped() {
                            should_save = true;
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("最大磁盘占用 (MB):");
                        let disk_resp = ui.add(egui::Slider::new(
                            &mut config.max_conversation_disk_mb,
                            100..=20000,
                        ));
                        disk_resp.surrender_focus();
                        if disk_resp.drag_stopped() {
                            should_save = true;
                        }
                    });
                    ui.small("超出任一限制时，自动删除最旧的对话");
                });
                if should_save {
                    let _ = config.save();
                }
            }

            ui.add_space(8.0);

            // 对话数据管理
            ui.small(format!(
                "当前共有 {} 个本地对话记录（文件位于 {}）",
                chat.conversation_store.list_conversations().len(),
                chat.conversation_store.dir().display()
            ));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("🗑 清空所有对话").clicked() {
                    chat.confirm_clear_conversations = true;
                }
                if ui.button("📂 打开对话目录").clicked() {
                    let _ = open::that(chat.conversation_store.dir());
                }
            });

            ui.add_space(8.0);
            });

        ui.add_space(4.0);

        // 确认删除模型弹窗（状态驱动，跨帧保持）
        if let Some((delete_id, name)) = chat.confirm_delete.clone() {
            let mut should_delete = false;
            let mut should_close = false;
            egui::Window::new("确认删除")
                .collapsible(false)
                .resizable(false)
                .fixed_size([360.0, 140.0])
                .show(ui.ctx(), |ui| {
                    ui.label(format!("确定要删除模型 \"{}\" 吗？", name));
                    ui.label("此操作无法撤销，API Key 将从钥匙串中删除。");
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            should_close = true;
                        }
                        if ui.button("确认删除").clicked() {
                            should_delete = true;
                        }
                    });
                });
            if should_delete {
                config.delete_model(&delete_id);
                let _ = config.save();
            }
            if should_delete || should_close {
                chat.confirm_delete = None;
            }
        }

        // 确认清空所有对话弹窗
        if chat.confirm_clear_conversations {
            let mut should_clear = false;
            let mut should_close = false;
            egui::Window::new("确认清空对话")
                .collapsible(false)
                .resizable(false)
                .fixed_size([380.0, 140.0])
                .show(ui.ctx(), |ui| {
                    ui.label("确定要清空所有本地对话记录吗？");
                    ui.label("此操作无法撤销，所有对话文件将被永久删除。");
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            should_close = true;
                        }
                        if ui.button("确认清空").clicked() {
                            should_clear = true;
                        }
                    });
                });
            if should_clear {
                let _ = chat.conversation_store.clear_all();
                chat.state.clear();
                chat.state.current_conversation_id = None;
            }
            if should_clear || should_close {
                chat.confirm_clear_conversations = false;
            }
        }

        // 渲染模型编辑弹窗
        render_model_edit_dialog(ui, chat);
    });
}

/// 渲染新增/编辑模型弹窗
fn render_model_edit_dialog(ui: &mut egui::Ui, chat: &mut ChatUiState) {
    if !chat.model_editor.show_dialog {
        return;
    }

    let is_new = chat.model_editor.editing_id.is_none();
    let title = if is_new {
        "新增模型"
    } else {
        "编辑模型"
    };

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .default_size([480.0, 360.0])
        .max_width(480.0)
        .show(ui.ctx(), |ui| {
            // 输入框统一样式
            let input_frame = egui::Frame::group(ui.style())
                .stroke(egui::Stroke::new(1.0_f32, egui::Color32::from_gray(180)))
                .corner_radius(4.0)
                .inner_margin(egui::Margin::symmetric(4, 2));

            let label_w = 70.0;
            ui.add_space(8.0);

            // 名称
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(label_w, 0.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label("名称:");
                    },
                );
                ui.add(
                    egui::TextEdit::singleline(&mut chat.model_editor.name)
                        .desired_width(ui.available_width())
                        .frame(input_frame),
                );
            });
            ui.add_space(4.0);

            // 提供商
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(label_w, 0.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label("提供商:");
                    },
                );
                egui::ComboBox::from_id_salt("edit_provider_combo")
                    .width(ui.available_width())
                    .selected_text(chat.model_editor.provider.to_string())
                    .show_ui(ui, |ui| {
                        let providers = [
                            Provider::DeepSeek,
                            Provider::Zhipu,
                            Provider::Moonshot,
                            Provider::Qwen,
                            Provider::Yi,
                            Provider::Doubao,
                            Provider::Hunyuan,
                            Provider::Qianfan,
                            Provider::Qna360,
                            Provider::MiniMax,
                            Provider::Baichuan,
                            Provider::StepFun,
                            Provider::SenseNova,
                            Provider::MiniCPM,
                            Provider::SkyWork,
                            Provider::Mobvoi,
                            Provider::SiliconFlow,
                            Provider::Custom,
                        ];
                        for p in &providers {
                            if ui
                                .selectable_value(
                                    &mut chat.model_editor.provider,
                                    p.clone(),
                                    p.to_string(),
                                )
                                .clicked()
                            {
                                chat.model_editor.api_url = p.default_api_url().to_string();
                                chat.model_editor.model = p.default_model().to_string();
                            }
                        }
                    });
            });
            ui.add_space(4.0);

            // API URL
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(label_w, 0.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label("API URL:");
                    },
                );
                ui.add(
                    egui::TextEdit::singleline(&mut chat.model_editor.api_url)
                        .desired_width(ui.available_width())
                        .frame(input_frame),
                );
            });
            ui.add_space(4.0);

            // 模型名
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(label_w, 0.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label("模型:");
                    },
                );
                ui.add(
                    egui::TextEdit::singleline(&mut chat.model_editor.model)
                        .desired_width(ui.available_width())
                        .frame(input_frame),
                );
            });
            ui.add_space(4.0);

            // API Key
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(label_w, 0.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label("API Key:");
                    },
                );
                ui.add(
                    egui::TextEdit::singleline(&mut chat.model_editor.api_key)
                        .password(true)
                        .desired_width(ui.available_width())
                        .frame(input_frame),
                );
            });
            if !is_new {
                ui.horizontal(|ui| {
                    ui.add_space(label_w + 8.0);
                    ui.small("留空表示不修改");
                });
            }
            ui.add_space(4.0);

            // 温度
            let mut temp_pct = (chat.model_editor.temperature * 100.0).round() as i32;
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(label_w, 0.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label("温度:");
                    },
                );
                ui.add(egui::Slider::new(&mut temp_pct, 0..=200).step_by(1.0));
                ui.label("%");
                chat.model_editor.temperature = temp_pct as f32 / 100.0;
            });
            ui.add_space(4.0);

            // 词元上限
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(label_w, 0.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.label("词元上限:");
                    },
                );
                ui.add(
                    egui::Slider::new(&mut chat.model_editor.max_tokens, 100..=8192).step_by(100.0),
                );
            });

            ui.add_space(8.0);

            // 测试连接
            ui.horizontal(|ui| {
                let can_test = !chat.model_editor.api_url.trim().is_empty()
                    && !chat.model_editor.model.trim().is_empty()
                    && (!chat.model_editor.api_key.trim().is_empty()
                        || chat.model_editor.editing_id.is_some())
                    && !chat.test_in_progress;
                let test_label = if chat.test_in_progress {
                    "测试中..."
                } else {
                    "🔌 测试连接"
                };
                if ui
                    .add_enabled(can_test, egui::Button::new(test_label))
                    .clicked()
                {
                    // 构造临时 ModelConfig
                    let model_config = crate::ai::config::ModelConfig::new(
                        chat.model_editor.name.clone(),
                        chat.model_editor.provider.clone(),
                        chat.model_editor.api_url.trim().to_string(),
                        chat.model_editor.model.trim().to_string(),
                        chat.model_editor.temperature,
                        chat.model_editor.max_tokens,
                    );
                    // 获取 API Key：优先用输入框，为空则从已有模型读取
                    let api_key_result: Result<String, String> =
                        if !chat.model_editor.api_key.trim().is_empty() {
                            Ok(chat.model_editor.api_key.trim().to_string())
                        } else if let Some(edit_id) = &chat.model_editor.editing_id {
                            match chat.ai_config.find_model(edit_id) {
                                Some(m) => m.get_api_key(),
                                None => Err("未找到原模型".to_string()),
                            }
                        } else {
                            Err("API Key 不能为空".to_string())
                        };

                    match api_key_result {
                        Ok(api_key) => {
                            if let Some(runtime) = chat.tokio_runtime.as_ref() {
                                let (tx, rx) = tokio::sync::oneshot::channel::<String>();
                                chat.test_rx = Some(rx);
                                chat.test_in_progress = true;
                                chat.test_response.clear();
                                let ctx_clone = ui.ctx().clone();
                                runtime.spawn(async move {
                                    let result = async {
                                        let agent = crate::ai::client::build_test_agent(
                                            &model_config,
                                            &api_key,
                                        )?;
                                        crate::ai::client::send_message(&agent, "ping").await
                                    }
                                    .await;
                                    let msg = match result {
                                        Ok(_) => "✅ 连接成功".to_string(),
                                        Err(e) => format!("❌ {e}"),
                                    };
                                    let _ = tx.send(msg);
                                    ctx_clone.request_repaint();
                                });
                            } else {
                                chat.test_response = "❌ 运行时未初始化".to_string();
                            }
                        }
                        Err(e) => {
                            chat.test_response = format!("❌ {e}");
                        }
                    }
                }

                if !chat.test_response.is_empty() {
                    let color = if chat.test_response.starts_with("❌") {
                        egui::Color32::from_rgb(220, 50, 50)
                    } else {
                        egui::Color32::from_rgb(34, 150, 80)
                    };
                    ui.add(
                        egui::Label::new(egui::RichText::new(&chat.test_response).color(color))
                            .wrap(),
                    );
                }
            });

            ui.add_space(16.0);

            if !chat.model_editor.error_message.is_empty() {
                ui.label(
                    egui::RichText::new(&chat.model_editor.error_message)
                        .color(egui::Color32::from_rgb(220, 50, 50)),
                );
                ui.add_space(8.0);
            }

            ui.horizontal(|ui| {
                if ui.button("取消").clicked() {
                    chat.model_editor.show_dialog = false;
                    chat.model_editor.error_message = String::new();
                    chat.test_response.clear();
                    chat.test_in_progress = false;
                }

                if ui
                    .button(if is_new {
                        "创建并激活"
                    } else {
                        "保存修改"
                    })
                    .clicked()
                {
                    // 校验
                    let mut error = None;
                    if chat.model_editor.name.trim().is_empty() {
                        error = Some("名称不能为空");
                    } else if chat.model_editor.name.trim().chars().count() > 50 {
                        error = Some("名称长度不能超过 50 个字符");
                    } else if chat.model_editor.api_url.trim().is_empty() {
                        error = Some("API URL 不能为空");
                    } else if !chat.model_editor.api_url.starts_with("http://")
                        && !chat.model_editor.api_url.starts_with("https://")
                    {
                        error = Some("API URL 必须以 http:// 或 https:// 开头");
                    } else if chat.model_editor.model.trim().is_empty() {
                        error = Some("模型名不能为空");
                    } else if is_new && chat.model_editor.api_key.trim().is_empty() {
                        error = Some("API Key 不能为空");
                    }

                    if let Some(e) = error {
                        chat.model_editor.error_message = format!("❌ {e}");
                        return;
                    }

                    let name = chat.model_editor.name.trim().to_string();
                    let provider = chat.model_editor.provider.clone();
                    let api_url = chat.model_editor.api_url.trim().to_string();
                    let model_name = chat.model_editor.model.trim().to_string();
                    let temperature = chat.model_editor.temperature;
                    let max_tokens = chat.model_editor.max_tokens;
                    let api_key = chat.model_editor.api_key.trim().to_string();

                    match &chat.model_editor.editing_id {
                        None => {
                            // 新增
                            let mut new_model = crate::ai::config::ModelConfig::new(
                                name,
                                provider,
                                api_url,
                                model_name,
                                temperature,
                                max_tokens,
                            );
                            new_model.api_key = api_key;
                            chat.ai_config.models.push(new_model);
                            chat.ai_config.active_model_id =
                                Some(chat.ai_config.models.last().unwrap().id.clone());
                        }
                        Some(edit_id) => {
                            // 编辑
                            if let Some(model) = chat.ai_config.find_model_mut(edit_id) {
                                model.name = name;
                                model.provider = provider;
                                model.api_url = api_url;
                                model.model = model_name.clone();
                                model.temperature = temperature;
                                model.max_tokens = max_tokens;
                                if !api_key.is_empty() {
                                    model.api_key = api_key;
                                } else if model.api_key.is_empty() {
                                    chat.save_response =
                                        "❌ 该模型尚未配置 API Key，请输入 API Key".to_string();
                                    return;
                                }
                                model.touch();
                            }
                        }
                    }

                    chat.ai_config.validate_and_fix();
                    let _ = chat.ai_config.save();
                    chat.model_editor.show_dialog = false;
                    chat.test_response.clear();
                    chat.test_in_progress = false;
                    chat.save_response.clear();
                }
            });
        });
}
