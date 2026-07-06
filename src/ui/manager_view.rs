use crate::CopyKind;
use crate::app::{ManagerState, SortField, SortOrder, ViewMode};
use crate::dict::SearchableEntry;
use crate::search;
use eframe::egui;
use egui::output::OutputCommand;

use super::add_word_dialog;
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
                // Keyboard nav: left arrow to go back（添加新词输入框聚焦时不触发）
                if ui.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
                    let add_word_focused = ui.memory(|mem| {
                        mem.has_focus(egui::Id::new("add_word_text"))
                            || mem.has_focus(egui::Id::new("add_word_code"))
                    });
                    if !add_word_focused {
                        *current_view = ViewMode::Dict;
                    }
                }
                if ui.button("← 默认数据").clicked() {
                    *current_view = ViewMode::Dict;
                }
            });
        });

        // 状态消息
        if let Some(msg) = &state.status_message {
            ui.colored_label(ManagerViewStyle::success_color(), msg);
        }
    });
}

/// 渲染输入法数据视图的底部面板（使用 Panel::bottom 锚定）
pub fn render_manager_bottom_panel(state: &ManagerState, ui: &mut egui::Ui) {
    let status_bg = egui::Color32::from_rgb(245, 245, 250);
    egui::Panel::bottom("manager_status_panel")
        .frame(
            egui::Frame::new()
                .fill(status_bg)
                .inner_margin(egui::Margin::symmetric(8, 4)),
        )
        .show_inside(ui, |ui| {
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
                    let added_paths: std::collections::HashSet<&str> = state
                        .config
                        .external_dict_files
                        .iter()
                        .map(|f| f.path.as_str())
                        .collect();
                    let undiscovered = state
                        .discovered_files
                        .iter()
                        .filter(|f| !added_paths.contains(f.path.as_str()))
                        .count();
                    let total_dicts = state.config.external_dict_files.len() + undiscovered;
                    let enabled_count = state.config.enabled_external_dicts().len();
                    ui.label(format!(
                        "扫描 {} 个词典 | 启用 {} 个 | 外部词典 {} 条 | 扫描文件共 {} 条",
                        total_dicts, enabled_count, total_external, total_entries
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

    // Handle status and add_word timers (need repaint for countdown bar)
    if state.status_timer > 0.0 || state.add_word_timer > 0.0 {
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

    let modifier_key = if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    };

    ui.horizontal(|ui| {
        let response = ui
            .allocate_ui_with_layout(
                egui::vec2(
                    ui.available_width() - 130.0,
                    super::styles::INPUT_BOX_HEIGHT,
                ),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut state.external_query)
                            .id(search_box_id)
                            .hint_text("🔍 在外部词典中搜索文字或编码...")
                            .desired_width(f32::INFINITY)
                            .frame(
                                egui::Frame::default()
                                    .stroke(egui::Stroke::new(1.0, egui::Color32::GRAY)),
                            ),
                    )
                },
            )
            .inner;
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

        let open_dialog = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(format!("➕ 添加新词  {modifier_key}N")).clicked() {
                state.show_add_word_dialog = true;
                state.add_word_dialog_auto_focus = true;
            }
        });
        if open_dialog {
            state.show_add_word_dialog = true;
            state.add_word_dialog_auto_focus = true;
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

    // Main content
    if !state.external_query.is_empty() {
        if let Some((idx, kind)) = render_external_search_results(state, ui, ctx) {
            state.external_copied_feedback = Some((idx, kind));
            state.external_feedback_timer = 2.0;
        }
    } else {
        render_file_management(state, ui);
    }

    // Add word dialog
    if state.show_add_word_dialog {
        add_word_dialog::show_add_word_dialog(state, ctx);
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum FileStatus {
    Enabled,
    Disabled,
    Discovered,
}

struct DictFileItem {
    path: String,
    name: String,
    entry_count: Option<usize>,
    status: FileStatus,
    is_manual: bool,
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

    // Build merged file list
    let added_paths: std::collections::HashSet<&str> = state
        .config
        .external_dict_files
        .iter()
        .map(|f| f.path.as_str())
        .collect();

    let rime_dir = std::path::Path::new(&state.config.rime_user_dir);
    let is_manual_path = |path: &str| -> bool { !std::path::Path::new(path).starts_with(rime_dir) };

    let mut default_items: Vec<DictFileItem> = Vec::new();
    let mut manual_items: Vec<DictFileItem> = Vec::new();

    for f in &state.config.external_dict_files {
        let item = DictFileItem {
            path: f.path.clone(),
            name: f.name.clone(),
            entry_count: f.entry_count,
            status: if f.is_enabled {
                FileStatus::Enabled
            } else {
                FileStatus::Disabled
            },
            is_manual: is_manual_path(&f.path),
        };
        if item.is_manual {
            manual_items.push(item);
        } else {
            default_items.push(item);
        }
    }

    for f in &state.discovered_files {
        if !added_paths.contains(f.path.as_str()) {
            default_items.push(DictFileItem {
                path: f.path.clone(),
                name: f.name.clone(),
                entry_count: f.entry_count,
                status: FileStatus::Discovered,
                is_manual: false,
            });
        }
    }

    let mut all_items: Vec<&DictFileItem> =
        default_items.iter().chain(manual_items.iter()).collect();
    all_items.sort_by(|a, b| {
        let score = |item: &&DictFileItem| -> u8 {
            match item.status {
                FileStatus::Enabled => 0,
                FileStatus::Disabled => 1,
                FileStatus::Discovered => 2,
            }
        };
        score(a).cmp(&score(b))
    });

    let col_widths = [40.0, 150.0, 40.0, 70.0, 90.0, 70.0];
    let row_height = 24.0;

    // Title row + scan button (always visible)
    ui.horizontal(|ui| {
        ui.strong(format!("词典文件（{}）", all_items.len()));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("🔄 重扫默认目录").clicked() {
                state.refresh_discovered_files();
            }
            let has_manual = manual_items
                .iter()
                .any(|item| item.status != FileStatus::Discovered);
            if has_manual && ui.button("🗑 删除全部自定义词典").clicked() {
                state.remove_manual_dicts();
            }
            if ui.link("手动添加词典").clicked() {
                state.show_manual_add = !state.show_manual_add;
                if !state.show_manual_add {
                    state.new_file_path.clear();
                }
            }
        });
    });

    // Inline manual add (visible when triggered)
    if state.show_manual_add {
        let mut close = false;
        let mut do_add = false;
        ui.horizontal(|ui| {
            let input_w = (ui.available_width() - 130.0).max(100.0);
            let response = ui.add(
                egui::TextEdit::singleline(&mut state.new_file_path)
                    .id("add_file_path".into())
                    .hint_text("输入词典文件路径...")
                    .desired_width(input_w)
                    .frame(
                        egui::Frame::default().stroke(egui::Stroke::new(1.0, egui::Color32::GRAY)),
                    ),
            );
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                do_add = true;
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌ 关闭").clicked() {
                    close = true;
                }
                if ui.button("📥 添加").clicked() {
                    do_add = true;
                }
            });
        });
        if do_add && !state.new_file_path.trim().is_empty() {
            let raw_path = state.new_file_path.trim().to_string();
            let path = std::path::Path::new(&raw_path);
            if path.is_dir() {
                // 目录 → 扫描该目录下的 .dict.yaml / .txt 文件逐个添加
                if let Ok(entries) = std::fs::read_dir(path) {
                    let mut count = 0;
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file()
                            && let Some(name) = p.file_name()
                        {
                            let name_str = name.to_string_lossy().to_string();
                            if name_str.ends_with(".dict.yaml") || name_str.ends_with(".txt") {
                                let dict_name =
                                    name_str.replace(".dict.yaml", "").replace(".txt", "");
                                state.add_external_dict(p.to_string_lossy().to_string(), dict_name);
                                count += 1;
                            }
                        }
                    }
                    state.set_status(format!("已从目录添加 {} 个词典文件", count));
                } else {
                    state.set_status(format!("无法读取目录: {}", raw_path));
                }
            } else {
                // 文件 → 直接添加
                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();
                state.add_external_dict(raw_path, name);
            }
            state.new_file_path.clear();
            close = true;
        }
        if close {
            state.show_manual_add = false;
            state.new_file_path.clear();
        }
    } else if !state.show_add_word_dialog
        && ui.input(|i| i.key_pressed(egui::Key::Escape))
        && !state.new_file_path.is_empty()
    {
        state.new_file_path.clear();
    }

    ui.separator();

    if all_items.is_empty() {
        ui.label("暂无词典文件");
        return;
    }

    // Table header — outside ScrollArea, always visible
    ui.horizontal(|ui| {
        render_cell(ui, col_widths[0], row_height, true, None, |ui| {
            ui.strong("序号");
        });
        render_cell(ui, col_widths[1], row_height, false, None, |ui| {
            ui.strong("文件名");
        });
        render_cell(ui, col_widths[2], row_height, true, None, |ui| {
            ui.strong("条目");
        });
        render_cell(ui, col_widths[3], row_height, true, None, |ui| {
            ui.strong("状态");
        });
        render_cell(ui, col_widths[4], row_height, true, None, |ui| {
            ui.strong("操作");
        });
        render_cell(ui, col_widths[5], row_height, true, None, |ui| {
            ui.strong("来源");
        });
        ui.strong("路径");
    });

    // Rows inside scrollable area
    let mut to_toggle: Option<String> = None;
    let mut to_remove: Option<String> = None;
    let mut to_add: Option<(String, String)> = None;

    egui::ScrollArea::vertical()
        .id_salt("manager_file_scroll")
        .max_height(ui.available_height())
        .show(ui, |ui| {
            for (row_num, item) in all_items.iter().enumerate() {
                let (row_bg, status_label, status_color) = match item.status {
                    FileStatus::Enabled => (
                        egui::Color32::from_rgb(240, 250, 240),
                        "已启用",
                        egui::Color32::from_rgb(30, 140, 60),
                    ),
                    FileStatus::Disabled => (
                        egui::Color32::from_rgb(245, 245, 245),
                        "已禁用",
                        egui::Color32::from_rgb(140, 140, 140),
                    ),
                    FileStatus::Discovered => (
                        egui::Color32::from_rgb(255, 255, 255),
                        "未添加",
                        egui::Color32::from_rgb(40, 100, 200),
                    ),
                };

                ui.horizontal(|ui| {
                    render_cell(ui, col_widths[0], row_height, true, Some(row_bg), |ui| {
                        ui.label(format!("{}", row_num + 1));
                    });

                    render_cell(ui, col_widths[1], row_height, false, Some(row_bg), |ui| {
                        ui.label(&item.name);
                    });

                    render_cell(ui, col_widths[2], row_height, true, Some(row_bg), |ui| {
                        if let Some(count) = item.entry_count {
                            ui.label(format!("{}", count));
                        } else {
                            ui.label("-");
                        }
                    });

                    render_cell(ui, col_widths[3], row_height, true, Some(row_bg), |ui| {
                        ui.colored_label(status_color, status_label);
                    });

                    let action_bg = if item.status == FileStatus::Discovered {
                        egui::Color32::from_rgb(245, 250, 255)
                    } else {
                        row_bg
                    };
                    {
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(col_widths[4], row_height),
                            egui::Sense::hover(),
                        );
                        let mut child_ui = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(rect)
                                .layout(egui::Layout::top_down(egui::Align::Center)),
                        );
                        child_ui.painter().rect_filled(rect, 0.0, action_bg);
                        match item.status {
                            FileStatus::Enabled | FileStatus::Disabled => {
                                child_ui.horizontal(|ui| {
                                    ui.add_space((ui.available_width() - 70.0).max(0.0) / 2.0);
                                    let toggle_label = if item.status == FileStatus::Enabled {
                                        "禁用"
                                    } else {
                                        "启用"
                                    };
                                    if ui.button(toggle_label).clicked() {
                                        to_toggle = Some(item.path.clone());
                                    }
                                    if ui.button("删除").clicked() {
                                        to_remove = Some(item.path.clone());
                                    }
                                });
                            }
                            FileStatus::Discovered => {
                                child_ui.horizontal(|ui| {
                                    ui.add_space((ui.available_width() - 30.0).max(0.0) / 2.0);
                                    if ui.button("添加").clicked() {
                                        to_add = Some((item.path.clone(), item.name.clone()));
                                    }
                                });
                            }
                        }
                    }

                    render_cell(ui, col_widths[5], row_height, true, Some(row_bg), |ui| {
                        let source = if item.is_manual {
                            "自定义"
                        } else {
                            "默认"
                        };
                        ui.label(source);
                    });

                    let path_bg = egui::Color32::from_rgb(248, 248, 250);
                    let avail_w = ui.available_width().max(100.0);
                    let (rect, _) = ui
                        .allocate_exact_size(egui::vec2(avail_w, row_height), egui::Sense::hover());
                    let mut child_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect)
                            .layout(egui::Layout::left_to_right(egui::Align::Center)),
                    );
                    child_ui.painter().rect_filled(rect, 0.0, path_bg);
                    egui::ScrollArea::horizontal()
                        .id_salt(format!("path_scroll_{}", item.path))
                        .show(&mut child_ui, |ui| {
                            ui.set_min_height(row_height);
                            ui.monospace(&item.path);
                        });
                });
            }
        });

    // Apply actions after scroll area
    if let Some(path) = to_toggle {
        state.toggle_external_dict(&path);
    }
    if let Some(path) = to_remove {
        state.remove_external_dict(&path);
    }
    if let Some((path, name)) = to_add {
        state.add_external_dict(path, name);
    }
}
