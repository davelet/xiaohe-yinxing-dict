use crate::DictApp;
use crate::ai::config::{AiConfig, Provider};
use eframe::egui;

/// 渲染 AI 对话内容（在主窗口内全屏显示）
pub fn render_chat_viewport(ui: &mut egui::Ui, app: &mut DictApp) {
    // 确保样式正确
    {
        let style = ui.style_mut();
        style.visuals = egui::Visuals::light();
        crate::ui::styles::DictViewStyle::apply(style);
    }

    // 轮询后台 AI 响应（非阻塞）
    let ctx = ui.ctx().clone();
    poll_chat_stream(app, &ctx);
    poll_chat_test(app);

    // 加载 AI 配置一次，全程使用同一实例
    let ai_config = AiConfig::load();
    let mandatory_privacy =
        !ai_config.privacy_acknowledged && app.chat_tab == crate::app::ChatTab::Conversation;
    if mandatory_privacy || app.show_privacy_dialog {
        render_privacy_dialog(ui, app);
    }

    egui::CentralPanel::default().show_inside(ui, |ui| {
        // 顶部 tab 栏
        ui.horizontal(|ui| {
            let conv_btn =
                ui.selectable_label(app.chat_tab == crate::app::ChatTab::Conversation, "AI 对话");
            let settings_btn =
                ui.selectable_label(app.chat_tab == crate::app::ChatTab::Settings, "AI 设置");

            if conv_btn.clicked() {
                app.chat_tab = crate::app::ChatTab::Conversation;
            }
            if settings_btn.clicked() {
                app.chat_tab = crate::app::ChatTab::Settings;
            }

            // 添加关闭按钮
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("✖ 关闭").clicked() {
                    app.show_chat_viewport = false;
                    app.current_view = crate::app::ViewMode::Dict;
                }
            });
        });
        ui.separator();

        // 根据当前 tab 渲染内容
        match app.chat_tab {
            crate::app::ChatTab::Conversation => {
                render_conversation_tab(ui, app, &ai_config);
            }
            crate::app::ChatTab::Settings => {
                render_settings_tab(ui, app, ai_config);
            }
        }
    });
}

/// 渲染隐私提示弹窗
fn render_privacy_dialog(ui: &mut egui::Ui, app: &mut DictApp) {
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
                    let mut config = AiConfig::load();
                    config.privacy_acknowledged = true;
                    config.save().ok();
                    app.show_privacy_dialog = false;
                }
            });
        });
}

/// 渲染对话标签页
fn render_conversation_tab(ui: &mut egui::Ui, app: &mut DictApp, ai_config: &AiConfig) {
    // 对话工具栏
    render_conversation_toolbar(ui, app, ai_config);

    // 未配置模型时显示提示条
    if !ai_config.configured || ai_config.active_model().is_none() {
        ui.add_space(4.0);
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(255, 245, 230))
            .corner_radius(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("⚠ 请先在");
                    if ui.link("AI 设置").clicked() {
                        app.chat_tab = crate::app::ChatTab::Settings;
                    }
                    ui.label("中添加模型并配置 API Key 后开始对话");
                });
            });
        ui.add_space(4.0);
    }

    // 消息列表区域：预留输入框高度（分隔线 + 间距 + 边框输入区）
    let input_reserved = 100.0;
    let scroll_height = (ui.available_height() - input_reserved).max(0.0);
    egui::ScrollArea::vertical()
        .max_height(scroll_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(8.0);

            // 渲染消息（按时间顺序交替渲染）
            let msg_count = app.chat_state.messages.len();
            for idx in 0..msg_count {
                let is_last = idx == msg_count - 1;
                if app.chat_state.messages[idx].role == crate::ai::chat::Role::User {
                    let content = app.chat_state.messages[idx].content.clone();
                    render_user_message(ui, &content);
                    ui.add_space(4.0);
                } else {
                    render_ai_message(ui, app, idx, is_last);
                    ui.add_space(4.0);
                }
            }

            // 正在生成时显示流式内容
            if app.chat_state.is_generating {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(255, 255, 255))
                        .corner_radius(8.0)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            ui.set_max_width(350.0);
                            ui.spinner();
                            if let crate::ai::chat::StreamState::Generating(content) =
                                &app.chat_state.stream_state
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

    ui.separator();

    // 输入区域
    render_input_area(ui, app);
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
fn render_ai_message(ui: &mut egui::Ui, app: &mut DictApp, msg_idx: usize, is_last: bool) {
    let msg = &app.chat_state.messages[msg_idx];
    let content = msg.content.clone();
    let tool_calls = msg.tool_calls.clone();
    let is_interrupted = msg.is_interrupted;

    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(255, 255, 255))
            .corner_radius(8.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_max_width(380.0);

                // Markdown 渲染
                if is_interrupted {
                    ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "⚠ 此回复已被中断");
                    ui.add_space(4.0);
                }

                if is_interrupted && content.is_empty() {
                    // 无内容，不渲染
                } else {
                    egui_commonmark::CommonMarkViewer::new().show(
                        ui,
                        &mut app.chat_md_cache,
                        &content,
                    );
                }

                if is_interrupted && !content.is_empty() {
                    ui.add_space(2.0);
                    ui.colored_label(egui::Color32::from_rgb(150, 150, 150), "⚠ 已中断");
                }

                // 渲染工具调用结果
                if !tool_calls.is_empty() {
                    ui.add_space(6.0);
                    ui.separator();
                    let tool_calls_for_detail = tool_calls.clone();
                    ui.collapsing("工具调用详情", |ui| {
                        for tc in &tool_calls_for_detail {
                            ui.label(format!("🔧 {}: {}", tc.tool_name, tc.arguments));
                            ui.small(&tc.result);
                            ui.add_space(2.0);
                        }
                    });
                }

                // 快捷操作按钮（仅在非生成中的最后一条消息显示）
                if is_last && !app.chat_state.is_generating {
                    ui.add_space(6.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        // 复制编码
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

                        // 在词典中查看
                        ui.add_enabled_ui(has_codes, |ui| {
                            if ui.small_button("🔍 词典中查看").clicked()
                                && let Some(tc) = tool_calls.first()
                                && let Ok(v) =
                                    serde_json::from_str::<serde_json::Value>(&tc.arguments)
                                && let Some(q) = v
                                    .get("query")
                                    .or_else(|| v.get("code"))
                                    .and_then(|s| s.as_str())
                            {
                                app.current_view = crate::app::ViewMode::Dict;
                                app.query = q.to_string();
                                app.search_dirty = true;
                                app.show_chat_viewport = false;
                            }
                        });

                        // 复制完整回复
                        if ui.small_button("📋 复制回复").clicked() {
                            ui.ctx().copy_text(content.clone());
                        }

                        // 重新生成
                        if ui.small_button("🔄 重新生成").clicked()
                            && let Some(user_msg) = app.chat_state.prepare_regenerate()
                        {
                            app.chat_input = user_msg;
                            send_message(app, &ui.ctx().clone());
                        }
                    });
                }
            });
    });
}

/// 渲染输入区域
fn render_input_area(ui: &mut egui::Ui, app: &mut DictApp) {
    let ctx = ui.ctx().clone();
    let input_height = 52.0;
    let frame_v_margin = 12.0; // inner_margin 上下各 6
    let mut text_edit_has_focus = false;

    ui.horizontal_top(|ui| {
        let btn_width = 80.0;
        let spacing = ui.spacing().item_spacing.x;
        let frame_h_margin = 16.0; // inner_margin 左右各 8
        let input_width = (ui.available_width() - btn_width - spacing - frame_h_margin).max(80.0);

        // 输入框带边框 - 固定高度，内容超出时内部滚动
        let input_frame = egui::Frame::NONE
            .stroke(egui::Stroke::new(
                1.5,
                egui::Color32::from_rgb(130, 130, 145),
            ))
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(8, 6));
        let response = input_frame.show(ui, |ui| {
            ui.set_min_height(input_height);
            ui.set_max_height(input_height);
            ui.set_max_width(input_width);
            let scroll = egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut app.chat_input)
                            .hint_text("输入消息...（Enter 发送，Shift+Enter 换行）")
                            .font(egui::TextStyle::Body)
                            .desired_width(input_width)
                            .frame(egui::Frame::NONE),
                    )
                });
            scroll.inner
        });
        text_edit_has_focus = response.inner.has_focus();

        // 发送/停止按钮（与输入框等高，包含 frame 边距）
        let btn_height = input_height + frame_v_margin;
        if app.chat_state.is_generating {
            if ui
                .add_sized([btn_width, btn_height], egui::Button::new("⏹ 停止"))
                .clicked()
            {
                app.chat_state.interrupt_generation();
            }
        } else {
            let send_btn = ui.add_sized([btn_width, btn_height], egui::Button::new("发送"));
            if send_btn.clicked() && !app.chat_input.trim().is_empty() {
                send_message(app, &ctx);
            }
        }
    });

    // Enter 发送（Shift+Enter 换行不触发）
    if text_edit_has_focus && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift) {
        send_message(app, &ctx);
    }
}

/// 轮询后台 AI 流式响应（非阻塞，每帧调用）
fn poll_chat_stream(app: &mut DictApp, ctx: &egui::Context) {
    let Some(rx) = &mut app.chat_rx else { return };
    let mut need_repaint = false;
    loop {
        match rx.try_recv() {
            Ok(crate::ai::client::StreamMessage::TextDelta(delta)) => {
                app.chat_state.append_stream(delta);
                need_repaint = true;
            }
            Ok(crate::ai::client::StreamMessage::ToolCall {
                tool_name,
                arguments,
                result,
            }) => {
                // 流式结束时从 FinalResponse 提取的带 result 的 tool call
                app.chat_state
                    .record_tool_call(tool_name, arguments, result);
                need_repaint = true;
            }
            Ok(crate::ai::client::StreamMessage::Complete) => {
                crate::ai_log!("生成完成");
                app.chat_state.finish_generation();
                let ai_config = AiConfig::load();
                save_current_conversation(app, &ai_config);
                app.chat_rx = None;
                ctx.request_repaint();
                break;
            }
            Ok(crate::ai::client::StreamMessage::Error(e)) => {
                crate::ai_log!("请求失败: {}", e);
                app.chat_state.add_ai_message(format!("请求失败: {e}"));
                app.chat_state.is_generating = false;
                app.chat_state.abort_handle = None;
                app.chat_rx = None;
                ctx.request_repaint();
                break;
            }
            Err(_) => break, // 暂无更多消息
        }
    }
    if need_repaint {
        // 流式输出期间主动请求重绘，让 UI 逐字实时刷新
        ctx.request_repaint();
    } else if app.chat_rx.is_some() {
        // 仍在流式接收中但本帧无新增量时，也请求重绘以保持轮询存活，
        // 避免模型思考/网络停顿期间 UI 因无输入事件而停止刷新
        ctx.request_repaint();
    }
}

/// 轮询测试连接结果（非阻塞，每帧调用）
fn poll_chat_test(app: &mut DictApp) {
    let Some(rx) = &mut app.chat_test_rx else {
        return;
    };
    match rx.try_recv() {
        Ok(msg) => {
            app.chat_test_response = msg;
            app.chat_test_in_progress = false;
            app.chat_test_rx = None;
        }
        Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
        Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
            app.chat_test_response = "❌ 测试请求异常终止".to_string();
            app.chat_test_in_progress = false;
            app.chat_test_rx = None;
        }
    }
}

/// 保存当前对话到磁盘（仅在 persist_conversations 开启时生效）
fn save_current_conversation(app: &mut DictApp, ai_config: &AiConfig) {
    if app.chat_state.messages.is_empty() {
        return;
    }
    // 仅在用户开启"记录对话历史"时持久化
    if !ai_config.persist_conversations {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let id = match app.chat_state.current_conversation_id.clone() {
        Some(id) => id,
        None => {
            let id = crate::ai::history::generate_conversation_id();
            app.chat_state.current_conversation_id = Some(id.clone());
            id
        }
    };
    let title = crate::ai::history::Conversation::title_from_messages(&app.chat_state.messages);
    let conv = crate::ai::history::Conversation {
        id,
        title,
        created_at: now,
        updated_at: now,
        messages: app.chat_state.messages.clone(),
    };
    let _ = app.conversation_store.save_conversation(&conv);
    app.conversation_store.enforce_max_conversations(50);
}

/// 渲染对话工具栏（新建、导出、历史切换、删除）
fn render_conversation_toolbar(ui: &mut egui::Ui, app: &mut DictApp, ai_config: &AiConfig) {
    let persist_enabled = ai_config.persist_conversations;

    ui.horizontal(|ui| {
        if ui.button("📄 新建对话").clicked() {
            // 保存当前对话再清空（仅 persist 开启时保存）
            save_current_conversation(app, ai_config);
            app.chat_state.clear();
            app.chat_state.current_conversation_id = None;
        }

        // 显示当前激活的模型
        if let Some(active) = ai_config.active_model() {
            ui.separator();
            ui.label(
                egui::RichText::new(format!("当前模型: {} - {}", active.name, active.model))
                    .size(12.0)
                    .color(egui::Color32::from_gray(100)),
            );
        }

        ui.separator();

        if ui
            .add_enabled(
                !app.chat_state.messages.is_empty(),
                egui::Button::new("💾 导出回复"),
            )
            .clicked()
        {
            let conv = crate::ai::history::Conversation {
                id: app
                    .chat_state
                    .current_conversation_id
                    .clone()
                    .unwrap_or_default(),
                title: crate::ai::history::Conversation::title_from_messages(
                    &app.chat_state.messages,
                ),
                created_at: 0,
                updated_at: 0,
                messages: app.chat_state.messages.clone(),
            };
            let md = crate::ai::history::ConversationStore::export_markdown(&conv);
            ui.ctx().copy_text(md);
        }

        ui.separator();

        if ui.button("🗑 清空当前").clicked() {
            app.chat_state.clear();
            app.chat_state.current_conversation_id = None;
        }

        // 仅在开启持久化时显示历史切换和删除功能
        if persist_enabled {
            ui.separator();

            // 历史对话切换下拉
            let conversations = app.conversation_store.list_conversations();
            let current_title = if let Some(id) = &app.chat_state.current_conversation_id {
                conversations
                    .iter()
                    .find(|c| &c.id == id)
                    .map(|c| c.title.clone())
                    .unwrap_or_else(|| "当前对话".to_string())
            } else {
                "当前对话".to_string()
            };

            egui::ComboBox::from_id_salt("conv_history_combo")
                .width(160.0)
                .selected_text(egui::RichText::new(format!("📂 {current_title}")).small())
                .show_ui(ui, |ui| {
                    for meta in &conversations {
                        let is_current = app
                            .chat_state
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
                            if let Ok(conv) = app.conversation_store.load_conversation(&meta.id) {
                                app.chat_state.messages = conv.messages;
                                app.chat_state.current_conversation_id = Some(conv.id);
                                app.chat_state.is_generating = false;
                            }
                        }
                    }
                });

            // 删除当前对话记录
            if app.chat_state.current_conversation_id.is_some()
                && ui.small_button("🗑 删除此记录").clicked()
                && let Some(id) = app.chat_state.current_conversation_id.take()
            {
                let _ = app.conversation_store.delete_conversation(&id);
                app.chat_state.clear();
            }

            // 删除所有记录
            if !conversations.is_empty() && ui.small_button("⚠ 删除所有记录").clicked() {
                let _ = app.conversation_store.clear_all();
                app.chat_state.clear();
                app.chat_state.current_conversation_id = None;
            }
        }
    });
    ui.separator();
}

/// 发送消息（后台异步，不阻塞 UI 线程）
fn send_message(app: &mut DictApp, ctx: &egui::Context) {
    let message = app.chat_input.trim().to_string();
    if message.is_empty() || app.chat_state.is_generating {
        return;
    }

    // 请求频率控制：最小间隔 500ms，防止误触连发
    const MIN_INTERVAL_MS: u128 = 500;
    if let Some(last) = app.last_chat_send_time {
        let elapsed = last.elapsed().as_millis();
        if elapsed < MIN_INTERVAL_MS {
            return;
        }
    }
    app.last_chat_send_time = Some(std::time::Instant::now());

    // 添加用户消息
    app.chat_state.add_user_message(message.clone());
    app.chat_input.clear();
    app.chat_state.start_generation();

    // 从磁盘加载最新配置，获取当前激活模型
    let ai_config = crate::ai::config::AiConfig::load();
    let Some(model_config) = ai_config.active_model() else {
        app.chat_state.is_generating = false;
        app.chat_state
            .add_ai_message("无法发送消息：请先添加并激活一个模型配置".to_string());
        return;
    };
    let api_key = match model_config.get_api_key() {
        Ok(k) => k,
        Err(e) => {
            app.chat_state.is_generating = false;
            app.chat_state.add_ai_message(format!("无法发送消息：{e}"));
            return;
        }
    };

    let runtime = app.tokio_runtime.as_ref();
    let Some(runtime) = runtime else {
        app.chat_state.is_generating = false;
        app.chat_state
            .add_ai_message("无法发送消息：后台运行时未初始化".to_string());
        return;
    };

    // 准备外部词典数据（仅在有外部词典且用户启用时）
    let external_dict_data =
        if !app.manager.external_entries.is_empty() && ai_config.enable_external_dict_tool {
            Some(crate::ai::tools::ExternalDictData::from_manager(
                &app.manager,
            ))
        } else {
            None
        };

    let engine = app.engine.clone();
    let help_mgr = app.help_manager.clone();

    // 按配置的轮数裁剪历史
    app.chat_state
        .trim_history(ai_config.history_rounds as usize);

    // 按 token 预算二次裁剪，确保总 token 不超过模型上下文限制
    // 保守假设模型上下文窗口 = 8 * max_tokens，留出余量给模型输出
    let token_budget = (model_config.max_tokens as usize) * 8;
    app.chat_state.trim_by_token_budget(token_budget);

    // 收集多轮上下文（裁剪后的历史，不含当前消息）
    let chat_history = app.chat_state.to_rig_history();

    // 创建 channel 接收响应
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::ai::client::StreamMessage>();
    app.chat_rx = Some(rx);

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
            engine,
            help_mgr,
            external_dict_data,
        );
        crate::ai_log!("Agent 创建成功，开始流式请求");
        let _ =
            crate::ai::client::send_message_stream(&agent, &message_clone, chat_history, tx).await;
        crate::ai_log!("流式请求完成");
        ctx_clone.request_repaint();
    });

    app.chat_state.abort_handle = Some(handle.abort_handle());
}

/// 渲染设置标签页
fn render_settings_tab(ui: &mut egui::Ui, app: &mut DictApp, mut config: AiConfig) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        // 捕获可用宽度，所有区块都约束在此宽度内，避免内容溢出窗口外/重叠
        ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 4.0);
        ui.add_space(8.0);

        ui.add_space(8.0);

        // 当前激活模型 + 新增按钮（放在列表上方）
        ui.horizontal(|ui| {
            if let Some(active) = config.active_model() {
                ui.label(egui::RichText::new(format!("当前激活: {} - {} ({})", active.name, active.provider, active.model)).color(egui::Color32::from_rgb(100, 100, 100)));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕ 新增模型").clicked() {
                    app.show_model_edit_dialog = true;
                    app.editing_model_id = None;
                    app.editing_model_name = String::new();
                    app.editing_model_provider = Provider::default();
                    app.editing_model_api_url = "https://api.deepseek.com".to_string();
                    app.editing_model_model = "deepseek-chat".to_string();
                    app.editing_model_api_key = String::new();
                    app.editing_model_temperature = 0.7;
                    app.editing_model_max_tokens = 2048;
                    app.chat_test_response.clear();
                    app.chat_test_in_progress = false;
                }
            });
        });

        // 模型操作结果提示
        if !app.chat_save_response.is_empty() {
            let color = if app.chat_save_response.starts_with("❌") {
                egui::Color32::from_rgb(220, 50, 50)
            } else {
                egui::Color32::from_rgb(34, 150, 80)
            };
            ui.label(egui::RichText::new(&app.chat_save_response).color(color));
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
                let model_ids: Vec<String> = config.models.iter().map(|m| m.id.clone()).rev().collect();
                egui::ScrollArea::vertical()
                    .max_height(250.0)
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        for model_id in model_ids {
                            let Some(model) = config.find_model(&model_id).cloned() else {
                                continue;
                            };
                            let is_active = config.active_model_id.as_deref() == Some(&model.id);
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
                                    if !is_active && ui.small_button("使用此模型").clicked() {
                                        if !app.chat_state.is_generating {
                                            config.active_model_id = Some(model.id.clone());
                                            let _ = config.save();
                                        } else {
                                            app.chat_save_response = "⚠️ 当前正在生成，请完成后再切换".to_string();
                                        }
                                    }
                                    if ui.small_button("✎").clicked() {
                                        app.show_model_edit_dialog = true;
                                        app.editing_model_id = Some(model.id.clone());
                                        app.editing_model_name = model.name.clone();
                                        app.editing_model_provider = model.provider.clone();
                                        app.editing_model_api_url = model.api_url.clone();
                                        app.editing_model_model = model.model.clone();
                                        app.editing_model_api_key = String::new();
                                        app.editing_model_temperature = model.temperature;
                                        app.editing_model_max_tokens = model.max_tokens;
                                        app.chat_test_response.clear();
                                        app.chat_test_in_progress = false;
                                    }
                                    if ui.small_button("🗑").clicked() {
                                        app.confirm_delete_model_id = Some(model.id.clone());
                                        app.confirm_delete_model_name = model.name.clone();
                                    }
                                    if ui.small_button("📋").clicked()
                                        && let Some(_new_model) = config.duplicate_model(&model.id) {
                                            config.active_model_id = Some(_new_model.id.clone());
                                            let _ = config.save();
                                        }
                                });
                            });

                            // 第二行：提供商 | 模型名
                            ui.horizontal_wrapped(|ui| {
                                ui.small(model.provider.to_string());
                                ui.small("|");
                                ui.small(&model.model);
                            });
                            ui.separator();
                        }
                    });
            }
        });

        ui.add_space(12.0);

        // 高级选项
        ui.group(|ui| {
            ui.label("全局设置");
            ui.add_space(4.0);

            // 外部词典工具开关（控制 AI 是否可查询本地 Rime 词典）
            ui.add(
                egui::Checkbox::new(
                    &mut config.enable_external_dict_tool,
                    "启用外部词典工具",
                ),
            )
            .on_hover_text("启用后 AI 可以搜索您加载的外部 Rime 词典内容。查询时会发送本地词库片段到 AI 服务商");
            ui.horizontal_wrapped(|ui| {
                ui.small("（查询会上传本地词库片段，见");
                if ui
                    .link(egui::RichText::new("隐私声明").text_style(egui::TextStyle::Small))
                    .clicked()
                {
                    app.show_privacy_dialog = true;
                }
                ui.small("）");
            });

            ui.add_space(4.0);

            egui::Grid::new("ai_global_settings_grid")
                .spacing(egui::vec2(8.0, 8.0))
                .show(ui, |ui| {
                    ui.label("上下文保留轮数:");
                    ui.add(egui::Slider::new(&mut config.history_rounds, 4..=30));
                    ui.end_row();

                    ui.label("最大工具调用轮次:");
                    ui.add(egui::Slider::new(&mut config.max_tool_turns, 1..=30));
                    ui.end_row();

                    ui.label("");
                    ui.label("AI 在一次对话中最多连续调用工具的轮次，防止无限循环消耗额度");
                    ui.end_row();
                });

            ui.add_space(4.0);

            // 对话历史持久化开关
            ui.add(
                egui::Checkbox::new(
                    &mut config.persist_conversations,
                    "记录对话历史",
                ),
            )
            .on_hover_text("开启后，对话内容会在每次 AI 回复完成后保存到本地，并可在工具栏中切换和删除历史对话。关闭后不再记录新对话，但不会删除已有记录");

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("💾 保存全局设置").clicked() {
                    match config.save() {
                        Ok(()) => app.chat_global_save_response = "✅ 全局设置已保存".to_string(),
                        Err(e) => app.chat_global_save_response = format!("❌ 保存失败: {e}"),
                    }
                }
                if !app.chat_global_save_response.is_empty() {
                    let color = if app.chat_global_save_response.starts_with("❌") {
                        egui::Color32::from_rgb(220, 50, 50)
                    } else {
                        egui::Color32::from_rgb(34, 150, 80)
                    };
                    ui.label(egui::RichText::new(&app.chat_global_save_response).color(color));
                }
            });
            });

        ui.add_space(4.0);

        // 确认删除模型弹窗（状态驱动，跨帧保持）
        if let Some(delete_id) = app.confirm_delete_model_id.clone() {
            let name = app.confirm_delete_model_name.clone();
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
                app.confirm_delete_model_id = None;
                app.confirm_delete_model_name.clear();
            }
        }

        // 渲染模型编辑弹窗
        render_model_edit_dialog(ui, app, &mut config);
    });
}

/// 渲染新增/编辑模型弹窗
fn render_model_edit_dialog(ui: &mut egui::Ui, app: &mut DictApp, config: &mut AiConfig) {
    if !app.show_model_edit_dialog {
        return;
    }

    let is_new = app.editing_model_id.is_none();
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
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(180)))
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
                    egui::TextEdit::singleline(&mut app.editing_model_name)
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
                    .selected_text(app.editing_model_provider.to_string())
                    .show_ui(ui, |ui| {
                        let providers = [
                            Provider::DeepSeek,
                            Provider::Zhipu,
                            Provider::Moonshot,
                            Provider::Qwen,
                            Provider::Yi,
                            Provider::Custom,
                        ];
                        for p in &providers {
                            if ui
                                .selectable_value(
                                    &mut app.editing_model_provider,
                                    p.clone(),
                                    p.to_string(),
                                )
                                .clicked()
                            {
                                // 根据提供商设置默认 API URL 和模型名
                                match p {
                                    Provider::DeepSeek => {
                                        app.editing_model_api_url =
                                            "https://api.deepseek.com".to_string();
                                        app.editing_model_model = "deepseek-chat".to_string();
                                    }
                                    Provider::Zhipu => {
                                        app.editing_model_api_url =
                                            "https://open.bigmodel.cn/api/paas/v4".to_string();
                                        app.editing_model_model = "glm-4-flash".to_string();
                                    }
                                    Provider::Moonshot => {
                                        app.editing_model_api_url =
                                            "https://api.moonshot.cn/v1".to_string();
                                        app.editing_model_model = "moonshot-v1-8k".to_string();
                                    }
                                    Provider::Qwen => {
                                        app.editing_model_api_url =
                                            "https://dashscope.aliyuncs.com/compatible-mode/v1"
                                                .to_string();
                                        app.editing_model_model = "qwen-turbo".to_string();
                                    }
                                    Provider::Yi => {
                                        app.editing_model_api_url =
                                            "https://api.lingyiwanwu.com/v1".to_string();
                                        app.editing_model_model = "yi-large".to_string();
                                    }
                                    Provider::Custom => {
                                        if app.editing_model_api_url.is_empty() {
                                            app.editing_model_api_url =
                                                "https://your-api-endpoint.com/v1".to_string();
                                        }
                                    }
                                }
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
                    egui::TextEdit::singleline(&mut app.editing_model_api_url)
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
                    egui::TextEdit::singleline(&mut app.editing_model_model)
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
                    egui::TextEdit::singleline(&mut app.editing_model_api_key)
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
            let mut temp_pct = (app.editing_model_temperature * 100.0).round() as i32;
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
                app.editing_model_temperature = temp_pct as f32 / 100.0;
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
                    egui::Slider::new(&mut app.editing_model_max_tokens, 100..=8192).step_by(100.0),
                );
            });

            ui.add_space(8.0);

            // 测试连接
            ui.horizontal(|ui| {
                let can_test = !app.editing_model_api_url.trim().is_empty()
                    && !app.editing_model_model.trim().is_empty()
                    && (!app.editing_model_api_key.trim().is_empty()
                        || app.editing_model_id.is_some())
                    && !app.chat_test_in_progress;
                let test_label = if app.chat_test_in_progress {
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
                        app.editing_model_name.clone(),
                        app.editing_model_provider.clone(),
                        app.editing_model_api_url.trim().to_string(),
                        app.editing_model_model.trim().to_string(),
                        app.editing_model_temperature,
                        app.editing_model_max_tokens,
                    );
                    // 获取 API Key：优先用输入框，为空则从已有模型读取
                    let api_key_result: Result<String, String> =
                        if !app.editing_model_api_key.trim().is_empty() {
                            Ok(app.editing_model_api_key.trim().to_string())
                        } else if let Some(edit_id) = &app.editing_model_id {
                            match config.find_model(edit_id) {
                                Some(m) => m.get_api_key(),
                                None => Err("未找到原模型".to_string()),
                            }
                        } else {
                            Err("API Key 不能为空".to_string())
                        };

                    match api_key_result {
                        Ok(api_key) => {
                            if let Some(runtime) = app.tokio_runtime.as_ref() {
                                let (tx, rx) = tokio::sync::oneshot::channel::<String>();
                                app.chat_test_rx = Some(rx);
                                app.chat_test_in_progress = true;
                                app.chat_test_response.clear();
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
                                app.chat_test_response = "❌ 运行时未初始化".to_string();
                            }
                        }
                        Err(e) => {
                            app.chat_test_response = format!("❌ {e}");
                        }
                    }
                }

                if !app.chat_test_response.is_empty() {
                    let color = if app.chat_test_response.starts_with("❌") {
                        egui::Color32::from_rgb(220, 50, 50)
                    } else {
                        egui::Color32::from_rgb(34, 150, 80)
                    };
                    ui.add(
                        egui::Label::new(egui::RichText::new(&app.chat_test_response).color(color))
                            .wrap(),
                    );
                }
            });

            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if ui.button("取消").clicked() {
                    app.show_model_edit_dialog = false;
                    app.chat_test_response.clear();
                    app.chat_test_in_progress = false;
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
                    if app.editing_model_name.trim().is_empty() {
                        error = Some("名称不能为空");
                    } else if app.editing_model_api_url.trim().is_empty() {
                        error = Some("API URL 不能为空");
                    } else if !app.editing_model_api_url.starts_with("http://")
                        && !app.editing_model_api_url.starts_with("https://")
                    {
                        error = Some("API URL 必须以 http:// 或 https:// 开头");
                    } else if app.editing_model_model.trim().is_empty() {
                        error = Some("模型名不能为空");
                    } else if is_new && app.editing_model_api_key.trim().is_empty() {
                        error = Some("API Key 不能为空");
                    }

                    if let Some(e) = error {
                        app.chat_save_response = format!("❌ {e}");
                        return;
                    }

                    let name = app.editing_model_name.trim().to_string();
                    let provider = app.editing_model_provider.clone();
                    let api_url = app.editing_model_api_url.trim().to_string();
                    let model_name = app.editing_model_model.trim().to_string();
                    let temperature = app.editing_model_temperature;
                    let max_tokens = app.editing_model_max_tokens;
                    let api_key = app.editing_model_api_key.trim().to_string();

                    match &app.editing_model_id {
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
                            config.models.push(new_model);
                            config.active_model_id = Some(config.models.last().unwrap().id.clone());
                        }
                        Some(edit_id) => {
                            // 编辑
                            if let Some(model) = config.find_model_mut(edit_id) {
                                model.name = name;
                                model.provider = provider;
                                model.api_url = api_url;
                                model.model = model_name.clone();
                                model.temperature = temperature;
                                model.max_tokens = max_tokens;
                                if !api_key.is_empty() {
                                    model.api_key = api_key;
                                } else if model.api_key.is_empty() {
                                    app.chat_save_response =
                                        "❌ 该模型尚未配置 API Key，请输入 API Key".to_string();
                                    return;
                                }
                            }
                        }
                    }

                    config.validate_and_fix();
                    let _ = config.save();
                    app.show_model_edit_dialog = false;
                    app.chat_test_response.clear();
                    app.chat_test_in_progress = false;
                    app.chat_save_response.clear();
                }
            });
        });
}
