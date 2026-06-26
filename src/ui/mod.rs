use crate::DictApp;
use eframe::egui;

mod help;
mod manager_view;
mod panel;
mod styles;
mod table;

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
        if let Ok(guard) = self.update_info.lock()
            && let Some(info) = guard.as_ref()
            && !self.show_update_dialog
        {
            self.show_update_dialog = true;
            self.update_info_for_dialog = Some(info.clone());
        }

        // Show update dialog
        let mut close_dialog = false;
        if self.show_update_dialog
            && let Some(info) = &self.update_info_for_dialog
        {
            let info_clone = info.clone();
            egui::Window::new("发现新版本")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(&ctx, |ui| {
                    ui.label(format!("发现新版本: v{}", info_clone.latest_version));
                    ui.separator();
                    ui.label("更新内容:");
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            ui.label(&info_clone.release_notes);
                        });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("前往下载").clicked() {
                            let _ = open::that(&info_clone.download_url);
                            close_dialog = true;
                        }
                        if ui.button("稍后再说").clicked() {
                            close_dialog = true;
                        }
                    });
                });
        }
        if close_dialog {
            self.show_update_dialog = false;
            self.update_info_for_dialog = None;
            if let Ok(mut guard) = self.update_info.lock() {
                *guard = None;
            }
        }

        // F1 切换帮助文档
        if ui.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_help_panel = !self.show_help_panel;
            if self.show_help_panel && self.selected_help_chapter.is_none() {
                self.selected_help_chapter = Some("readme".to_string());
            }
        }

        // 根据视图模式应用样式（必须在所有 Panel 渲染之前，否则 egui Panel 用默认主题）
        if !self.show_help_panel {
            match self.current_view {
                crate::app::ViewMode::Dict => styles::DictViewStyle::apply(ui.style_mut()),
                crate::app::ViewMode::Manager => styles::ManagerViewStyle::apply(ui.style_mut()),
            }
        }

        // Top panel (hidden in help mode, only for dict view)
        if !self.show_help_panel && self.current_view == crate::app::ViewMode::Dict {
            panel::render_top_panel(self, ui, &ctx);
        }

        // Bottom panel (hidden in help mode, only for dict view)
        if !self.show_help_panel && self.current_view == crate::app::ViewMode::Dict {
            panel::render_bottom_panel(self, ui);
        }

        // Central panel
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if self.show_help_panel {
                help::render_help_fullscreen(self, ui);
            } else {
                self.render_main_content(ui, &ctx);
            }
        });
    }

    fn on_exit(&mut self) {}
}

impl DictApp {
    fn render_main_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        // 根据当前视图渲染不同内容
        match self.current_view {
            crate::app::ViewMode::Dict => {
                self.render_dict_content(ui, ctx);
            }
            crate::app::ViewMode::Manager => {
                self.render_manager_content(ui, ctx);
            }
        }
    }

    fn render_dict_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        use crate::dict::Category;

        let search_box_id = egui::Id::new("main_search_box");

        // 当没有任何 widget 持有焦点时，自动聚焦搜索框
        let has_focus = ui.memory(|mem| mem.has_focus(search_box_id));
        if !has_focus && !self.show_update_dialog {
            let no_widget_focused = ui.memory(|mem| mem.focused().is_none());
            if no_widget_focused {
                ui.memory_mut(|mem| mem.request_focus(search_box_id));
            }
        }

        // Search bar with clear button
        ui.horizontal(|ui| {
            ui.add_sized(
                [ui.available_width() - 28.0, 32.0],
                egui::TextEdit::singleline(&mut self.query)
                    .id(search_box_id)
                    .hint_text("🔍 输入文字 或 编码...")
                    .desired_width(f32::INFINITY),
            );

            // ESC 清空
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.query.clear();
            }

            // 清除按钮
            if !self.query.is_empty() && ui.button("×").on_hover_text("清除搜索 (Esc)").clicked()
            {
                self.query.clear();
            }
        });

        ui.separator();

        // Check if category changed
        let category_changed = self.last_category != self.selected_category;
        self.last_category = self.selected_category;

        // Check if query was just cleared (non-empty -> empty)
        let query_cleared = !self.last_query.is_empty() && self.query.is_empty();
        self.last_query = self.query.clone();

        // Perform search or show category content
        if !self.query.is_empty() {
            let (results, total) = self
                .engine
                .search(self.query.trim(), self.selected_category);
            self.search_results = results;
            self.total_results = total;
        } else if query_cleared || category_changed || self.search_results.is_empty() {
            self.search_results = self.engine.get_by_category(self.selected_category);
            self.total_results = self.search_results.len();
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
        manager_view::render_manager_view(
            &mut self.manager,
            ui,
            ctx,
            &mut self.current_view,
        );
    }
}

pub(crate) fn split_at_range(s: &str, range: std::ops::Range<usize>) -> (String, String, String) {
    let before = s[..range.start].to_string();
    let matched = s[range.start..range.end].to_string();
    let after = s[range.end..].to_string();
    (before, matched, after)
}
