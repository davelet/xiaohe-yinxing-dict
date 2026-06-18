use crate::DictApp;
use eframe::egui;

mod help;
mod panel;
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

        // Top panel (hidden in help mode)
        if !self.show_help_panel {
            panel::render_top_panel(self, ui, &ctx);
        }

        // Bottom panel (hidden in help mode)
        if !self.show_help_panel {
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
        use crate::dict::Category;

        // Search bar
        let _search_changed = ui
            .add_sized(
                [ui.available_width(), 32.0],
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("🔍 输入文字 或 编码...")
                    .desired_width(f32::INFINITY),
            )
            .changed();

        ui.separator();

        // Check if category changed
        let category_changed = self.last_category != self.selected_category;
        self.last_category = self.selected_category;

        // Check if query was just cleared (non-empty -> empty)
        let query_cleared = !self.last_query.is_empty() && self.query.is_empty();
        self.last_query = self.query.clone();

        // Perform search or show category content
        if !self.query.is_empty() {
            self.search_results = self
                .engine
                .search(self.query.trim(), self.selected_category);
        } else if query_cleared || category_changed || self.search_results.is_empty() {
            self.search_results = self.engine.get_by_category(self.selected_category);
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
}

pub(crate) fn split_at_range(s: &str, range: std::ops::Range<usize>) -> (String, String, String) {
    let before = s[..range.start].to_string();
    let matched = s[range.start..range.end].to_string();
    let after = s[range.end..].to_string();
    (before, matched, after)
}
