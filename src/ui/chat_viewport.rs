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

    // 加载 AI 配置检查是否已配置 API Key
    let ai_config = AiConfig::load();
    let has_api_key = ai_config.get_api_key().is_ok();

    // 检查是否需要显示隐私提示弹窗
    if !ai_config.privacy_acknowledged && app.chat_tab == crate::app::ChatTab::Conversation {
        render_privacy_dialog(ui, app);
        return;
    }

    // 使用 CentralPanel 填充窗口背景，避免子窗口原生背景透出显示为黑色
    egui::CentralPanel::default().show_inside(ui, |ui| {
        // 顶部 tab 栏
        ui.horizontal(|ui| {
            let conv_btn = ui.selectable_label(app.chat_tab == crate::app::ChatTab::Conversation, "AI 对话");
            let settings_btn = ui.selectable_label(app.chat_tab == crate::app::ChatTab::Settings, "AI 设置");

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
                    ui.label("1. 您的对话内容将发送至所选的 AI 服务提供商（如 OpenAI、DeepSeek 等）");
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
                }

                ui.add_space(8.0);

                if ui.button("返回设置").clicked() {
                    app.chat_tab = crate::app::ChatTab::Settings;
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
        ui.label("支持 OpenAI、DeepSeek、Ollama 等多种模型。");
        ui.add_space(24.0);
        if ui.button("前往设置").clicked() {
            app.chat_tab = crate::app::ChatTab::Settings;
        }
    });
}

/// 渲染对话标签页
fn render_conversation_tab(ui: &mut egui::Ui, app: &mut DictApp) {
    // 消息列表区域
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add_space(8.0);

            // 渲染消息
            for msg in &app.chat_state.messages {
                match msg.role {
                    crate::ai::chat::Role::User => {
                        render_user_message(ui, &msg.content);
                    }
                    crate::ai::chat::Role::AI => {
                        render_ai_message(ui, &msg.content, &msg.tool_calls, msg.is_interrupted);
                    }
                    _ => {}
                }
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
                            if let crate::ai::chat::StreamState::Generating(content) = &app.chat_state.stream_state {
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
fn render_ai_message(
    ui: &mut egui::Ui,
    content: &str,
    tool_calls: &[crate::ai::chat::ToolCallRecord],
    is_interrupted: bool,
) {
    ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(255, 255, 255))
            .corner_radius(8.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.set_max_width(350.0);

                // 渲染内容（简单文本，后续可加 Markdown）
                if is_interrupted {
                    ui.label(format!("{} [已中断]", content));
                } else {
                    ui.label(content);
                }

                // 渲染工具调用结果
                if !tool_calls.is_empty() {
                    ui.add_space(4.0);
                    ui.collapsing("工具调用详情", |ui| {
                        for tc in tool_calls {
                            ui.label(format!("🔧 {}: {}", tc.tool_name, tc.arguments));
                            ui.small(&tc.result);
                            ui.add_space(4.0);
                        }
                    });
                }
            });
    });
}

/// 渲染输入区域
fn render_input_area(ui: &mut egui::Ui, app: &mut DictApp) {
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
            send_message(app);
        }

        // 发送/停止按钮
        if app.chat_state.is_generating {
            if ui.button("⏹ 停止").clicked() {
                app.chat_state.interrupt_generation();
            }
        } else {
            if ui.button("发送").clicked() && !app.chat_input.trim().is_empty() {
                send_message(app);
            }
        }
    });
}

/// 发送消息
fn send_message(app: &mut DictApp) {
    let message = app.chat_input.trim().to_string();
    if message.is_empty() || app.chat_state.is_generating {
        return;
    }

    // 添加用户消息
    app.chat_state.add_user_message(message.clone());
    app.chat_input.clear();
    app.chat_state.start_generation();

    // 获取必要的引用
    let ai_config = crate::ai::config::AiConfig::load();
    let runtime = app.tokio_runtime.as_ref();

    if let (Ok(api_key), Some(runtime)) = (ai_config.get_api_key(), runtime) {
        let engine = app.engine.clone();
        let chat_state = &mut app.chat_state;

        // 按配置的轮数裁剪历史
        chat_state.trim_history(ai_config.history_rounds as usize);

        // 创建 channel 接收流式响应
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<crate::ai::client::StreamMessage>();

        // 在 tokio runtime 中执行异步请求
        let result = runtime.block_on(async {
            let client = crate::ai::client::create_client(&ai_config, &api_key)?;
            let agent = crate::ai::client::build_agent(client, &ai_config, engine);

            // 启动流式请求任务
            let handle = tokio::spawn(async move {
                crate::ai::client::send_message_stream(&agent, &message, tx).await
            });

            // 存储 abort handle 用于中断
            Ok::<_, String>(handle)
        });

        match result {
            Ok(handle) => {
                // 存储 abort handle
                chat_state.abort_handle = Some(handle.abort_handle());

                // 同步收集所有流式响应
                let mut full_response = String::new();
                while let Some(msg) = rx.blocking_recv() {
                    match msg {
                        crate::ai::client::StreamMessage::Text(text) => {
                            full_response = text;
                            // 更新流式状态
                            chat_state.update_stream(full_response.clone());
                        }
                        crate::ai::client::StreamMessage::Complete => {
                            break;
                        }
                        crate::ai::client::StreamMessage::Error(e) => {
                            chat_state.add_ai_message(format!("请求失败: {e}"));
                            chat_state.is_generating = false;
                            return;
                        }
                    }
                }

                // 完成生成
                if !full_response.is_empty() {
                    chat_state.finish_generation();
                } else {
                    chat_state.is_generating = false;
                }
            }
            Err(e) => {
                chat_state.add_ai_message(format!("请求失败: {e}"));
                chat_state.is_generating = false;
            }
        }
    } else {
        // 没有 API Key 或 runtime，显示错误
        app.chat_state.add_ai_message("无法发送消息：请先配置 API Key".to_string());
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
    let mut api_key_changed = false;

    egui::ScrollArea::vertical().show(ui, |ui| {
        // 收紧行间距，避免字段之间距离过大
        ui.style_mut().spacing.item_spacing = egui::vec2(8.0, 4.0);
        ui.add_space(8.0);
        ui.heading("AI 助手设置");
        ui.add_space(12.0);

        // API 配置区块
        ui.group(|ui| {
            ui.label("API 配置");
            ui.add_space(4.0);

            // 提供商选择
            ui.horizontal(|ui| {
                ui.label("提供商:");
                egui::ComboBox::from_id_salt("provider_combo")
                    .selected_text(config.provider.to_string())
                    .show_ui(ui, |ui| {
                        let providers = [
                            Provider::OpenAI,
                            Provider::DeepSeek,
                            Provider::Ollama,
                            Provider::Anthropic,
                            Provider::Gemini,
                            Provider::Custom,
                        ];
                        for p in &providers {
                            if ui.selectable_value(&mut config.provider, p.clone(), p.to_string()).clicked() {
                                // 根据提供商设置默认 API URL
                                match p {
                                    Provider::OpenAI => config.api_url = "https://api.openai.com/v1".to_string(),
                                    Provider::DeepSeek => config.api_url = "https://api.deepseek.com".to_string(),
                                    Provider::Ollama => config.api_url = "http://localhost:11434".to_string(),
                                    Provider::Anthropic => config.api_url = "https://api.anthropic.com".to_string(),
                                    Provider::Gemini => config.api_url = "https://generativelanguage.googleapis.com".to_string(),
                                    _ => {}
                                }
                            }
                        }
                    });
            });

            ui.add_space(4.0);

            // API Key 输入
            ui.horizontal(|ui| {
                ui.label("API Key:");
                let key_frame = egui::Frame::group(ui.style())
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_gray(180)))
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(4, 2));
                let key_response = ui.add(
                    egui::TextEdit::singleline(&mut app.chat_api_key_draft)
                        .password(true)
                        .desired_width(200.0)
                        .frame(key_frame),
                );
                if key_response.changed() {
                    api_key_changed = true;
                }
            });

            ui.add_space(4.0);

            // API URL（仅 Custom 提供商显示）
            if config.provider == Provider::Custom {
                ui.horizontal(|ui| {
                    ui.label("API URL:");
                    ui.text_edit_singleline(&mut config.api_url);
                });
                ui.add_space(8.0);
            }

            // 模型选择
            ui.horizontal(|ui| {
                ui.label("模型:");
                ui.text_edit_singleline(&mut config.model);
            });

            ui.add_space(4.0);

            // 温度
            ui.horizontal(|ui| {
                ui.label("温度:");
                ui.add(egui::Slider::new(&mut config.temperature, 0.0..=2.0).step_by(0.1));
            });

            ui.add_space(4.0);

            // Max Tokens
            ui.horizontal(|ui| {
                ui.label("Max Tokens:");
                ui.add(egui::Slider::new(&mut config.max_tokens, 100..=8192).step_by(100.0));
            });
        });

        ui.add_space(10.0);

        // 高级选项
        ui.group(|ui| {
            ui.label("高级选项");
            ui.add_space(4.0);

            ui.checkbox(&mut config.enable_external_dict_tool, "启用外部词典工具");
            ui.small("（查询会上传本地词库片段，见隐私说明）");

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("上下文保留轮数:");
                ui.add(egui::Slider::new(&mut config.history_rounds, 4..=30));
            });
        });

        ui.add_space(10.0);

        // 快速配置
        ui.group(|ui| {
            ui.label("快速配置");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("OpenAI").clicked() {
                    config.provider = Provider::OpenAI;
                    config.api_url = "https://api.openai.com/v1".to_string();
                    config.model = "gpt-4o-mini".to_string();
                }
                if ui.button("DeepSeek").clicked() {
                    config.provider = Provider::DeepSeek;
                    config.api_url = "https://api.deepseek.com".to_string();
                    config.model = "deepseek-chat".to_string();
                }
                if ui.button("Ollama 本地").clicked() {
                    config.provider = Provider::Ollama;
                    config.api_url = "http://localhost:11434".to_string();
                    config.model = "qwen2.5:latest".to_string();
                }
            });
        });

        ui.add_space(10.0);

        // 保存按钮
        if ui.button("保存设置").clicked() {
            // 保存 API Key 到钥匙串
            if api_key_changed && !app.chat_api_key_draft.is_empty() {
                if let Err(e) = config.set_api_key(&app.chat_api_key_draft) {
                    eprintln!("保存 API Key 失败: {e}");
                }
            }
            // 保存配置
            if let Err(e) = config.save() {
                eprintln!("保存配置失败: {e}");
            }
        }

        ui.add_space(4.0);

        // 测试连接按钮
        if ui.button("🔗 测试连接").clicked() {
            if app.chat_api_key_draft.is_empty() {
                app.chat_test_response = "❌ 请先输入 API Key".to_string();
            } else {
                // 保存当前配置用于测试
                let test_config = config.clone();
                test_config.set_api_key(&app.chat_api_key_draft).ok();

                let runtime = app.tokio_runtime.as_ref();
                if let Some(runtime) = runtime {
                    let result = runtime.block_on(async {
                        let client = crate::ai::client::create_client(&test_config, &app.chat_api_key_draft)?;
                        let engine = app.engine.clone();
                        let agent = crate::ai::client::build_agent(client, &test_config, engine);
                        crate::ai::client::send_message(&agent, "测试连接").await
                    });

                    match result {
                        Ok(_) => app.chat_test_response = "✅ 连接成功".to_string(),
                        Err(e) => app.chat_test_response = format!("❌ {e}"),
                    }
                } else {
                    app.chat_test_response = "❌ Tokio 运行时未初始化".to_string();
                }
            }
        }

        // 显示测试结果
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
