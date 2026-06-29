use crate::CopyKind;
use crate::app::{ManagerState, SortField, SortOrder, ViewMode};
use crate::dict::SearchableEntry;
use crate::search;
use eframe::egui;
use egui::output::OutputCommand;

use super::common::{CODE_BG, ROW_NUM_BG, TEXT_BG, render_cell};
use super::split_at_range;
use super::styles::ManagerViewStyle;

/// 渲染输入法数据视图的顶部面板（使用 Panel::top 锚定）
pub fn render_manager_top_panel(
    state: &mut ManagerState,
    ui: &mut egui::Ui,
    current_view: &mut ViewMode,
) {
    egui::Panel::top("manager_top_panel").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading("输入法数据");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Keyboard nav: left arrow to go back
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    *current_view = ViewMode::Dict;
                }
                if ui.button("📖 默认数据").clicked() {
                    *current_view = ViewMode::Dict;
                }
            });
        });

        // 添加文件区域
        ui.horizontal(|ui| {
            ui.label("添加文件路径:");
            let response = ui.add_sized(
                [400.0, 30.0],
                egui::TextEdit::singleline(&mut state.new_file_path)
                    .hint_text("输入词典文件路径..."),
            );

            if (ui.button("添加").clicked()
                || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))))
                && !state.new_file_path.trim().is_empty()
            {
                let path = state.new_file_path.clone();
                let name = std::path::Path::new(&path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                state.add_external_dict(path, name);
                state.new_file_path.clear();
            }

            if ui.button("扫描目录").clicked() {
                state.refresh_discovered_files();
            }
        });

        // 状态消息
        if let Some(msg) = &state.status_message {
            ui.colored_label(ManagerViewStyle::success_color(), msg);
        }

        ui.separator();
    });
}

/// 渲染输入法数据视图的底部面板（使用 Panel::bottom 锚定）
pub fn render_manager_bottom_panel(state: &ManagerState, ui: &mut egui::Ui) {
    egui::Panel::bottom("manager_status_panel").show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            let total_external = state.external_entries.len();
            if !state.external_query.is_empty() {
                let displayed = state.external_search_results.len();
                let total = state.external_total_results;
                if displayed < total {
                    ui.label(format!(
                        "找到 {} 条结果（当前显示 {} 条）| 外部词典共 {} 条",
                        total, displayed, total_external
                    ));
                } else {
                    ui.label(format!(
                        "找到 {} 条结果 | 外部词典共 {} 条",
                        total, total_external
                    ));
                }
            } else {
                let total_entries: usize = state
                    .discovered_files
                    .iter()
                    .filter_map(|f| f.entry_count)
                    .sum();
                ui.label(format!(
                    "外部词典 {} 条 | 扫描文件共 {} 条",
                    total_external, total_entries
                ));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("v{}", env!("CARGO_PKG_VERSION")));
            });
        });
    });
}

/// 渲染输入法数据视图的中央内容区域
pub fn render_manager_view(state: &mut ManagerState, ui: &mut egui::Ui, ctx: &egui::Context) {
    // Handle feedback timer
    if state.external_feedback_timer > 0.0 {
        let dt = ctx.input(|i| i.unstable_dt);
        state.external_feedback_timer -= dt;
        if state.external_feedback_timer <= 0.0 {
            state.external_copied_feedback = None;
        }
        ctx.request_repaint();
    }

    // Handle status message timer
    if state.status_timer > 0.0 {
        let dt = ctx.input(|i| i.unstable_dt);
        state.tick(dt);
        ctx.request_repaint();
    }

    // Search box — auto-focus only on the first frame after entering Manager view
    let search_box_id = egui::Id::new("external_search_box");
    if state.search_auto_focus {
        ui.memory_mut(|mem| mem.request_focus(search_box_id));
        state.search_auto_focus = false;
    }

    let mut query_changed = false;

    ui.horizontal(|ui| {
        let response = ui.add_sized(
            [ui.available_width() - 28.0, 28.0],
            egui::TextEdit::singleline(&mut state.external_query)
                .id(search_box_id)
                .hint_text("🔍 在外部词典中搜索文字或编码...")
                .desired_width(f32::INFINITY),
        );
        if response.changed() {
            query_changed = true;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) && !state.external_query.is_empty() {
            state.external_query.clear();
            query_changed = true;
        }
        if !state.external_query.is_empty()
            && ui.button("×").on_hover_text("清除搜索 (Esc)").clicked()
        {
            state.external_query.clear();
            query_changed = true;
        }
    });

    // Perform search only when query changed
    if query_changed {
        state.search_dirty = true;
    }
    if state.search_dirty {
        state.search_dirty = false;
        if !state.external_query.is_empty() {
            let (results, total) = state
                .external_engine
                .search(state.external_query.trim(), None);
            state.external_search_results = results;
            state.external_total_results = total;
            // 重新应用当前排序
            if state.sort_field != SortField::Default {
                state.sort_search_results();
            }
        } else {
            state.external_search_results.clear();
            state.external_total_results = 0;
        }
    }

    ui.separator();

    // Main content
    if !state.external_query.is_empty() {
        if let Some((idx, kind)) = render_external_search_results(state, ui, ctx) {
            state.external_copied_feedback = Some((idx, kind));
            state.external_feedback_timer = 2.0;
        }
    } else {
        render_file_management(state, ui);
    }
}

/// 渲染外部词典搜索结果，返回复制操作（如有）
fn render_external_search_results(
    state: &mut ManagerState,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
) -> Option<(usize, CopyKind)> {
    let result_count = state.external_search_results.len();
    ui.label(format!("找到 {} 条结果", result_count));

    // Table header with sortable columns
    let mut sort_clicked: Option<SortField> = None;

    ui.horizontal(|ui| {
        let widths = [40.0, 150.0, 100.0, 100.0, 60.0, 60.0];

        // 序号 column (not sortable)
        render_cell(ui, widths[0], 20.0, true, None, |ui| {
            ui.strong("序号");
        });

        // 文字 column (sortable)
        render_cell(ui, widths[1], 20.0, false, None, |ui| {
            let sort_indicator = match state.sort_field {
                SortField::Text => {
                    if state.sort_order == SortOrder::Ascending {
                        " ▲"
                    } else {
                        " ▼"
                    }
                }
                _ => "",
            };
            if ui.strong(format!("文字{}", sort_indicator)).clicked() {
                sort_clicked = Some(SortField::Text);
            }
        });

        // 编码 column (sortable)
        render_cell(ui, widths[2], 20.0, true, None, |ui| {
            let sort_indicator = match state.sort_field {
                SortField::Code => {
                    if state.sort_order == SortOrder::Ascending {
                        " ▲"
                    } else {
                        " ▼"
                    }
                }
                _ => "",
            };
            if ui.strong(format!("编码{}", sort_indicator)).clicked() {
                sort_clicked = Some(SortField::Code);
            }
        });

        // 来源 column (not sortable)
        render_cell(ui, widths[3], 20.0, true, None, |ui| {
            ui.strong("来源");
        });

        // 操作 columns (not sortable)
        render_cell(ui, widths[4], 20.0, true, None, |ui| {
            ui.strong("操作");
        });
        render_cell(ui, widths[5], 20.0, true, None, |ui| {
            ui.strong("");
        });
    });

    // Handle sort click
    if let Some(field) = sort_clicked {
        if state.sort_field == field {
            // Toggle sort order
            state.sort_order = match state.sort_order {
                SortOrder::Ascending => SortOrder::Descending,
                SortOrder::Descending => SortOrder::Ascending,
            };
        } else {
            state.sort_field = field;
            state.sort_order = SortOrder::Ascending;
        }
        state.sort_search_results();
    }

    ui.separator();

    let mut copy_action: Option<(usize, CopyKind)> = None;

    // Results
    egui::ScrollArea::vertical()
        .id_salt("external_search_scroll")
        .max_height(ui.available_height())
        .show(ui, |ui| {
            if !state.external_search_results.is_empty() {
                let entries = state.external_engine.entries();
                for (row_num, (idx, match_kind)) in state.external_search_results.iter().enumerate()
                {
                    let entry = &entries[*idx];

                    ui.horizontal(|ui| {
                        let widths = [40.0, 150.0, 100.0, 100.0, 60.0, 60.0];
                        let row_height = 28.0;

                        // Row number
                        render_cell(ui, widths[0], row_height, true, Some(ROW_NUM_BG), |ui| {
                            ui.label(format!("{}", row_num + 1));
                        });

                        // Text with highlight
                        render_cell(ui, widths[1], row_height, false, Some(TEXT_BG), |ui| {
                            egui::ScrollArea::horizontal()
                                .id_salt(format!("ext_text_scroll_{}", idx))
                                .show(ui, |ui| {
                                    ui.set_min_height(row_height);
                                    if let search::MatchKind::Text(matched_range) = match_kind {
                                        let (before, matched, after) =
                                            split_at_range(entry.text(), matched_range.clone());
                                        ui.label(&before);
                                        ui.colored_label(
                                            egui::Color32::from_rgb(0, 130, 0),
                                            &matched,
                                        );
                                        ui.monospace(&after);
                                    } else {
                                        ui.label(entry.text());
                                    }
                                });
                        });

                        // Code
                        render_cell(ui, widths[2], row_height, true, Some(CODE_BG), |ui| {
                            ui.monospace(entry.code());
                        });

                        // Source
                        render_cell(ui, widths[3], row_height, true, None, |ui| {
                            ui.label(entry.source());
                        });

                        // Copy text
                        render_cell(ui, widths[4], row_height, true, None, |ui| {
                            let copied = state
                                .external_copied_feedback
                                .as_ref()
                                .is_some_and(|(id, kind)| *id == *idx && *kind == CopyKind::Text);
                            let label = if copied { "✅文字" } else { "📋文字" };
                            let btn = ui.add_sized([64.0, 24.0], egui::Button::new(label));
                            if btn.clicked() {
                                ctx.output_mut(|o| {
                                    o.commands
                                        .push(OutputCommand::CopyText(entry.text().to_owned()));
                                });
                                copy_action = Some((*idx, CopyKind::Text));
                            }
                        });

                        // Copy code
                        render_cell(ui, widths[5], row_height, true, None, |ui| {
                            let copied = state
                                .external_copied_feedback
                                .as_ref()
                                .is_some_and(|(id, kind)| *id == *idx && *kind == CopyKind::Code);
                            let label = if copied { "✅编码" } else { "📋编码" };
                            let btn = ui.add_sized([64.0, 24.0], egui::Button::new(label));
                            if btn.clicked() {
                                ctx.output_mut(|o| {
                                    o.commands
                                        .push(OutputCommand::CopyText(entry.code().to_owned()));
                                });
                                copy_action = Some((*idx, CopyKind::Code));
                            }
                        });
                    });
                }
            } else {
                ui.label("无匹配结果");
            }
        });

    copy_action
}

/// 渲染文件管理区域（搜索框为空时显示）
fn render_file_management(state: &mut ManagerState, ui: &mut egui::Ui) {
    // 显示初始加载失败的文件警告
    if !state.load_errors.is_empty() {
        let errors = state.load_errors.clone();
        egui::Frame::new()
            .fill(egui::Color32::from_rgb(255, 240, 240))
            .corner_radius(4.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.colored_label(
                    egui::Color32::from_rgb(180, 0, 0),
                    "以下词典文件加载失败（文件可能已被移动或删除）：",
                );
                for path in &errors {
                    ui.monospace(path);
                }
                if ui.button("移除所有失败项").clicked() {
                    for path in &errors {
                        state.config.remove_external_dict(path);
                    }
                    let _ = state.config.save();
                    state.load_errors.clear();
                    state.reload_external_dicts();
                }
            });
        ui.separator();
    }

    egui::ScrollArea::vertical()
        .id_salt("manager_file_scroll")
        .max_height(ui.available_height() - 25.0)
        .show(ui, |ui| {
            render_external_files(ui, state);
            ui.separator();
            render_discovered_files(ui, state);
        });
}

/// 渲染外部词典文件列表
fn render_external_files(ui: &mut egui::Ui, state: &mut ManagerState) {
    ui.label("已添加的外部词典:");

    let mut to_remove = None;
    let mut to_toggle = None;

    let dict_files: Vec<_> = state.config.external_dict_files.iter().collect();

    for dict_file in &dict_files {
        let card_bg = ManagerViewStyle::card_background();

        egui::Frame::new()
            .fill(card_bg)
            .corner_radius(8.0)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let mut enabled = dict_file.is_enabled;
                    if ui.checkbox(&mut enabled, &dict_file.name).changed() {
                        to_toggle = Some(dict_file.path.clone());
                    }

                    ui.label(&dict_file.path);

                    if let Some(count) = dict_file.entry_count {
                        ui.label(format!("({} 条)", count));
                    }

                    if ui.button("移除").clicked() {
                        to_remove = Some(dict_file.path.clone());
                    }
                });
            });
    }

    if let Some(path) = to_toggle {
        state.toggle_external_dict(&path);
    }

    if let Some(path) = to_remove {
        state.remove_external_dict(&path);
    }

    if state.config.external_dict_files.is_empty() {
        ui.label("暂无外部词典文件");
    }
}

/// 渲染发现的文件列表
fn render_discovered_files(ui: &mut egui::Ui, state: &mut ManagerState) {
    ui.label("扫描到的词典文件:");

    let mut to_add = None;

    let added_paths: std::collections::HashSet<&str> = state
        .config
        .external_dict_files
        .iter()
        .map(|f| f.path.as_str())
        .collect();

    for dict_file in &state.discovered_files {
        let already_added = added_paths.contains(dict_file.path.as_str());

        ui.horizontal(|ui| {
            ui.label(&dict_file.name);
            ui.label(&dict_file.path);

            if already_added {
                ui.label("已添加");
            } else if ui.button("添加").clicked() {
                to_add = Some((dict_file.path.clone(), dict_file.name.clone()));
            }
        });
    }

    if let Some((path, name)) = to_add {
        state.add_external_dict(path, name);
    }

    if state.discovered_files.is_empty() {
        ui.label("未扫描到词典文件");
    }
}
