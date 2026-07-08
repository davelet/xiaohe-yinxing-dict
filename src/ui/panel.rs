use crate::DictApp;
use crate::dict::Category;
use crate::dict_data::DICT_ENTRIES;
use eframe::egui;

pub(crate) fn render_top_panel(app: &mut DictApp, ui: &mut egui::Ui, _ctx: &egui::Context) {
    egui::Panel::top("header_panel").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("小鹤音形词典");

            // Help button with image tooltip
            let help_btn = ui.button("⌨键位图");
            if help_btn.clicked() {
                app.show_help_image = !app.show_help_image;
            }

            if app.show_help_image
                && let Some(texture) = &app.help_image
            {
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
                            let scale = (max_w / img_size.x).min(max_h / img_size.y).min(1.0);
                            ui.image((texture.id(), img_size * scale));
                        });
                    });

                if !help_btn.hovered() && !area_response.response.hovered() {
                    app.show_help_image = false;
                }
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

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Keyboard nav for category cycling
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

                if ui.button("→ 输入法数据").clicked() {
                    app.current_view = crate::app::ViewMode::Manager;
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

pub(crate) fn render_bottom_panel(app: &DictApp, ui: &mut egui::Ui) {
    egui::Panel::bottom("status_panel").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            let total = DICT_ENTRIES.len();
            let cat_label = app
                .selected_category
                .map(|c| c.display_name())
                .unwrap_or("全部");

            let (display_count, max_note) = if app.query.trim().is_empty() {
                let count = app.engine.count_by_category(app.selected_category);
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

                if let crate::update::UpdateState::Failed(ref err) =
                    *app.update_state.lock().unwrap_or_else(|e| e.into_inner())
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
