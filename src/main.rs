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
        // Allow clean exit when window close is requested
        if ui.input(|i| i.viewport().close_requested()) {
            // eframe handles the exit, just allow it
        }

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
                    // Keyboard nav for category cycling (works even without popup open)
                    let k_down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
                    let k_up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
                    if k_down || k_up {
                        let all_cats: Vec<Option<Category>> = std::iter::once(None)
                            .chain(self.categories.iter().copied().map(Some))
                            .collect();
                        let curr = all_cats
                            .iter()
                            .position(|&c| c == self.selected_category)
                            .unwrap_or(0);
                        let next = if k_down {
                            (curr + 1) % all_cats.len()
                        } else {
                            (curr + all_cats.len() - 1) % all_cats.len()
                        };
                        self.selected_category = all_cats[next];
                    }

                    // ComboBox (handles mouse clicks natively)
                    ui.style_mut().spacing.combo_height = 480.0;
                    egui::ComboBox::from_id_salt("category_combo")
                        .selected_text(
                            self.selected_category
                                .map(|c| c.display_name())
                                .unwrap_or("全部"),
                        )
                        .width(350.0)
                        .height(480.0)
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
                    ui.label("分类筛选");
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
                self.search_results = self
                    .engine
                    .search(self.query.trim(), self.selected_category);
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
                                        let (before, matched, after) =
                                            split_at_range(entry.text, matched_range.clone());
                                        ui.horizontal(|ui| {
                                            ui.label(&before);
                                            ui.colored_label(
                                                egui::Color32::from_rgb(0, 130, 0),
                                                &matched,
                                            );
                                            ui.monospace(&after);
                                        });
                                    } else {
                                        ui.label(entry.text);
                                    }

                                    // Code column
                                    ui.monospace(entry.code);

                                    // Category column
                                    ui.label(entry.category.display_name());

                                    // Copy button column
                                    let btn_label = if self
                                        .copied_feedback
                                        .as_ref()
                                        .is_some_and(|(id, _)| *id == *idx)
                                    {
                                        "✅ 已复制"
                                    } else {
                                        "📋 复制"
                                    };
                                    let btn_response =
                                        ui.add_sized([80.0, 24.0], egui::Button::new(btn_label));
                                    if btn_response.clicked() {
                                        let code = entry.code.to_owned();
                                        ctx.output_mut(|o| {
                                            o.commands.push(OutputCommand::CopyText(code.clone()));
                                        });
                                        self.copied_feedback = Some((*idx, entry.code.to_owned()));
                                        self.feedback_timer = 2.0;
                                    }

                                    ui.end_row();
                                }
                            });
                    });
            }
        });
    }

    fn on_exit(&mut self) {}
}

fn split_at_range(s: &str, range: std::ops::Range<usize>) -> (String, String, String) {
    let before = s[..range.start].to_string();
    let matched = s[range.start..range.end].to_string();
    let after = s[range.end..].to_string();
    (before, matched, after)
}

fn create_app_icon() -> egui::IconData {
    let size = 64u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];
    let half = size as f32 / 2.0;
    let radius = half - 1.0;

    for y in 0..size {
        for x in 0..size {
            let i = (y * size + x) as usize * 4;
            let dx = x as f32 - half;
            let dy = y as f32 - half;
            let dist = (dx * dx + dy * dy).sqrt();

            if dist > radius {
                rgba[i + 3] = 0; // transparent outside circle
                continue;
            }

            // Background: teal (#0d9488) to indigo (#4338ca) gradient
            let t = dist / radius;
            rgba[i] = (13.0 + (67.0 - 13.0) * t) as u8;
            rgba[i + 1] = (148.0 - (148.0 - 56.0) * t) as u8;
            rgba[i + 2] = (136.0 - (136.0 - 202.0) * t) as u8;
            rgba[i + 3] = 255;

            let nx = dx / radius; // -1..1
            let ny = dy / radius; // -1..1
            let _n_dist = dist / radius; // 0..1

            // Draw white "crane in flight" silhouette
            let mut white = false;

            // Head: small circle at top
            let hx = 0.0;
            let hy = -0.45;
            if (nx - hx).powi(2) + (ny - hy).powi(2) < 0.035 {
                white = true;
            }

            // Beak: small triangle pointing right from head
            if ny > -0.50 && ny < -0.40 && nx > 0.12 && nx < 0.30 {
                let beak_top = -0.50 + (nx - 0.12) * 0.2;
                let beak_bot = -0.40 - (nx - 0.12) * 0.2;
                if ny > beak_top && ny < beak_bot {
                    white = true;
                }
            }

            // Body: thin vertical oval
            if nx.abs() < 0.08 && ny > -0.35 && ny < 0.25 {
                white = true;
            }

            // Left wing: triangular shape spreading up-left
            if nx < -0.05 && ny > -0.45 && ny < 0.05 {
                let wing_upper = -0.45 + (nx + 0.7) * 0.6;
                let wing_lower = 0.05 - (nx + 0.7) * 0.4;
                if ny > wing_upper && ny < wing_lower && nx > -0.7 {
                    white = true;
                }
            }

            // Right wing: triangular shape spreading up-right
            if nx > 0.05 && ny > -0.45 && ny < 0.05 {
                let wing_upper = -0.45 + (0.7 - nx) * 0.6;
                let wing_lower = 0.05 - (0.7 - nx) * 0.4;
                if ny > wing_upper && ny < wing_lower && nx < 0.7 {
                    white = true;
                }
            }

            // Tail feathers: small fan at bottom
            if ny > 0.20 && ny < 0.45 && nx.abs() < 0.15 {
                let tail_width = 0.15 * (1.0 - (ny - 0.20) / 0.25);
                if nx.abs() < tail_width {
                    white = true;
                }
            }

            if white {
                rgba[i] = 255;
                rgba[i + 1] = 255;
                rgba[i + 2] = 255;
            }
        }
    }

    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

fn main() -> eframe::Result {
    let icon = create_app_icon();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "小鹤音形词典",
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
