use eframe::egui;
use egui::output::OutputCommand;

pub mod dict;
pub mod help;
pub mod icon;
pub mod search;
mod trie;

use dict::{Category, DictEntry};
use help::HelpManager;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CopyKind {
    Text,
    Code,
}
use search::SearchEngine;

include!(concat!(env!("OUT_DIR"), "/generated_dict.rs"));

impl DictApp {
    fn new(engine: SearchEngine, ctx: &egui::Context) -> Self {
        let categories = DictEntry::all_categories();
        let help_image = Self::load_help_image(ctx);
        let help_manager = HelpManager::new();
        Self {
            engine,
            categories,
            query: String::new(),
            last_query: String::new(),
            selected_category: None,
            last_category: None,
            search_results: Vec::new(),
            copied_feedback: None,
            feedback_timer: 0.0,
            help_image,
            show_help_image: false,
            help_manager,
            show_help_panel: false,
            selected_help_chapter: None,
        }
    }

    fn load_help_image(ctx: &egui::Context) -> Option<egui::TextureHandle> {
        let image_data = include_bytes!("../assets/xhzg.webp");
        if let Ok(img) = image::load_from_memory(image_data) {
            let size = [img.width() as _, img.height() as _];
            let pixels = img.to_rgba8().into_flat_samples().samples;
            let image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
            Some(ctx.load_texture("help_image", image, egui::TextureOptions::default()))
        } else {
            None
        }
    }
}

struct DictApp {
    engine: SearchEngine,
    categories: Vec<Category>,
    query: String,
    last_query: String,
    selected_category: Option<Category>,
    last_category: Option<Category>,
    search_results: Vec<(usize, search::MatchKind)>,
    copied_feedback: Option<(usize, CopyKind)>,
    feedback_timer: f32,
    help_image: Option<egui::TextureHandle>,
    show_help_image: bool,
    help_manager: HelpManager,
    show_help_panel: bool,
    selected_help_chapter: Option<String>,
}

impl eframe::App for DictApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Handle feedback timer - get context up front
        if self.feedback_timer > 0.0 {
            let dt = ctx.input(|i| i.unstable_dt);
            self.feedback_timer -= dt;
            if self.feedback_timer <= 0.0 {
                self.copied_feedback = None;
            }
            ctx.request_repaint();
        }

        // Top panel: title + category filter (hidden in help mode)
        if !self.show_help_panel {
        egui::Panel::top("header_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading("小鹤音形词典");

                // Help button with image tooltip (click to toggle, stays on hover)
                let help_btn = ui.button("❓ 部件字根键位图");

                if help_btn.clicked() {
                    self.show_help_image = !self.show_help_image;
                }

                if self.show_help_image {
                    if let Some(texture) = &self.help_image {
                        let tooltip_pos = help_btn.rect.left_bottom() + egui::vec2(0.0, 4.0);
                        let area_response = egui::Area::new("help_tooltip".into())
                            .fixed_pos(tooltip_pos)
                            .order(egui::Order::Tooltip)
                            .show(ui.ctx(), |ui| {
                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                    let max_w = 750.0;
                                    let max_h = 600.0;
                                    let [iw, ih] = texture.size();
                                    let img_size = egui::vec2(iw as f32, ih as f32);
                                    let scale =
                                        (max_w / img_size.x).min(max_h / img_size.y).min(1.0);
                                    ui.image((texture.id(), img_size * scale));
                                });
                            });

                        // Hide only when mouse leaves both button and image
                        if !help_btn.hovered() && !area_response.response.hovered() {
                            self.show_help_image = false;
                        }
                    }
                }

                // Help documentation button
                let help_doc_btn = ui.button("📖 帮助文档");
                if help_doc_btn.clicked() {
                    self.show_help_panel = !self.show_help_panel;
                    if self.show_help_panel && self.selected_help_chapter.is_none() {
                        self.selected_help_chapter = Some("readme".to_string());
                    }
                }

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
        } // end if !self.show_help_panel

        // Bottom panel: status bar (hidden in help mode)
        if !self.show_help_panel {
        egui::Panel::bottom("status_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let total = DICT_ENTRIES.len();
                let filtered = self.search_results.len();
                let cat_label = self
                    .selected_category
                    .map(|c| c.display_name())
                    .unwrap_or("全部");

                // Get real category count when query is empty
                let display_count = if self.query.trim().is_empty() {
                    self.engine.count_by_category(self.selected_category)
                } else {
                    filtered
                };

                // Show "max 100" note when needed
                let max_note = if self.query.trim().is_empty() && display_count > 100 {
                    " (最多显示100条)"
                } else {
                    ""
                };

                ui.label(format!(
                    "找到 {} 条结果{} | 词典共 {} 条 | 分类: {}",
                    display_count, max_note, total, cat_label
                ));
            });
        });
        } // end if !self.show_help_panel

        // Central panel: search bar + results OR help documentation
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if self.show_help_panel {
                self.render_help_fullscreen(ui);
            } else {
                self.render_main_content(ui, &ctx);
            }
        });
    }

    fn on_exit(&mut self) {}
}

impl DictApp {
    fn render_main_content(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
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
            // Reload when: query cleared, category changed, or first load
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
                    // Table layout constants
                    const COL_WIDTHS: [f32; 5] = [120.0, 100.0, 150.0, 70.0, 70.0];
                    const HEADER_HEIGHT: f32 = 20.0;
                    const ROW_HEIGHT: f32 = 28.0;

                    fn render_cell<F>(
                        ui: &mut egui::Ui,
                        width: f32,
                        height: f32,
                        align_center: bool,
                        content: F,
                    ) where
                        F: FnOnce(&mut egui::Ui),
                    {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(width, height),
                            egui::Sense::hover(),
                        );
                        let layout = if align_center {
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
                        } else {
                            egui::Layout::left_to_right(egui::Align::Center)
                                .with_main_wrap(false)
                        };
                        let mut child_ui =
                            ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout));
                        content(&mut child_ui);
                    }

                    // Table header + body
                    let body_height = ui.available_height();
                    egui::ScrollArea::vertical()
                        .id_salt("table_scroll")
                        .max_height(body_height)
                        .show(ui, |ui| {
                            // Table header
                            ui.horizontal(|ui| {
                                let headers = ["文字", "编码", "分类", "操作", ""];
                                for (i, header) in headers.iter().enumerate() {
                                    render_cell(
                                        ui,
                                        COL_WIDTHS[i],
                                        HEADER_HEIGHT,
                                        i != 0,
                                        |ui| {
                                            ui.strong(*header);
                                        },
                                    );
                                }
                            });
                            ui.separator();

                            if !self.search_results.is_empty() {
                                let results = self.search_results.clone();
                                for (idx, match_kind) in &results {
                                    let entry = &DICT_ENTRIES[*idx];

                                    ui.horizontal(|ui| {
                                        // Text column (with highlight) - left aligned with horizontal scroll
                                        render_cell(
                                            ui,
                                            COL_WIDTHS[0],
                                            ROW_HEIGHT,
                                            false,
                                            |ui| {
                                                egui::ScrollArea::horizontal()
                                                    .id_salt(format!("text_scroll_{}", idx))
                                                    .show(ui, |ui| {
                                                        ui.set_min_height(ROW_HEIGHT);
                                                        if let search::MatchKind::Text(
                                                            matched_range,
                                                        ) = match_kind
                                                        {
                                                            let (before, matched, after) =
                                                                split_at_range(
                                                                    entry.text,
                                                                    matched_range.clone(),
                                                                );
                                                            ui.label(&before);
                                                            ui.colored_label(
                                                                egui::Color32::from_rgb(
                                                                    0, 130, 0,
                                                                ),
                                                                &matched,
                                                            );
                                                            ui.monospace(&after);
                                                        } else {
                                                            ui.label(entry.text);
                                                        }
                                                    });
                                            },
                                        );

                                        // Code column - centered
                                        render_cell(
                                            ui,
                                            COL_WIDTHS[1],
                                            ROW_HEIGHT,
                                            true,
                                            |ui| {
                                                ui.monospace(entry.code);
                                            },
                                        );

                                        // Category column - centered
                                        render_cell(
                                            ui,
                                            COL_WIDTHS[2],
                                            ROW_HEIGHT,
                                            true,
                                            |ui| {
                                                ui.label(entry.category.display_name());
                                            },
                                        );

                                        // Copy text button
                                        render_cell(
                                            ui,
                                            COL_WIDTHS[3],
                                            ROW_HEIGHT,
                                            true,
                                            |ui| {
                                                let copied_text = self
                                                    .copied_feedback
                                                    .as_ref()
                                                    .is_some_and(|(id, kind)| {
                                                        *id == *idx && *kind == CopyKind::Text
                                                    });
                                                let btn_label = if copied_text {
                                                    "✅文字"
                                                } else {
                                                    "📋文字"
                                                };
                                                let btn_response = ui.add_sized(
                                                    [64.0, 24.0],
                                                    egui::Button::new(btn_label),
                                                );
                                                if btn_response.clicked() {
                                                    ctx.output_mut(|o| {
                                                        o.commands.push(
                                                            OutputCommand::CopyText(
                                                                entry.text.to_owned(),
                                                            ),
                                                        );
                                                    });
                                                    self.copied_feedback =
                                                        Some((*idx, CopyKind::Text));
                                                    self.feedback_timer = 2.0;
                                                }
                                            },
                                        );

                                        // Copy code button
                                        render_cell(
                                            ui,
                                            COL_WIDTHS[4],
                                            ROW_HEIGHT,
                                            true,
                                            |ui| {
                                                let copied_code = self
                                                    .copied_feedback
                                                    .as_ref()
                                                    .is_some_and(|(id, kind)| {
                                                        *id == *idx && *kind == CopyKind::Code
                                                    });
                                                let btn_label = if copied_code {
                                                    "✅编码"
                                                } else {
                                                    "📋编码"
                                                };
                                                let btn_response = ui.add_sized(
                                                    [64.0, 24.0],
                                                    egui::Button::new(btn_label),
                                                );
                                                if btn_response.clicked() {
                                                    ctx.output_mut(|o| {
                                                        o.commands.push(
                                                            OutputCommand::CopyText(
                                                                entry.code.to_owned(),
                                                            ),
                                                        );
                                                    });
                                                    self.copied_feedback =
                                                        Some((*idx, CopyKind::Code));
                                                    self.feedback_timer = 2.0;
                                                }
                                            },
                                        );
                                    });
                                }
                            } else {
                                ui.label("无匹配结果");
                            }
                        });
                },
            );

            // Right side: category description (always visible)
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

                    let desc = get_category_description(self.selected_category);
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

    fn render_help_fullscreen(&mut self, ui: &mut egui::Ui) {
        // Top bar: title + back button
        ui.horizontal(|ui| {
            if ui.button("← 返回词典").clicked() {
                self.show_help_panel = false;
            }
            ui.heading("📖 帮助文档");
        });
        ui.separator();

        // Two-column layout using SidePanel for chapter list
        let chapters = self.help_manager.chapters().to_vec();
        let current_chapter = self.selected_help_chapter.clone();

        egui::Panel::left("help_sidebar")
            .resizable(false)
            .min_size(180.0)
            .max_size(250.0)
            .default_size(200.0)
            .show_inside(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("help_chapter_list")
                    .show(ui, |ui| {
                        for chapter in &chapters {
                            let is_selected =
                                current_chapter.as_deref() == Some(chapter.id);
                            let button =
                                ui.selectable_label(is_selected, chapter.title);
                            if button.clicked() {
                                self.selected_help_chapter =
                                    Some(chapter.id.to_string());
                            }
                        }
                    });
            });

        // Content fills the remaining central area
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(chapter_id) = &self.selected_help_chapter {
                if let Some(chapter) = self.help_manager.get_chapter(chapter_id) {
                    egui::ScrollArea::vertical()
                        .id_salt("help_content")
                        .show(ui, |ui| {
                            HelpManager::render_markdown(ui, chapter.content);
                        });
                }
            } else {
                ui.label("请从左侧选择帮助章节");
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

fn get_category_description(category: Option<Category>) -> &'static str {
    match category {
        Some(Category::YiJiJianMa) => {
            "一级简码是小鹤音形中最常用的26个汉字，每个字母对应一个高频字。\n\n例如：\n- q = 起\n- w = 我\n- e = 而\n- r = 人\n直接按一个字母加空格即可输入。"
        }
        Some(Category::ErChongJianMa) => {
            "二重简码是使用两个字母编码的常用汉字，约有600多个。\n\n例如：\n- qb = 情\n- qc = 请\n- qd = 巧\n输入两个字母后按空格即可输入。"
        }
        Some(Category::SanMaTianKong) => {
            "三码填空是指三码编码的汉字，在输入三码后系统会自动填空上屏。\n\n这是小鹤音形的特色功能，无需按空格确认，大大提升输入速度。"
        }
        Some(Category::SiMaQuanMaZi) => {
            "四码全码（单字）是完整的四码编码汉字。\n\n编码规则：声 + 韵 + 首形 + 尾形\n\n当输入四码时，如果只有一个候选字，会自动上屏。"
        }
        Some(Category::SiMaQuanMaCiZhiDing) => {
            "四码全码（词置顶）是指在词库中优先级最高的词组，会在候选框中置顶显示。\n\n这些词组通常是最常用的固定搭配。"
        }
        Some(Category::SiMaQuanMaCi) => {
            "四码全码（词）是普通词组，按照词组编码规则输入。\n\n双字词：首字前两码 + 次字前两码\n三字词：前两字首码 + 第三字前两码\n四字及以上词：前三字首码 + 末字首码"
        }
        Some(Category::KuaiFu) => {
            "快符是快速输入特殊符号的功能。\n\n例如：\n- ; = 。\n- ;; = ；\n- ;a = ！\n- ;b = （\n输入分号后跟一个字母即可快速输入对应符号。"
        }
        Some(Category::FuHao) => {
            "符号分类包含各种特殊符号和标点。\n\n包含：数学符号、标点符号、箭头符号、括号符号等。\n\n可以通过编码反查来快速找到需要的符号。"
        }
        Some(Category::BuShouBuJian) => {
            "部首部件是汉字的基本组成部分，了解这些部件有助于理解和记忆字形编码。\n\n每个部首都有对应的编码，掌握后可以更准确地拆分生僻字。"
        }
        Some(Category::Emoji) => {
            "Emoji表情符号，可以通过编码输入各种表情。\n\n例如：\n- hh = 😄 (哈哈)\n- kx = 😊 (开心)\n- wq = 😢 (委屈)\n通过拼音首字母即可快速输入常用表情。"
        }
        Some(Category::WeiXinBiaoQing) => {
            "微信表情是微信中常用的表情图。\n\n可以通过编码快速输入对应的微信表情文字描述或快捷短语。"
        }
        Some(Category::WangZhanZhiDa) => {
            "网站直达是通过编码快速打开常用网站的功能。\n\n例如输入对应编码后按指定键即可在浏览器中打开网站。"
        }
        Some(Category::SuiXinSuoYu) => {
            "随心所欲是一些自定义的特殊短语和快捷输入。\n\n包含各种实用的快捷短语和特殊功能，方便快速输入长文本。"
        }
        Some(Category::ErJianCiXuan) => {
            "二简（次选字）是二重简码的次选字，即两个字母编码的第二个候选字。\n\n通常按分号键选择次选，按引号键选择三选。"
        }
        Some(Category::SiMaCiXuan) => {
            "四码（次选词）是四码全码词组的次选候选词。\n\n当多个词组编码相同时，按分号可以选择次选词，按引号可以选择三选词。"
        }
        Some(Category::ShouXuanSiMa) => {
            "首选四码（词/短语）是四码词组中的首选候选，即第一个候选词。\n\n这些是最常用的词组，输入四码后直接按空格即可上屏。"
        }
        None => {
            "全部分类显示词典中所有类型的条目。\n\n小鹤音形是一款音形码输入法，结合了拼音和字形的优点：\n\n• 音码：双拼方案，每个拼音两码完成\n• 形码：基于汉字首尾部结构\n\n特点：\n- 低重码：音形结合大幅降低重码\n- 盲打：低重码支持真正的盲打\n- 易学：双拼+简单字形规则\n- 高效：自动填空、四码唯一自动上屏\n\n建议从一级简码和二重简码开始学习，逐步掌握三码和四码全码。"
        }
    }
}

fn create_app_icon() -> egui::IconData {
    let size = 64u32;
    let rgba = icon::generate_icon_rgba(size);
    egui::IconData {
        rgba,
        width: size,
        height: size,
    }
}

fn main() -> eframe::Result {
    let icon = create_app_icon();
    let options = eframe::NativeOptions {
        run_and_return: false,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([950.0, 650.0])
            .with_resizable(true)
            .with_maximize_button(true)
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "小鹤音形词典",
        options,
        Box::new(|cc| {
            setup_chinese_fonts(&cc.egui_ctx);
            let engine = SearchEngine::build(&DICT_ENTRIES);
            Ok(Box::new(DictApp::new(engine, &cc.egui_ctx)))
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
