use crate::DictApp;
use eframe::egui;

mod add_word_dialog;
pub(crate) mod chat_state;
mod chat_viewport;
mod common;
mod help;
mod manager_view;
mod panel;
mod styles;
mod table;
mod update_toast;

pub(crate) use chat_state::ChatAction;
pub(crate) use update_toast::UpdateUiState;

impl eframe::App for DictApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Handle feedback timer
        if self.feedback_timer > 0.0 {
            let dt = ctx.input(|i| i.unstable_dt);
            self.feedback_timer -= dt;
            if self.feedback_timer <= 0.0 {
                self.copied_feedback = None;
            }
            ctx.request_repaint();
        }

        // Check for update info from background thread
        self.update_ui.poll_update_info(&ctx);

        // Show update dialog
        self.update_ui.render_dialog(&ctx);

        // About dialog
        if self.show_about_dialog {
            let window_response = egui::Window::new("关于小鹤音形词典")
                .collapsible(false)
                .resizable(false)
                .min_width(420.0)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(&ctx, |ui| {
                    ui.label("本软件是《小鹤音形》输入法的离线词典查询工具。");
                    ui.add_space(8.0);
                    ui.label("使用方法：");
                    ui.label("  • 搜索框输入汉字或编码可查询内置数据");
                    ui.label("  • 点击分类筛选下拉菜单可按分类过滤结果");
                    ui.label("  • 操作按钮可复制文字或编码到剪贴板");
                    ui.label("  • 按 → （方向右键）切换到输入法管理视图");
                    ui.label("  • 输入法管理视图可添加自定义新词，自动部署Rime立即生效");
                    ui.add_space(12.0);
                    let version = env!("CARGO_PKG_VERSION");
                    ui.label(format!("版本：v{}", version));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("有问题或建议？欢迎到 GitHub 提");
                        ui.hyperlink_to(
                            "Issue",
                            "https://github.com/davelet/xiaohe-yinxing-dict/issues/new",
                        );
                        ui.label("。");
                    });
                    ui.add_space(8.0);
                    if ui.button("关闭[Esc]").clicked() {
                        self.show_about_dialog = false;
                    }
                });

            if let Some(inner) = window_response
                && inner.response.clicked_elsewhere()
            {
                self.show_about_dialog = false;
            }
        }

        // F1 切换帮助文档
        if ui.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_help_panel = !self.show_help_panel;
            if self.show_help_panel && self.selected_help_chapter.is_none() {
                self.selected_help_chapter = Some("readme".to_string());
            }
        }

        // Cmd/Ctrl + , 切换 AI 助手
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Comma)) {
            self.chat.show_viewport = !self.chat.show_viewport;
            self.current_view = if self.chat.show_viewport {
                crate::types::ViewMode::Chat
            } else {
                crate::types::ViewMode::Dict
            };
            ctx.request_repaint();
        }

        // Esc 关闭关于弹窗（消费该 Esc 事件，避免同帧内搜索框也响应清空）
        if self.show_about_dialog && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.show_about_dialog = false;
            ctx.input_mut(|input| {
                input.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
            });
        }

        // 记录当前视图，用于检测视图切换
        let previous_view = self.current_view;

        // 右方向键切换到输入法管理视图
        if !self.show_help_panel
            && self.current_view == crate::types::ViewMode::Dict
            && ui.input(|i| i.key_pressed(egui::Key::ArrowRight))
        {
            self.current_view = crate::types::ViewMode::Manager;
            self.manager.search_auto_focus = true;
            self.manager.check_and_reload_changed_files();
        }

        // 根据视图模式应用样式（必须在所有 Panel 渲染之前，否则 egui Panel 用默认主题）
        if !self.show_help_panel {
            match self.current_view {
                crate::types::ViewMode::Dict => styles::DictViewStyle::apply(ui.style_mut()),
                crate::types::ViewMode::Manager => styles::ManagerViewStyle::apply(ui.style_mut()),
                crate::types::ViewMode::Chat => styles::DictViewStyle::apply(ui.style_mut()),
            }
        }

        // Top panel (hidden in help mode and chat mode)
        if !self.show_help_panel && !self.chat.show_viewport {
            match self.current_view {
                crate::types::ViewMode::Dict => panel::render_top_panel(self, ui, &ctx),
                crate::types::ViewMode::Manager => {
                    manager_view::render_manager_top_panel(
                        &mut self.manager,
                        ui,
                        &mut self.current_view,
                    );
                }
                crate::types::ViewMode::Chat => panel::render_top_panel(self, ui, &ctx),
            }
        }

        // Bottom panel (hidden in help mode and chat mode)
        if !self.show_help_panel && !self.chat.show_viewport {
            match self.current_view {
                crate::types::ViewMode::Dict => panel::render_bottom_panel(self, ui),
                crate::types::ViewMode::Manager => {
                    manager_view::render_manager_bottom_panel(&self.manager, ui);
                }
                crate::types::ViewMode::Chat => panel::render_bottom_panel(self, ui),
            }
        }

        // Central panel
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if self.show_help_panel {
                help::render_help_fullscreen(self, ui);
            } else {
                self.render_main_content(ui, &ctx);
            }
        });

        // 检查是否切换到了词典视图，如果是则设置自动聚焦标志
        self.check_view_changed_to_dict(previous_view);

        // ========== 跨视图更新进度 toast（从 update_progress 读取详情）==========
        self.update_ui.render_toast(ui);
    }

    fn on_exit(&mut self) {}
}

impl DictApp {
    /// 检查是否切换到了词典视图，如果是则设置自动聚焦标志
    fn check_view_changed_to_dict(&mut self, previous_view: crate::types::ViewMode) {
        if self.current_view == crate::types::ViewMode::Dict
            && previous_view != crate::types::ViewMode::Dict
        {
            self.search_auto_focus = true;
        }
    }

    fn render_main_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // 如果显示 AI 聊天，直接全屏渲染
        if self.chat.show_viewport {
            let action = chat_viewport::render_chat_viewport(
                ui,
                &mut self.chat,
                &self.engine,
                &self.help_manager,
                &self.manager,
            );
            match action {
                Some(ChatAction::Close) => {
                    self.chat.show_viewport = false;
                    self.current_view = crate::types::ViewMode::Dict;
                }
                Some(ChatAction::JumpToDict(q)) => {
                    self.chat.show_viewport = false;
                    self.current_view = crate::types::ViewMode::Dict;
                    self.query = q;
                    self.search_dirty = true;
                }
                None => {}
            }
        } else {
            // 不显示 AI 时根据当前视图渲染主内容
            match self.current_view {
                crate::types::ViewMode::Dict => {
                    self.render_dict_content(ui, ctx);
                }
                crate::types::ViewMode::Manager => {
                    self.render_manager_content(ui, ctx);
                }
                crate::types::ViewMode::Chat => {
                    self.render_dict_content(ui, ctx);
                }
            }
        }
    }

    fn render_dict_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        use crate::dict::Category;

        let search_box_id = egui::Id::new("main_search_box");

        // 自动聚焦搜索框（仅首帧）
        if self.search_auto_focus {
            ui.memory_mut(|mem| mem.request_focus(search_box_id));
            self.search_auto_focus = false;
        }

        // Search bar with clear button
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width() - 28.0, styles::INPUT_BOX_HEIGHT),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.query)
                            .id(search_box_id)
                            .hint_text("🔍 输入文字 或 编码...")
                            .desired_width(f32::INFINITY)
                            .frame(
                                egui::Frame::default()
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY)),
                            ),
                    );
                },
            );

            // ESC 清空（关于弹窗打开时，Esc 仅用于关闭弹窗，不清空搜索框）
            if !self.show_about_dialog && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.query.clear();
            }

            // 清除按钮
            if !self.query.is_empty() && ui.button("×").on_hover_text("清除搜索 (Esc)").clicked()
            {
                self.query.clear();
            }
        });

        // Check if category changed
        let category_changed = self.last_category != self.selected_category;
        self.last_category = self.selected_category;

        // Check if query changed
        let query_changed = self.last_query != self.query;
        self.last_query = self.query.clone();

        // Mark search as dirty when query or category changes
        if query_changed || category_changed {
            self.search_dirty = true;
            self.cached_category_count = None;
        }

        // Perform search only when dirty (optimization: avoid per-frame full scan)
        if self.search_dirty {
            self.search_dirty = false;
            if !self.query.is_empty() {
                let (results, total) = self
                    .engine
                    .search(self.query.trim(), self.selected_category);
                self.search_results = results;
                self.total_results = total;
            } else {
                self.search_results = self.engine.get_by_category(self.selected_category);
                self.total_results = self.search_results.len();
            }
        }

        // Results header
        let result_count = self.search_results.len();
        let result_text = if self.query.trim().is_empty() {
            String::new()
        } else {
            format!("找到 {} 条结果", result_count)
        };
        if !result_text.is_empty() {
            ui.label(result_text);
        }

        // Main content area: table (left) + category description (right)
        let available_height = ui.available_height();

        ui.horizontal(|ui| {
            // Left side: table
            ui.allocate_ui_with_layout(
                egui::vec2(550.0, available_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    table::render_table(self, ui, ctx);
                },
            );

            // Right side: category description
            ui.separator();
            ui.allocate_ui_with_layout(
                egui::vec2(ui.available_width(), available_height),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let cat_label = self
                        .selected_category
                        .map(|c| c.display_name())
                        .unwrap_or("全部");
                    ui.heading(cat_label);
                    ui.separator();

                    let desc = self
                        .selected_category
                        .map(|c| c.description())
                        .unwrap_or(Category::all_description());
                    egui::ScrollArea::vertical()
                        .id_salt("description_scroll")
                        .max_height(ui.available_height())
                        .show(ui, |ui| {
                            ui.label(desc);
                        });
                },
            );
        });
    }

    fn render_manager_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        manager_view::render_manager_view(&mut self.manager, ui, ctx);
    }
}

pub(crate) fn split_at_range(s: &str, range: std::ops::Range<usize>) -> (String, String, String) {
    let before = s[..range.start].to_string();
    let matched = s[range.start..range.end].to_string();
    let after = s[range.end..].to_string();
    (before, matched, after)
}
