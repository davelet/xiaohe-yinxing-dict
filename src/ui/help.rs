use crate::DictApp;
use crate::help;
use eframe::egui;

/// 清理输入文本，只保留汉字、字母和数字
fn clean_input_text(text: &str) -> String {
    help::filter_chinese_and_letters(text)
}

/// 渲染左侧章节列表
fn chapter_sidebar(
    ui: &mut egui::Ui,
    chapters: &[help::HelpChapter],
    current_chapter: &Option<String>,
    selected: &mut Option<String>,
) {
    egui::ScrollArea::vertical()
        .id_salt("help_chapter_list")
        .show(ui, |ui| {
            for chapter in chapters {
                let is_selected = current_chapter.as_deref() == Some(chapter.id);
                if ui.selectable_label(is_selected, chapter.title).clicked() {
                    *selected = Some(chapter.id.to_string());
                }
            }
        });
}

pub(crate) fn render_help_fullscreen(app: &mut DictApp, ui: &mut egui::Ui) {
    // Top bar: title + back button
    ui.horizontal(|ui| {
        let back_text = "[←Del] 返回";
        if ui.button(back_text).clicked() || ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
            app.show_help_panel = false;
        }
        ui.heading("📖 帮助文档");
    });
    ui.separator();

    // Two-column layout
    let chapters = app.help_manager.chapters().to_vec();
    let current_chapter = app.selected_help_chapter.clone();

    egui::Panel::left("help_sidebar")
        .resizable(false)
        .min_size(240.0)
        .max_size(240.0)
        .default_size(240.0)
        .show_inside(ui, |ui| {
            // Search box with clear button
            ui.horizontal(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width() - 28.0, super::styles::INPUT_BOX_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.add(
                            egui::TextEdit::singleline(&mut app.help_search_query)
                                .hint_text("🔍 搜索（仅汉字和字母）...")
                                .desired_width(f32::INFINITY),
                        );
                    },
                );

                // 检查ESC键清除（无论焦点状态）
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    app.help_search_query.clear();
                }

                // 清除按钮（仅当有内容时显示）
                if !app.help_search_query.is_empty()
                    && ui.button("×").on_hover_text("清除搜索 (Esc)").clicked()
                {
                    app.help_search_query.clear();
                }
            });

            // 自动清理输入：检查是否有非文字字母字符
            let cleaned = clean_input_text(&app.help_search_query);
            if cleaned != app.help_search_query {
                app.help_search_query = cleaned;
            }

            let query = app.help_search_query.trim().to_string();
            if !query.is_empty() {
                let results = app.help_manager.search(&query);
                ui.label(format!("找到 {} 个章节", results.len()));
                ui.separator();

                egui::ScrollArea::vertical()
                    .id_salt("help_search_results")
                    .show(ui, |ui| {
                        for (chapter, segments) in &results {
                            let is_selected = current_chapter.as_deref() == Some(chapter.id);
                            if ui.selectable_label(is_selected, chapter.title).clicked() {
                                app.selected_help_chapter = Some(chapter.id.to_string());
                            }
                            for (_, seg) in segments.iter().take(2) {
                                let matches = help::find_all_matches(seg, &query);
                                if matches.is_empty() {
                                    ui.label(
                                        egui::RichText::new(*seg)
                                            .small()
                                            .color(egui::Color32::GRAY),
                                    );
                                } else {
                                    ui.horizontal_wrapped(|ui| {
                                        let mut cursor = 0;
                                        for &(start, end) in &matches {
                                            if cursor < start {
                                                ui.label(
                                                    egui::RichText::new(&seg[cursor..start])
                                                        .small()
                                                        .color(egui::Color32::GRAY),
                                                );
                                            }
                                            ui.label(
                                                egui::RichText::new(&seg[start..end])
                                                    .small()
                                                    .strong()
                                                    .color(egui::Color32::from_rgb(255, 165, 0)),
                                            );
                                            cursor = end;
                                        }
                                        if cursor < seg.len() {
                                            ui.label(
                                                egui::RichText::new(&seg[cursor..])
                                                    .small()
                                                    .color(egui::Color32::GRAY),
                                            );
                                        }
                                    });
                                }
                            }
                        }
                    });

                // 仅在搜索词从空变为非空时自动导航到首个结果
                if !app.help_search_auto_navigated
                    && let Some(first) = results.first()
                {
                    app.selected_help_chapter = Some(first.0.id.to_string());
                    app.help_search_auto_navigated = true;
                }
            } else {
                app.help_search_auto_navigated = false;
                ui.separator();
                chapter_sidebar(
                    ui,
                    &chapters,
                    &current_chapter,
                    &mut app.selected_help_chapter,
                );
            }
        });

    // Content fills the remaining central area
    egui::CentralPanel::default().show_inside(ui, |ui| {
        let chapter_id = app.selected_help_chapter.clone();
        let mut nav = help::HelpNav { goto: None };
        if let Some(id) = &chapter_id {
            // 将搜索词传入，供正文高亮使用
            help::set_search_query(&app.help_search_query);

            egui::ScrollArea::vertical()
                .id_salt("help_content")
                .show(ui, |ui| {
                    if let Some(chapter) = app.help_manager.get_chapter(id) {
                        help::reset_table_counter();
                        (chapter.render)(ui, &mut nav);
                    }
                });
        } else {
            ui.label("请从左侧选择帮助章节");
        }
        if let Some(target) = nav.goto {
            app.selected_help_chapter = Some(target.to_string());
        }
    });
}
