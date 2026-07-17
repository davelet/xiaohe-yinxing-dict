use crate::DictApp;
use crate::ai::config::{AiConfig, Provider};
use eframe::egui;

/// 渲染 AI 对话子窗口内容
pub fn render_chat_viewport(ui: &mut egui::Ui, app: &mut DictApp) {
    // 子窗口是独立 viewport，默认是深色主题，需每帧直接覆盖其样式为浅色，
    // 否则设置页等会显示为深色
    {
        let style = ui.style_mut();
        style.visuals = egui::Visuals::light();
        crate::ui::styles::DictViewStyle::apply(style);
    }

    // 处理原生标题栏关闭按钮：用户点关闭时隐藏 AI 助手
    if ui.input(|i| i.viewport().close_requested()) {
        app.show_chat_viewport = false;
        app.current_view = crate::app::ViewMode::Dict;
        return;
    }

    // 轮询后台 AI 响应（非阻塞）
    let ctx = ui.ctx().clone();
    poll_chat_stream(app, &ctx);
    poll_chat_test(app);

    // 加载 AI 配置检查是否已配置 API Key
    let ai_config = AiConfig::load();
    let has_api_key = ai_config.get_api_key().is_ok();

    // 检查是否需要显示隐私提示弹窗
    let mandatory_privacy =
        !ai_config.privacy_acknowledged && app.chat_tab == crate::app::ChatTab::Conversation;
    if mandatory_privacy || app.show_privacy_dialog {
        render_privacy_dialog(ui, app);
        // 仅“首次强制确认”时阻断底层内容；手动触发（设置页“见隐私声明”）
        // 时作为浮层叠加在设置页之上，不拦截。
        if mandatory_privacy {
            return;
        }
    }

    // 使用 CentralPanel 填充窗口背景，避免子窗口原生背景透出显示为黑色
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
        });
        ui.separator();

        // 根据当前 tab 渲染内容
        match app.chat_tab {
            crate::app::ChatTab::Conversation => {
                if has_api_key {
                    render_conversation_tab(ui, app);
                } else {
                    render_empty_state(ui, app);
                }
            }
            crate::app::ChatTab::Settings => {
                render_settings_tab(ui, app);
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

            ui.horizontal(|ui| {
                if ui.button("我已了解，开始使用").clicked() {
                    let mut config = AiConfig::load();
                    config.privacy_acknowledged = true;
                    config.save().ok();
                    app.show_privacy_dialog = false;
                }

                ui.add_space(8.0);

                if ui.button("返回设置").clicked() {
                    app.chat_tab = crate::app::ChatTab::Settings;
                    app.show_privacy_dialog = false;
                }
            });
        });
}

/// 渲染未配置 API Key 的空状态
fn render_empty_state(ui: &mut egui::Ui, app: &mut DictApp) {
    ui.vertical_centered(|ui| {
        ui.add_space(80.0);
        ui.heading("✨ 小鹤音形 AI 助手");
        ui.add_space(16.0);
        ui.label("使用 AI 对话功能前，请先配置 API Key。");
        ui.add_space(8.0);
        ui.label(
            "支持深度求索、智谱、通义千问等，也可在“自定义”中绑定任意兼容 OpenAI 接口协议的模型。",
        );
        ui.add_space(24.0);
        if ui.button("前往设置").clicked() {
            app.chat_tab = crate::app::ChatTab::Settings;
        }
    });
}

/// 渲染对话标签页
fn render_conversation_tab(ui: &mut egui::Ui, app: &mut DictApp) {
    // 对话工具栏
    render_conversation_toolbar(ui, app);

    // 消息列表区域
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(8.0);

            // 渲染消息
            let msg_count = app.chat_state.messages.len();
            let ai_indices: Vec<(usize, bool)> = (0..msg_count)
                .filter(|&i| app.chat_state.messages[i].role == crate::ai::chat::Role::AI)
                .map(|i| (i, i == msg_count - 1))
                .collect();

            for msg in app.chat_state.messages.iter() {
                if msg.role == crate::ai::chat::Role::User {
                    render_user_message(ui, &msg.content);
                    ui.add_space(4.0);
                }
            }

            // AI messages rendered in second pass for mutable borrow
            for &(idx, is_last) in &ai_indices {
                render_ai_message(ui, app, idx, is_last);
                ui.add_space(4.0);
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
                ui.label(content);
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
                    });
                }
            });
    });
}

/// 渲染输入区域
fn render_input_area(ui: &mut egui::Ui, app: &mut DictApp) {
    let ctx = ui.ctx().clone();
    ui.horizontal(|ui| {
        // 输入框
        let input_width = ui.available_width() - 80.0;
        let response = ui.add_sized(
            [input_width, 36.0],
            egui::TextEdit::singleline(&mut app.chat_input)
                .hint_text("输入消息...")
                .font(egui::TextStyle::Body),
        );

        // 回车发送
        if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            send_message(app, &ctx);
        }

        // 发送/停止按钮
        if app.chat_state.is_generating {
            if ui.button("⏹ 停止").clicked() {
                app.chat_state.interrupt_generation();
            }
        } else {
            if ui.button("发送").clicked() && !app.chat_input.trim().is_empty() {
                send_message(app, &ctx);
            }
        }
    });
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
            Ok(crate::ai::client::StreamMessage::Complete) => {
                app.chat_state.finish_generation();
                save_current_conversation(app);
                app.chat_rx = None;
                ctx.request_repaint();
                break;
            }
            Ok(crate::ai::client::StreamMessage::Error(e)) => {
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

/// 保存当前对话到磁盘（自动生成 ID 和标题）
fn save_current_conversation(app: &mut DictApp) {
    if app.chat_state.messages.is_empty() {
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

/// 渲染对话工具栏（新建、导出、清空历史）
fn render_conversation_toolbar(ui: &mut egui::Ui, app: &mut DictApp) {
    ui.horizontal(|ui| {
        if ui.button("📄 新建对话").clicked() {
            // 保存当前对话再清空
            save_current_conversation(app);
            app.chat_state.clear();
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

        if ui.button("🗑 清空历史").clicked() {
            let _ = app.conversation_store.clear_all();
            app.chat_state.clear();
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

    let ai_config = crate::ai::config::AiConfig::load();
    let runtime = app.tokio_runtime.as_ref();

    if let (Ok(api_key), Some(runtime)) = (ai_config.get_api_key(), runtime) {
        let engine = app.engine.clone();
        let help_mgr = app.help_manager.clone();

        // 按配置的轮数裁剪历史
        app.chat_state
            .trim_history(ai_config.history_rounds as usize);

        // 创建 channel 接收响应
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<crate::ai::client::StreamMessage>();
        app.chat_rx = Some(rx);

        // 在后台 tokio 任务中执行整个异步流程
        let ctx_clone = ctx.clone();
        let config_clone = ai_config.clone();
        let message_clone = message.clone();

        let handle = runtime.spawn(async move {
            let client = match crate::ai::client::create_client(&config_clone, &api_key) {
                Ok(c) => c,
                Err(e) => {
                    let _ = tx.send(crate::ai::client::StreamMessage::Error(e));
                    ctx_clone.request_repaint();
                    return;
                }
            };
            let agent = crate::ai::client::build_agent(client, &config_clone, engine, help_mgr);
            let _ = crate::ai::client::send_message_stream(&agent, &message_clone, tx).await;
            ctx_clone.request_repaint();
        });

        app.chat_state.abort_handle = Some(handle.abort_handle());
    } else {
        // 没有 API Key 或 runtime，显示错误
        app.chat_state
            .add_ai_message("无法发送消息：请先配置 API Key".to_string());
        app.chat_state.is_generating = false;
    }
}

/// 渲染设置标签页
fn render_settings_tab(ui: &mut egui::Ui, app: &mut DictApp) {
    // 仅首次从配置文件加载到草稿，之后所有控件绑定到持久草稿，
    // 避免每帧从磁盘重载导致修改（提供商/温度/token/轮数等）被清空
    if !app.chat_settings_init {
        let loaded = AiConfig::load();
        app.chat_api_key_draft = loaded.get_api_key().unwrap_or_default();
        app.chat_settings_draft = loaded;
        app.chat_settings_init = true;
    }
    let config = &mut app.chat_settings_draft;

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 4.0);
        ui.add_space(8.0);

        // 输入框统一样式：灰色描边 + 圆角，与其它地方一致
        let input_frame = egui::Frame::group(ui.style())
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(180)))
            .corner_radius(4.0)
            .inner_margin(egui::Margin::symmetric(4, 2));

        // 字段名固定宽度并居右对齐
        // API 配置区的标签较短，收窄列宽以消除左侧过多留白；
        // 高级区的“上下文保留轮数:”较长，单独给一个能容纳的宽度。
        let label_w_api = 66.0;
        let label_w_adv = 116.0;
        let field_label = |ui: &mut egui::Ui, text: &str, w: f32, left: bool| {
            ui.allocate_ui_with_layout(
                egui::vec2(w, 0.0),
                if left {
                    egui::Layout::left_to_right(egui::Align::Center)
                } else {
                    egui::Layout::right_to_left(egui::Align::Center)
                },
                |ui| {
                    ui.label(text);
                },
            );
        };

        ui.heading("AI 助手设置");
        ui.add_space(12.0);

        // API 配置区块
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label("API 配置");
            ui.add_space(4.0);

            // 提供商选择
            ui.horizontal(|ui| {
                field_label(ui, "提供商:", label_w_api, false);
                egui::ComboBox::from_id_salt("provider_combo")
                    .width(ui.available_width())
                    .selected_text(config.provider.to_string())
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
                                .selectable_value(&mut config.provider, p.clone(), p.to_string())
                                .clicked()
                            {
                                // 根据提供商设置默认 API URL 和模型名
                                match p {
                                    Provider::DeepSeek => {
                                        config.api_url = "https://api.deepseek.com".to_string();
                                        config.model = "deepseek-chat".to_string();
                                    }
                                    Provider::Zhipu => {
                                        config.api_url =
                                            "https://open.bigmodel.cn/api/paas/v4".to_string();
                                        config.model = "glm-4-flash".to_string();
                                    }
                                    Provider::Moonshot => {
                                        config.api_url = "https://api.moonshot.cn/v1".to_string();
                                        config.model = "moonshot-v1-8k".to_string();
                                    }
                                    Provider::Qwen => {
                                        config.api_url =
                                            "https://dashscope.aliyuncs.com/compatible-mode/v1"
                                                .to_string();
                                        config.model = "qwen-turbo".to_string();
                                    }
                                    Provider::Yi => {
                                        config.api_url =
                                            "https://api.lingyiwanwu.com/v1".to_string();
                                        config.model = "yi-large".to_string();
                                    }
                                    Provider::Custom => {
                                        // 切到自定义时清空 URL 和模型，避免残留
                                        config.api_url.clear();
                                        config.model.clear();
                                    }
                                }
                            }
                        }
                    });
            });

            ui.add_space(4.0);

            // API Key 输入
            ui.horizontal(|ui| {
                field_label(ui, "API Key:", label_w_api, false);
                let _key_response = ui.add(
                    egui::TextEdit::singleline(&mut app.chat_api_key_draft)
                        .password(true)
                        .desired_width(ui.available_width())
                        .frame(input_frame),
                );
            });

            ui.add_space(4.0);

            // API URL（仅 Custom 提供商显示）
            // API URL：内置与自定义提供商都显示，方便在内置默认值有误时手动修正
            ui.horizontal(|ui| {
                field_label(ui, "API URL:", label_w_api, false);
                ui.add(
                    egui::TextEdit::singleline(&mut config.api_url)
                        .desired_width(ui.available_width())
                        .frame(input_frame),
                );
            });
            ui.add_space(8.0);

            // 模型选择
            ui.horizontal(|ui| {
                field_label(ui, "模型:", label_w_api, false);
                ui.add(
                    egui::TextEdit::singleline(&mut config.model)
                        .desired_width(ui.available_width())
                        .frame(input_frame),
                );
            });

            ui.add_space(4.0);

            // 温度：内部仍按 0.0~2.0 的浮点保存，UI 以整数百分比展示（100% = 1.0）
            let mut temp_pct = (config.temperature * 100.0).round() as i32;
            ui.horizontal(|ui| {
                field_label(ui, "温度:", label_w_api, false);
                ui.add(egui::Slider::new(&mut temp_pct, 0..=200).step_by(1.0));
                ui.label("%");
                config.temperature = temp_pct as f32 / 100.0;
            });

            ui.add_space(4.0);

            // Max Tokens
            ui.horizontal(|ui| {
                field_label(ui, "词元上限:", label_w_api, false);
                ui.add(egui::Slider::new(&mut config.max_tokens, 100..=8192).step_by(100.0));
            });
        });

        ui.add_space(10.0);

        // 高级选项
        ui.group(|ui| {
            ui.set_min_width(ui.available_width());
            ui.label("高级选项");
            ui.add_space(4.0);

            // 外部词典工具尚未实现，暂时禁用
            ui.add_enabled(
                false,
                egui::Checkbox::new(
                    &mut config.enable_external_dict_tool,
                    "启用外部词典工具（即将推出）",
                ),
            )
            .on_hover_text("该工具尚未实现，敬请期待后续版本");
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

            ui.horizontal(|ui| {
                field_label(ui, "上下文保留轮数:", label_w_adv, true);
                ui.add(egui::Slider::new(&mut config.history_rounds, 4..=30));
            });
        });

        ui.add_space(10.0);

        // 保存按钮 + 清除 API Key（等宽等高并排）
        ui.horizontal(|ui| {
            let btn_w = ui.available_width() / 2.0;
            if ui
                .add_sized([btn_w, 28.0], egui::Button::new("保存设置"))
                .clicked()
            {
                // URL 校验
                let url_valid =
                    config.api_url.starts_with("http://") || config.api_url.starts_with("https://");
                if !url_valid {
                    app.chat_save_response =
                        "❌ API URL 必须以 http:// 或 https:// 开头".to_string();
                } else if config.api_url.contains(' ') {
                    app.chat_save_response = "❌ API URL 不能包含空格".to_string();
                } else {
                    // 保存 API Key 到钥匙串
                    let mut msg = String::new();
                    if !app.chat_api_key_draft.is_empty()
                        && let Err(e) = config.set_api_key(&app.chat_api_key_draft)
                    {
                        msg = format!("❌ 保存 API Key 失败: {e}");
                    }
                    // 保存配置（仅当 Key 保存未失败时继续）
                    if msg.is_empty() {
                        match config.save() {
                            Ok(()) => msg = "✅ 设置已保存".to_string(),
                            Err(e) => msg = format!("❌ 保存配置失败: {e}"),
                        }
                    }
                    app.chat_save_response = msg;
                }
            }

            if ui
                .add_sized([btn_w, 28.0], egui::Button::new("🗑 清除 API Key"))
                .clicked()
            {
                if let Err(e) = config.delete_api_key() {
                    app.chat_save_response = format!("❌ 清除失败: {e}");
                } else {
                    app.chat_api_key_draft.clear();
                    app.chat_save_response = "✅ API Key 已清除".to_string();
                }
            }
        });

        // 显示保存/清除结果
        if !app.chat_save_response.is_empty() {
            let color = if app.chat_save_response.starts_with("❌") {
                egui::Color32::from_rgb(220, 50, 50)
            } else {
                egui::Color32::from_rgb(34, 150, 80)
            };
            ui.label(egui::RichText::new(&app.chat_save_response).color(color));
        }

        ui.add_space(4.0);

        // 测试连接按钮（非阻塞，后台执行，进行中视觉禁用）
        let test_label = if app.chat_test_in_progress {
            "🔗 测试中..."
        } else {
            "🔗 测试连接"
        };
        if ui
            .add_enabled(!app.chat_test_in_progress, egui::Button::new(test_label))
            .clicked()
        {
            if app.chat_api_key_draft.is_empty() {
                app.chat_test_response = "❌ 请先输入 API Key".to_string();
            } else {
                let test_config = config.clone();
                let api_key = app.chat_api_key_draft.clone();
                let runtime = app.tokio_runtime.as_ref();
                if let Some(runtime) = runtime {
                    let (tx, rx) = tokio::sync::oneshot::channel::<String>();
                    app.chat_test_rx = Some(rx);
                    app.chat_test_in_progress = true;
                    app.chat_test_response.clear();

                    let engine = app.engine.clone();
                    let help_mgr_test = app.help_manager.clone();
                    let ctx_clone = ui.ctx().clone();

                    runtime.spawn(async move {
                        let result = async {
                            let client = crate::ai::client::create_client(&test_config, &api_key)?;
                            let agent = crate::ai::client::build_agent(
                                client,
                                &test_config,
                                engine,
                                help_mgr_test,
                            );
                            crate::ai::client::send_message(&agent, "hi").await
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
                    app.chat_test_response = "❌ Tokio 运行时未初始化".to_string();
                }
            }
        }

        // 显示测试连接结果
        if !app.chat_test_response.is_empty() {
            let color = if app.chat_test_response.starts_with("❌") {
                egui::Color32::from_rgb(220, 50, 50)
            } else {
                egui::Color32::from_rgb(34, 150, 80)
            };
            ui.label(egui::RichText::new(&app.chat_test_response).color(color));
        }
    });
}
