use crate::app::{ManagerState, ViewMode};
use crate::dict::SearchableEntry;
use crate::search;
use crate::CopyKind;
use eframe::egui;
use egui::output::OutputCommand;

use super::split_at_range;
use super::styles::ManagerViewStyle;

/// 渲染输入法数据视图
pub fn render_manager_view(
    state: &mut ManagerState,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    current_view: &mut ViewMode,
) {
    // Handle feedback timer
    if state.external_feedback_timer > 0.0 {
        let dt = ctx.input(|i| i.unstable_dt);
        state.external_feedback_timer -= dt;
        if state.external_feedback_timer <= 0.0 {
            state.external_copied_feedback = None;
        }
        ctx.request_repaint();
    }

    render_top_panel(state, ui, current_view);

    ui.separator();

    // Search box
    let search_box_id = egui::Id::new("external_search_box");
    let has_focus = ui.memory(|mem| mem.has_focus(search_box_id));
    if !has_focus {
        let no_widget_focused = ui.memory(|mem| mem.focused().is_none());
        if no_widget_focused {
            ui.memory_mut(|mem| mem.request_focus(search_box_id));
        }
    }

    ui.horizontal(|ui| {
        ui.add_sized(
            [ui.available_width() - 28.0, 28.0],
            egui::TextEdit::singleline(&mut state.external_query)
                .id(search_box_id)
                .hint_text("🔍 在外部词典中搜索文字或编码...")
                .desired_width(f32::INFINITY),
        );
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            state.external_query.clear();
        }
        if !state.external_query.is_empty()
            && ui.button("×").on_hover_text("清除搜索 (Esc)").clicked()
        {
            state.external_query.clear();
        }
    });

    // Perform search
    if !state.external_query.is_empty() {
        let (results, total) = state
            .external_engine
            .search(state.external_query.trim(), None);
        state.external_search_results = results;
        state.external_total_results = total;
    } else {
        state.external_search_results.clear();
        state.external_total_results = 0;
    }

    ui.separator();

    // Main content
    if !state.external_query.is_empty() {
        render_external_search_results(state, ui, ctx);
    } else {
        render_file_management(state, ui);
    }

    ui.separator();
    render_bottom_panel(state, ui);
}

/// 渲染外部词典搜索结果
fn render_external_search_results(
    state: &mut ManagerState,
    ui: &mut egui::Ui,
    ctx: &egui::Context,
) {
    let result_count = state.external_search_results.len();
    ui.label(format!("找到 {} 条结果", result_count));

    // Table header
    ui.horizontal(|ui| {
        let headers = ["序号", "文字", "编码", "操作", ""];
        let widths = [40.0, 180.0, 120.0, 60.0, 60.0];
        for (i, header) in headers.iter().enumerate() {
            let bg = None;
            render_cell(ui, widths[i], 20.0, i != 0, bg, |ui| {
                ui.strong(*header);
            });
        }
    });
    ui.separator();

    // Results
    egui::ScrollArea::vertical()
        .id_salt("external_search_scroll")
        .max_height(ui.available_height())
        .show(ui, |ui| {
            if !state.external_search_results.is_empty() {
                let results = state.external_search_results.clone();
                let entries = state.external_engine.entries();
                for (row_num, (idx, match_kind)) in results.iter().enumerate() {
                    let entry = &entries[*idx];

                    ui.horizontal(|ui| {
                        let widths = [40.0, 180.0, 120.0, 60.0, 60.0];
                        let row_height = 28.0;

                        // Row number
                        render_cell(ui, widths[0], row_height, true, Some(ROW_NUM_BG), |ui| {
                            ui.label(format!("{}", row_num + 1));
                        });

                        // Text with highlight
                        render_cell(
                            ui,
                            widths[1],
                            row_height,
                            false,
                            Some(TEXT_BG),
                            |ui| {
                                egui::ScrollArea::horizontal()
                                    .id_salt(format!("ext_text_scroll_{}", idx))
                                    .show(ui, |ui| {
                                        ui.set_min_height(row_height);
                                        if let search::MatchKind::Text(matched_range) = match_kind
                                        {
                                            let (before, matched, after) = split_at_range(
                                                entry.text(),
                                                matched_range.clone(),
                                            );
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
                            },
                        );

                        // Code
                        render_cell(
                            ui,
                            widths[2],
                            row_height,
                            true,
                            Some(CODE_BG),
                            |ui| {
                                ui.monospace(entry.code());
                            },
                        );

                        // Copy text
                        render_cell(ui, widths[3], row_height, true, None, |ui| {
                            let copied = state.external_copied_feedback.as_ref().is_some_and(
                                |(id, kind)| *id == *idx && *kind == CopyKind::Text,
                            );
                            let label = if copied { "✅文字" } else { "📋文字" };
                            let btn = ui.add_sized([64.0, 24.0], egui::Button::new(label));
                            if btn.clicked() {
                                ctx.output_mut(|o| {
                                    o.commands.push(OutputCommand::CopyText(
                                        entry.text().to_owned(),
                                    ));
                                });
                                state.external_copied_feedback = Some((*idx, CopyKind::Text));
                                state.external_feedback_timer = 2.0;
                            }
                        });

                        // Copy code
                        render_cell(ui, widths[4], row_height, true, None, |ui| {
                            let copied = state.external_copied_feedback.as_ref().is_some_and(
                                |(id, kind)| *id == *idx && *kind == CopyKind::Code,
                            );
                            let label = if copied { "✅编码" } else { "📋编码" };
                            let btn = ui.add_sized([64.0, 24.0], egui::Button::new(label));
                            if btn.clicked() {
                                ctx.output_mut(|o| {
                                    o.commands.push(OutputCommand::CopyText(
                                        entry.code().to_owned(),
                                    ));
                                });
                                state.external_copied_feedback = Some((*idx, CopyKind::Code));
                                state.external_feedback_timer = 2.0;
                            }
                        });
                    });
                }
            } else {
                ui.label("无匹配结果");
            }
        });
}

/// 渲染文件管理区域（搜索框为空时显示）
fn render_file_management(state: &mut ManagerState, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .id_salt("manager_file_scroll")
        .max_height(ui.available_height() - 25.0)
        .show(ui, |ui| {
            render_external_files(ui, state);
            ui.separator();
            render_discovered_files(ui, state);
        });
}

/// 渲染顶部面板
fn render_top_panel(state: &mut ManagerState, ui: &mut egui::Ui, current_view: &mut ViewMode) {
    ui.horizontal(|ui| {
        ui.heading("输入法数据");

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
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

        if ui.button("添加").clicked()
            || (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
        {
            if !state.new_file_path.trim().is_empty() {
                let path = state.new_file_path.clone();
                let name = std::path::Path::new(&path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                state.add_external_dict(path, name);
                state.new_file_path.clear();
            }
        }

        if ui.button("扫描目录").clicked() {
            state.refresh_discovered_files();
        }
    });

    // 状态消息
    if let Some(msg) = &state.status_message {
        ui.colored_label(ManagerViewStyle::success_color(), msg);
    }
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
            .inner_margin(12.0)
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

    let discovered = state.discovered_files.clone();
    let config = state.config.clone();

    for dict_file in &discovered {
        let already_added = config
            .external_dict_files
            .iter()
            .any(|f| f.path == dict_file.path);

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

    if discovered.is_empty() {
        ui.label("未扫描到词典文件");
    }
}

/// 渲染底部面板
fn render_bottom_panel(state: &ManagerState, ui: &mut egui::Ui) {
    ui.spacing_mut().item_spacing.y = 0.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        let total_external = state.external_entries.len();
        if !state.external_query.is_empty() {
            ui.label(format!(
                "找到 {} 条结果 | 外部词典共 {} 条",
                state.external_total_results, total_external
            ));
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
}

fn render_cell<F>(
    ui: &mut egui::Ui,
    width: f32,
    height: f32,
    align_center: bool,
    bg_color: Option<egui::Color32>,
    content: F,
) where
    F: FnOnce(&mut egui::Ui),
{
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    let layout = if align_center {
        egui::Layout::centered_and_justified(egui::Direction::LeftToRight)
    } else {
        egui::Layout::left_to_right(egui::Align::Center).with_main_wrap(false)
    };
    let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(layout));

    if let Some(color) = bg_color {
        child_ui
            .painter()
            .rect_filled(child_ui.max_rect(), 0.0, color);
    }

    content(&mut child_ui);
}

const ROW_NUM_BG: egui::Color32 = egui::Color32::from_rgb(235, 235, 235);
const TEXT_BG: egui::Color32 = egui::Color32::from_rgb(234, 237, 245);
const CODE_BG: egui::Color32 = egui::Color32::from_rgb(234, 245, 237);
