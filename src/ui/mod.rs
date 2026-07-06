use crate::DictApp;
use eframe::egui;

mod add_word_dialog;
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
                            let progress = self.update_progress.clone();
                            let info = info_clone.clone();
                            let ctx = ui.ctx().clone();
                            self.update_retry_info = Some(info.clone());
                            self.start_update_thread(state, progress, info, ctx);
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

        // ========== 跨视图更新进度 toast（从 update_progress 读取详情）==========
        let dt = ui.ctx().input(|i| i.unstable_dt);

        // 记录下载开始时间（用于计算已用时间）
        if let Ok(guard) = self.update_state.lock() {
            if *guard == crate::update::UpdateState::Downloading
                || *guard == crate::update::UpdateState::Installing
            {
                if self.update_started_at.is_none() {
                    self.update_started_at = Some(std::time::Instant::now());
                }
            } else {
                self.update_started_at = None;
            }
        }

        // 从 update_state 同步 toast 开关
        let mut in_progress = false;
        let mut is_done = false;
        let mut is_failed = false;
        if let Ok(guard) = self.update_state.lock() {
            match &*guard {
                crate::update::UpdateState::Downloading
                | crate::update::UpdateState::Installing => {
                    self.update_toast_timer = 0.0;
                    in_progress = true;
                }
                crate::update::UpdateState::Done(_) => {
                    if !self.update_toast_shown_done_or_failed {
                        self.update_toast_shown_done_or_failed = true;
                    }
                    self.update_toast_timer = 0.0; // 常驻，不自动消失
                    is_done = true;
                }
                crate::update::UpdateState::Failed(_) => {
                    if !self.update_toast_shown_done_or_failed {
                        self.update_toast_shown_done_or_failed = true;
                    }
                    self.update_toast_timer = 0.0; // 常驻，不自动消失
                    is_failed = true;
                }
                crate::update::UpdateState::Idle => {
                    self.update_toast_dismissed = true;
                }
            }
        }

        // toast 倒计时（不再用于 Done/Failed 自动消失，只用于兼容）
        if self.update_toast_timer > 0.0 {
            self.update_toast_timer -= dt;
            if self.update_toast_timer <= 0.0 {
                self.update_toast_timer = 0.0;
            }
        }

        let show_toast = in_progress || is_done || is_failed;

        // 渲染富文本 toast
        if show_toast {
            // 从 progress / state 读取详细信息
            let (prog_msg, prog_down, prog_total, sha256_ok) =
                if let Ok(p) = self.update_progress.lock() {
                    (
                        p.message.clone(),
                        p.bytes_downloaded,
                        p.bytes_total,
                        p.sha256_ok,
                    )
                } else {
                    (String::new(), 0, 0, None)
                };

            let (state_label, state_color) = if in_progress {
                if let Ok(guard) = self.update_state.lock() {
                    match &*guard {
                        crate::update::UpdateState::Downloading => ("下载中", egui::Color32::WHITE),
                        crate::update::UpdateState::Installing => ("安装中", egui::Color32::WHITE),
                        _ => ("进行中", egui::Color32::WHITE),
                    }
                } else {
                    ("进行中", egui::Color32::WHITE)
                }
            } else if is_done {
                ("完成", egui::Color32::from_rgb(100, 220, 100))
            } else {
                ("失败", egui::Color32::from_rgb(255, 80, 80))
            };

            // 计算已用时间
            let elapsed_str = self.update_started_at.map(|start| {
                let secs = start.elapsed().as_secs_f64();
                if secs < 60.0 {
                    format!("{:.0}秒", secs)
                } else if secs < 3600.0 {
                    format!("{:.0}分{:.0}秒", secs / 60.0, secs % 60.0)
                } else {
                    format!("{:.1}小时", secs / 3600.0)
                }
            });

            // 进度百分比
            let progress_ratio = if prog_total > 0 {
                (prog_down as f64 / prog_total as f64).clamp(0.0, 1.0)
            } else {
                0.0
            };

            let bg = if in_progress {
                egui::Color32::from_rgba_premultiplied(30, 30, 40, 230)
            } else {
                egui::Color32::from_rgba_premultiplied(40, 40, 50, 220)
            };
            let text_color = egui::Color32::from_rgb(220, 220, 220);

            // 完成/失败时读取 exe_path 和错误详情
            let (done_exe_path, fail_error) = if !in_progress {
                if let Ok(guard) = self.update_state.lock() {
                    match &*guard {
                        crate::update::UpdateState::Done(path) => (Some(path.clone()), None),
                        crate::update::UpdateState::Failed(e) => (None, Some(e.clone())),
                        _ => (None, None),
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

            let retry_info = self.update_retry_info.clone();

            egui::Area::new("update_progress_toast".into())
                .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -36.0])
                .order(egui::Order::Foreground)
                .show(ui.ctx(), |ui| {
                    let frame = egui::Frame::NONE
                        .fill(bg)
                        .corner_radius(8.0)
                        .inner_margin(egui::Margin::symmetric(14, 10));
                    frame.show(ui, |ui| {
                        ui.set_max_width(420.0);

                        // 第一行：状态标签 + 已用时间
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(state_label)
                                    .color(state_color)
                                    .size(12.0)
                                    .strong(),
                            );
                            if let Some(ref elap) = elapsed_str {
                                ui.label(
                                    egui::RichText::new(format!("  ⏱ {}", elap))
                                        .color(egui::Color32::from_rgb(160, 180, 200))
                                        .size(11.0),
                                );
                            }
                        });

                        ui.add_space(4.0);

                        // 第二行：下载进度 + 大小（仅在下载中显示）
                        if in_progress && prog_total > 0 {
                            // 进度条
                            let bar_width = 390.0;
                            let bar_height = 6.0;
                            let (bar_rect, _) = ui.allocate_exact_size(
                                egui::vec2(bar_width, bar_height),
                                egui::Sense::hover(),
                            );
                            if ui.is_rect_visible(bar_rect) {
                                ui.painter().rect_filled(
                                    bar_rect,
                                    egui::CornerRadius::same(3),
                                    egui::Color32::from_rgba_premultiplied(255, 255, 255, 30),
                                );
                                let filled_w = (bar_rect.width() as f64 * progress_ratio) as f32;
                                if filled_w > 0.0 {
                                    let filled_rect = egui::Rect::from_min_size(
                                        bar_rect.min,
                                        egui::vec2(filled_w, bar_height),
                                    );
                                    ui.painter().rect_filled(
                                        filled_rect,
                                        egui::CornerRadius::same(3),
                                        egui::Color32::from_rgb(80, 180, 255),
                                    );
                                }
                            }

                            ui.add_space(2.0);

                            // 大小文本
                            let down_mb = prog_down as f64 / 1_048_576.0;
                            let total_mb = prog_total as f64 / 1_048_576.0;
                            ui.label(
                                egui::RichText::new(format!(
                                    "{:.1} MB / {:.1} MB  ({:.0}%)",
                                    down_mb,
                                    total_mb,
                                    progress_ratio * 100.0
                                ))
                                .color(egui::Color32::from_rgb(180, 200, 220))
                                .size(11.0),
                            );
                        } else if in_progress && prog_total == 0 {
                            ui.label(egui::RichText::new(&prog_msg).color(text_color).size(12.0));
                        }

                        // 第三行：SHA256 校验 + 状态消息
                        if !prog_msg.is_empty() {
                            ui.add_space(2.0);
                            ui.horizontal(|ui| {
                                if let Some(ok) = sha256_ok {
                                    let (icon, hash_color) = if ok {
                                        ("✓", egui::Color32::from_rgb(100, 220, 100))
                                    } else {
                                        ("✗", egui::Color32::from_rgb(255, 80, 80))
                                    };
                                    ui.label(
                                        egui::RichText::new(format!("SHA256 {}", icon))
                                            .color(hash_color)
                                            .size(11.0),
                                    );
                                    ui.add_space(4.0);
                                }
                                ui.label(
                                    egui::RichText::new(&prog_msg).color(text_color).size(11.0),
                                );
                            });
                        }

                        // 第四行：失败详情
                        if let Some(ref err) = fail_error {
                            ui.add_space(2.0);
                            ui.label(
                                egui::RichText::new(err)
                                    .color(egui::Color32::from_rgb(255, 140, 140))
                                    .size(11.0),
                            );
                        }

                        // 第五行：操作按钮（Done / Failed / 下载中取消）
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if in_progress {
                                // 下载/安装中：取消按钮
                                if ui
                                    .add(
                                        egui::Button::new("x 取消")
                                            .min_size(egui::vec2(60.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    self.update_cancelled
                                        .store(true, std::sync::atomic::Ordering::Relaxed);
                                    *self.update_state.lock().unwrap_or_else(|e| e.into_inner()) =
                                        crate::update::UpdateState::Idle;
                                }
                            } else if is_done {
                                // 完成：重启 + 稍后
                                let exe_path = done_exe_path.clone();
                                if ui
                                    .add(
                                        egui::Button::new(" 🔄 立即重启 ")
                                            .min_size(egui::vec2(90.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    #[cfg(target_os = "macos")]
                                    if let Some(ref p) = exe_path {
                                        let _ = std::process::Command::new("open").arg(p).spawn();
                                    }
                                    std::process::exit(0);
                                }
                                if ui
                                    .add(
                                        egui::Button::new(" 稍后 ")
                                            .min_size(egui::vec2(60.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    *self.update_state.lock().unwrap_or_else(|e| e.into_inner()) =
                                        crate::update::UpdateState::Idle;
                                }
                            } else if is_failed {
                                // 失败：重试 + 关闭
                                if retry_info.is_some()
                                    && ui
                                        .add(
                                            egui::Button::new(" 🔄 重试 ")
                                                .min_size(egui::vec2(70.0, 24.0)),
                                        )
                                        .clicked()
                                    && let Some(ref info) = retry_info
                                {
                                    // 重置状态并重新开始下载
                                    *self.update_state.lock().unwrap_or_else(|e| e.into_inner()) =
                                        crate::update::UpdateState::Idle;
                                    self.start_update_thread(
                                        self.update_state.clone(),
                                        self.update_progress.clone(),
                                        info.clone(),
                                        ui.ctx().clone(),
                                    );
                                }
                                if ui
                                    .add(
                                        egui::Button::new("x 关闭")
                                            .min_size(egui::vec2(60.0, 24.0)),
                                    )
                                    .clicked()
                                {
                                    *self.update_state.lock().unwrap_or_else(|e| e.into_inner()) =
                                        crate::update::UpdateState::Idle;
                                }
                            }
                        });
                    });

                    if in_progress {
                        ui.ctx().request_repaint();
                    }
                });
        }
    }

    fn on_exit(&mut self) {}
}

impl DictApp {
    /// 启动更新下载和安装线程
    fn start_update_thread(
        &mut self,
        state: std::sync::Arc<std::sync::Mutex<crate::update::UpdateState>>,
        progress: std::sync::Arc<std::sync::Mutex<crate::update::UpdateProgress>>,
        info: crate::update::UpdateInfo,
        ctx: egui::Context,
    ) {
        let cancelled = self.update_cancelled.clone();
        cancelled.store(false, std::sync::atomic::Ordering::Relaxed);
        {
            let mut s = state.lock().unwrap_or_else(|e| e.into_inner());
            *s = crate::update::UpdateState::Downloading;
        }
        {
            let mut p = progress.lock().unwrap_or_else(|e| e.into_inner());
            p.message = "正在连接服务器...".to_string();
            p.bytes_downloaded = 0;
            p.bytes_total = 0;
            p.sha256_ok = None;
        }
        self.update_toast_shown_done_or_failed = false;
        ctx.request_repaint();
        std::thread::spawn(move || {
            let result = (|| -> Result<std::path::PathBuf, String> {
                let zip_path = crate::update::download_update(&info, &progress, &cancelled)?;
                // 如果中途被取消，直接返回
                if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                    let _ = std::fs::remove_file(&zip_path);
                    return Err("已取消".to_string());
                }
                *state.lock().unwrap_or_else(|e| e.into_inner()) =
                    crate::update::UpdateState::Installing;
                {
                    let mut p = progress.lock().unwrap_or_else(|e| e.into_inner());
                    p.message = "正在安装...".to_string();
                }
                ctx.request_repaint();
                // 安装前再次检查取消
                if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                    return Err("已取消".to_string());
                }
                let new_exe = crate::update::apply_update(&zip_path)?;
                Ok(new_exe)
            })();
            // 如果已取消，不更新状态（状态已在取消时被设为 Idle）
            if cancelled.load(std::sync::atomic::Ordering::Relaxed) {
                return;
            }
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
    }

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
