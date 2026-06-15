use eframe::egui;
use egui::output::OutputCommand;

pub mod dict;
pub mod search;
mod trie;

use dict::{Category, DictEntry};
use search::SearchEngine;

include!(concat!(env!("OUT_DIR"), "/generated_dict.rs"));

impl DictApp {
    fn new(engine: SearchEngine) -> Self {
        let categories = DictEntry::all_categories();
        Self {
            engine,
            categories,
            query: String::new(),
            selected_category: None,
            search_results: Vec::new(),
            copied_feedback: None,
            feedback_timer: 0.0,
        }
    }
}

struct DictApp {
    engine: SearchEngine,
    categories: Vec<Category>,
    query: String,
    selected_category: Option<Category>,
    search_results: Vec<(usize, search::MatchKind)>,
    copied_feedback: Option<(usize, String)>,
    feedback_timer: f32,
}

impl eframe::App for DictApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Handle feedback timer - get context up front
        if self.feedback_timer > 0.0 {
            let dt = ui.ctx().input(|i| i.unstable_dt);
            self.feedback_timer -= dt;
            if self.feedback_timer <= 0.0 {
                self.copied_feedback = None;
            }
            ui.ctx().request_repaint();
        }

        // Top panel: title + category filter
        egui::Panel::top("header_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("小鹤音形词典");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_label("分类筛选")
                        .selected_text(
                            self.selected_category
                                .map(|c| c.display_name())
                                .unwrap_or("全部"),
                        )
                        .width(180.0)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.selected_category,
                                None::<Category>,
                                "全部",
                            );
                            for &cat in &self.categories {
                                ui.selectable_value(
                                    &mut self.selected_category,
                                    Some(cat),
                                    cat.display_name(),
                                );
                            }
                        });
                });
            });
            ui.separator();
        });

        // Bottom panel: status bar
        egui::Panel::bottom("status_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let total = DICT_ENTRIES.len();
                let filtered = self.search_results.len();
                let cat_label = self
                    .selected_category
                    .map(|c| c.display_name())
                    .unwrap_or("全部");
                ui.label(format!(
                    "找到 {} 条结果 | 词典共 {} 条 | 分类: {}",
                    filtered, total, cat_label
                ));
            });
        });

        // Central panel: search bar + results
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let ctx = ui.ctx().clone();

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

            // Perform search
            if !self.query.is_empty() {
                self.search_results = self.engine.search(
                    self.query.trim(),
                    self.selected_category,
                );
            } else {
                self.search_results.clear();
            }

            // Results header
            let result_count = self.search_results.len();
            let result_text = if self.query.trim().is_empty() {
                "输入文字或编码开始搜索".to_string()
            } else {
                format!("找到 {} 条结果", result_count)
            };
            ui.label(result_text);

            // Results table
            if !self.search_results.is_empty() {
                egui::ScrollArea::vertical()
                    .max_height(ui.available_height())
                    .show(ui, |ui| {
                        use egui::Grid;

                        Grid::new("results_grid")
                            .striped(true)
                            .min_col_width(80.0)
                            .show(ui, |ui| {
                                ui.strong("文字");
                                ui.strong("编码");
                                ui.strong("分类");
                                ui.strong("操作");
                                ui.end_row();

                                let results = self.search_results.clone();
                                for (idx, match_kind) in &results {
                                    let entry = &DICT_ENTRIES[*idx];

                                    // Text column (with highlight)
                                    if let search::MatchKind::Text(matched_range) = match_kind {
                                        let (before, matched, after) = split_at_range(
                                            entry.text,
                                            matched_range.clone(),
                                        );
                                        ui.horizontal(|ui| {
                                            ui.label(&before);
                                            ui.colored_label(
                                                egui::Color32::YELLOW,
                                                &matched,
                                            );
                                            ui.label(&after);
                                        });
                                    } else {
                                        ui.label(entry.text);
                                    }

                                    // Code column (with highlight)
                                    if let search::MatchKind::Code(matched_range) = match_kind {
                                        let (before, matched, after) = split_at_range(
                                            entry.code,
                                            matched_range.clone(),
                                        );
                                        ui.horizontal(|ui| {
                                            ui.monospace(&before);
                                            ui.colored_label(
                                                egui::Color32::YELLOW,
                                                &matched,
                                            );
                                            ui.monospace(&after);
                                        });
                                    } else {
                                        ui.monospace(entry.code);
                                    }

                                    // Category column
                                    ui.label(entry.category.display_name());

                                    // Copy button column
                                    let btn_label = if self
                                        .copied_feedback
                                        .as_ref()
                                        .map_or(false, |(id, _)| *id == *idx)
                                    {
                                        "✅ 已复制"
                                    } else {
                                        "📋 复制"
                                    };
                                    let btn_response = ui.add_sized(
                                        [80.0, 24.0],
                                        egui::Button::new(btn_label),
                                    );
                                    if btn_response.clicked() {
                                        let code = entry.code.to_owned();
                                        ctx.output_mut(|o| {
                                            o.commands
                                                .push(OutputCommand::CopyText(code.clone()));
                                        });
                                        self.copied_feedback =
                                            Some((*idx, entry.code.to_owned()));
                                        self.feedback_timer = 2.0;
                                    }

                                    ui.end_row();
                                }
                            });
                    });
            }
        });
    }
}

fn split_at_range(s: &str, range: std::ops::Range<usize>) -> (String, String, String) {
    let before = s[..range.start].to_string();
    let matched = s[range.start..range.end].to_string();
    let after = s[range.end..].to_string();
    (before, matched, after)
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };

    eframe::run_native(
        "xiaohe-yinxing-dict",
        options,
        Box::new(|cc| {
            setup_chinese_fonts(&cc.egui_ctx);
            let engine = SearchEngine::build(&DICT_ENTRIES);
            Ok(Box::new(DictApp::new(engine)))
        }),
    )
}

fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans_sc".to_owned(),
        egui::FontData::from_static(include_bytes!("../fonts/NotoSansSC-Regular.ttf")).into(),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, "noto_sans_sc".to_owned());
    ctx.set_fonts(fonts);
}
