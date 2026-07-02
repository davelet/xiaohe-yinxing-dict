use std::path::PathBuf;

use crate::DictApp;
use eframe::egui;

mod common;
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
            && *self.update_state.lock().unwrap_or_else(|e| e.into_inner())
                == crate::update::UpdateState::Idle
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
            let update_state = self.update_state.clone();
            let update_state_clone = update_state.clone();
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
                        let is_downloading = {
                            let guard =
                                update_state_clone.lock().unwrap_or_else(|e| e.into_inner());
                            *guard == crate::update::UpdateState::Downloading
                                || *guard == crate::update::UpdateState::Installing
                        };
                        if ui
                            .add_enabled(!is_downloading, egui::Button::new("立即更新"))
                            .clicked()
                        {
                            let state = update_state.clone();
                            let info = info_clone.clone();
                            let ctx = ui.ctx().clone();
                            *state.lock().unwrap_or_else(|e| e.into_inner()) =
                                crate::update::UpdateState::Downloading;
                            ctx.request_repaint();
                            std::thread::spawn(move || {
                                let result = (|| -> Result<PathBuf, String> {
                                    let zip_path = crate::update::download_update(&info)?;
                                    *state.lock().unwrap_or_else(|e| e.into_inner()) =
                                        crate::update::UpdateState::Installing;
                                    ctx.request_repaint();
                                    let new_exe = crate::update::apply_update(&zip_path)?;
                                    Ok(new_exe)
                                })();
                                match result {
                                    Ok(new_exe) => {
                                        *state.lock().unwrap_or_else(|e| e.into_inner()) =
                                            crate::update::UpdateState::Done(new_exe);
                                    }
                                    Err(e) => {
                                        *state.lock().unwrap_or_else(|e| e.into_inner()) =
                                            crate::update::UpdateState::Failed(e);
                                    }
                                }
                                ctx.request_repaint();
                            });
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

        // Show update done dialog
        let mut close_done_dialog = false;
        if let crate::update::UpdateState::Done(ref exe_path) =
            *self.update_state.lock().unwrap_or_else(|e| e.into_inner())
        {
            let exe_path = exe_path.clone();
            egui::Window::new("更新完成")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(&ctx, |ui| {
                    ui.label("更新完成，重启后生效");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("立即重启").clicked() {
                            #[cfg(target_os = "macos")]
                            {
                                let _ = std::process::Command::new("open").arg(&exe_path).spawn();
                            }
                            std::process::exit(0);
                        }
                        if ui.button("稍后").clicked() {
                            close_done_dialog = true;
                        }
                    });
                });
        }
        if close_done_dialog {
            *self.update_state.lock().unwrap_or_else(|e| e.into_inner()) =
                crate::update::UpdateState::Idle;
        }

        // F1 切换帮助文档
        if ui.input(|i| i.key_pressed(egui::Key::F1)) {
            self.show_help_panel = !self.show_help_panel;
            if self.show_help_panel && self.selected_help_chapter.is_none() {
                self.selected_help_chapter = Some("readme".to_string());
            }
        }

        // 记录当前视图，用于检测视图切换
        let previous_view = self.current_view;

        // 右方向键切换到输入法管理视图
        if !self.show_help_panel
            && self.current_view == crate::app::ViewMode::Dict
            && ui.input(|i| i.key_pressed(egui::Key::ArrowRight))
        {
            self.current_view = crate::app::ViewMode::Manager;
            self.manager.search_auto_focus = true;
            self.manager.check_and_reload_changed_files();
        }

        // 根据视图模式应用样式（必须在所有 Panel 渲染之前，否则 egui Panel 用默认主题）
        if !self.show_help_panel {
            match self.current_view {
                crate::app::ViewMode::Dict => styles::DictViewStyle::apply(ui.style_mut()),
                crate::app::ViewMode::Manager => styles::ManagerViewStyle::apply(ui.style_mut()),
            }
        }

        // Top panel (hidden in help mode)
        if !self.show_help_panel {
            match self.current_view {
                crate::app::ViewMode::Dict => panel::render_top_panel(self, ui, &ctx),
                crate::app::ViewMode::Manager => {
                    manager_view::render_manager_top_panel(
                        &mut self.manager,
                        ui,
                        &mut self.current_view,
                    );
                }
            }
        }

        // Bottom panel (hidden in help mode)
        if !self.show_help_panel {
            match self.current_view {
                crate::app::ViewMode::Dict => panel::render_bottom_panel(self, ui),
                crate::app::ViewMode::Manager => {
                    manager_view::render_manager_bottom_panel(&self.manager, ui);
                }
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
    }

    fn on_exit(&mut self) {}
}

impl DictApp {
    /// 检查是否切换到了词典视图，如果是则设置自动聚焦标志
    fn check_view_changed_to_dict(&mut self, previous_view: crate::app::ViewMode) {
        if self.current_view == crate::app::ViewMode::Dict
            && previous_view != crate::app::ViewMode::Dict
        {
            self.search_auto_focus = true;
        }
    }

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
        manager_view::render_manager_view(&mut self.manager, ui, ctx);
    }
}

pub(crate) fn split_at_range(s: &str, range: std::ops::Range<usize>) -> (String, String, String) {
    let before = s[..range.start].to_string();
    let matched = s[range.start..range.end].to_string();
    let after = s[range.end..].to_string();
    (before, matched, after)
}
