use crate::DictApp;
use crate::dict::Category;
use crate::dict_data::DICT_ENTRIES;
use eframe::egui;

pub(crate) fn render_top_panel(app: &mut DictApp, ui: &mut egui::Ui, _ctx: &egui::Context) {
    egui::Panel::top("header_panel").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("小鹤音形词典");

            // Help button with image (click to toggle; shown in a Window so it
            // has no fragile manual hover detection / flicker)
            let help_btn = ui.button("⌨键位图");
            if help_btn.clicked() {
                app.show_help_image = !app.show_help_image;
            }

            if app.show_help_image {
                if app.help_image_shown_at.is_none() {
                    app.help_image_shown_at = Some(std::time::Instant::now());
                }
                let resp = egui::Window::new("键位图")
                    .resizable(false)
                    .collapsible(false)
                    .title_bar(false)
                    .fixed_pos(help_btn.rect.left_bottom())
                    .show(ui.ctx(), |ui| {
                        if let Some(texture) = &app.help_image {
                            let max_w = 700.0;
                            let max_h = 550.0;
                            let [iw, ih] = texture.size();
                            let img_size = egui::vec2(iw as f32, ih as f32);
                            let scale = (max_w / img_size.x).min(max_h / img_size.y).min(1.0);
                            ui.image((texture.id(), img_size * scale));
                        }
                    });
                if let Some(inner) = resp {
                    let elapsed = app
                        .help_image_shown_at
                        .map(|t| t.elapsed())
                        .unwrap_or_default();
                    if inner.response.clicked_elsewhere() && elapsed.as_millis() >= 200 {
                        app.show_help_image = false;
                        app.help_image_shown_at = None;
                    }
                }
            } else {
                app.help_image_shown_at = None;
            }

            // Help documentation button
            let help_doc_btn = ui.button("📖 帮助文档[F1]");
            if help_doc_btn.clicked() {
                app.show_help_panel = !app.show_help_panel;
                if app.show_help_panel && app.selected_help_chapter.is_none() {
                    app.selected_help_chapter = Some("readme".to_string());
                }
            }

            // About this software button
            if ui.button("❓关于").on_hover_text("关于本软件").clicked() {
                app.show_about_dialog = true;
            }

            // AI 对话 toggle button
            let ai_btn_label = format!(
                "✨ AI 助手[{}+,]",
                if cfg!(target_os = "macos") {
                    "cmd"
                } else {
                    "ctrl"
                }
            );
            let ai_btn = ui.add(
                egui::Button::new(ai_btn_label)
                    .fill(egui::Color32::from_rgb(207, 228, 235))
                    .selected(app.chat.show_viewport),
            );
            if ai_btn.clicked() {
                app.chat.show_viewport = !app.chat.show_viewport;
                app.current_view = if app.chat.show_viewport {
                    crate::types::ViewMode::Chat
                } else {
                    crate::types::ViewMode::Dict
                };
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Keyboard nav for category cycling (仅词典视图，AI 对话时不监听方向键)
                if !app.chat.show_viewport {
                    let k_down = ui.input(|i| i.key_pressed(egui::Key::ArrowDown));
                    let k_up = ui.input(|i| i.key_pressed(egui::Key::ArrowUp));
                    if k_down || k_up {
                        let all_cats: Vec<Option<Category>> = std::iter::once(None)
                            .chain(app.categories.iter().copied().map(Some))
                            .collect();
                        let curr = all_cats
                            .iter()
                            .position(|&c| c == app.selected_category)
                            .unwrap_or(0);
                        let next = if k_down {
                            (curr + 1) % all_cats.len()
                        } else {
                            (curr + all_cats.len() - 1) % all_cats.len()
                        };
                        app.selected_category = all_cats[next];
                    }
                }

                let rime_label = if cfg!(target_os = "macos") {
                    "→ Rime数据 (Squirrel)"
                } else if cfg!(target_os = "windows") {
                    "→ Rime数据 (小狼毫)"
                } else {
                    "→ Rime数据"
                };
                if ui
                    .add(egui::Button::new(rime_label).fill(egui::Color32::from_rgb(220, 210, 240)))
                    .clicked()
                {
                    app.current_view = crate::types::ViewMode::Manager;
                    app.manager.search_auto_focus = true;
                    // 检查文件变更并自动重载
                    app.manager.check_and_reload_changed_files();
                }

                // ComboBox
                ui.style_mut().spacing.combo_height = 550.0;
                egui::ComboBox::from_id_salt("category_combo")
                    .selected_text(
                        app.selected_category
                            .map(|c| c.display_name())
                            .unwrap_or("全部"),
                    )
                    .width(200.0)
                    .height(550.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut app.selected_category, None::<Category>, "全部");
                        for &cat in &app.categories {
                            ui.selectable_value(
                                &mut app.selected_category,
                                Some(cat),
                                cat.display_name(),
                            );
                        }
                    });
                ui.label("分类筛选");
            });
        });
    });
}

pub(crate) fn render_bottom_panel(app: &mut DictApp, ui: &mut egui::Ui) {
    egui::Panel::bottom("status_panel").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            let total = DICT_ENTRIES.len();
            let cat_label = app
                .selected_category
                .map(|c| c.display_name())
                .unwrap_or("全部");

            let (display_count, max_note) = if app.query.trim().is_empty() {
                let count = match app.cached_category_count {
                    Some((cat, n)) if cat == app.selected_category => n,
                    _ => {
                        let n = app.engine.count_by_category(app.selected_category);
                        app.cached_category_count = Some((app.selected_category, n));
                        n
                    }
                };
                let note = if count > 100 {
                    " (最多显示100条)"
                } else {
                    ""
                };
                (count, note)
            } else {
                let note = if app.total_results > 100 {
                    " (最多显示100条)"
                } else {
                    ""
                };
                (app.total_results, note)
            };

            ui.label(format!(
                "找到 {} 条结果{} | 词典共 {} 条 | 分类: {}",
                display_count, max_note, total, cat_label
            ));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let version = env!("CARGO_PKG_VERSION");
                let version_label = format!("v{}", version);

                if let crate::update::UpdateState::Failed(ref err) = *app
                    .update_ui
                    .state
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                {
                    let err_clone = err.clone();
                    ui.label(
                        egui::RichText::new("更新失败").color(egui::Color32::from_rgb(220, 50, 50)),
                    )
                    .on_hover_text(err_clone);
                    ui.label(&version_label);
                } else {
                    ui.label(&version_label);
                }
            });
        });
    });
}
