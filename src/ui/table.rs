use eframe::egui;
use egui::output::OutputCommand;

use crate::CopyKind;
use crate::DictApp;
use crate::dict_data::DICT_ENTRIES;
use crate::search;

use super::split_at_range;

const COL_WIDTHS: [f32; 6] = [40.0, 120.0, 100.0, 130.0, 60.0, 60.0];
const HEADER_HEIGHT: f32 = 20.0;
const ROW_HEIGHT: f32 = 28.0;
const ROW_NUM_BG: egui::Color32 = egui::Color32::from_rgb(235, 235, 235);
const TEXT_BG: egui::Color32 = egui::Color32::from_rgb(234, 237, 245);
const CODE_BG: egui::Color32 = egui::Color32::from_rgb(234, 245, 237);

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

    // Draw background in child_ui coordinate space (origin at rect.min)
    if let Some(color) = bg_color {
        child_ui
            .painter()
            .rect_filled(child_ui.max_rect(), 0.0, color);
    }

    content(&mut child_ui);
}

pub(crate) fn render_table(app: &mut DictApp, ui: &mut egui::Ui, ctx: &egui::Context) {
    // Fixed table header (outside scroll area)
    ui.horizontal(|ui| {
        let headers = ["序号", "文字", "编码", "分类", "操作", ""];
        for (i, header) in headers.iter().enumerate() {
            let bg = None;
            render_cell(ui, COL_WIDTHS[i], HEADER_HEIGHT, i != 0, bg, |ui| {
                ui.strong(*header);
            });
        }
    });
    ui.separator();

    // Scrollable data rows
    egui::ScrollArea::vertical()
        .id_salt("table_scroll")
        .max_height(ui.available_height())
        .show(ui, |ui| {
            if !app.search_results.is_empty() {
                let results = app.search_results.clone();
                for (row_num, (idx, match_kind)) in results.iter().enumerate() {
                    let entry = &DICT_ENTRIES[*idx];

                    ui.horizontal(|ui| {
                        // Row number column (with background)
                        render_cell(
                            ui,
                            COL_WIDTHS[0],
                            ROW_HEIGHT,
                            true,
                            Some(ROW_NUM_BG),
                            |ui| {
                                ui.label(format!("{}", row_num + 1));
                            },
                        );

                        // Text column (with highlight)
                        render_cell(ui, COL_WIDTHS[1], ROW_HEIGHT, false, Some(TEXT_BG), |ui| {
                            egui::ScrollArea::horizontal()
                                .id_salt(format!("text_scroll_{}", idx))
                                .show(ui, |ui| {
                                    ui.set_min_height(ROW_HEIGHT);
                                    if let search::MatchKind::Text(matched_range) = match_kind {
                                        let (before, matched, after) =
                                            split_at_range(entry.text, matched_range.clone());
                                        ui.label(&before);
                                        ui.colored_label(
                                            egui::Color32::from_rgb(0, 130, 0),
                                            &matched,
                                        );
                                        ui.monospace(&after);
                                    } else {
                                        ui.label(entry.text);
                                    }
                                });
                        });

                        // Code column
                        render_cell(ui, COL_WIDTHS[2], ROW_HEIGHT, true, Some(CODE_BG), |ui| {
                            ui.monospace(entry.code);
                        });

                        // Category column
                        render_cell(ui, COL_WIDTHS[3], ROW_HEIGHT, true, None, |ui| {
                            ui.label(entry.category.display_name());
                        });

                        // Copy text button
                        render_cell(ui, COL_WIDTHS[4], ROW_HEIGHT, true, None, |ui| {
                            let copied_text = app
                                .copied_feedback
                                .as_ref()
                                .is_some_and(|(id, kind)| *id == *idx && *kind == CopyKind::Text);
                            let btn_label = if copied_text {
                                "✅文字"
                            } else {
                                "📋文字"
                            };
                            let btn_response =
                                ui.add_sized([64.0, 24.0], egui::Button::new(btn_label));
                            if btn_response.clicked() {
                                ctx.output_mut(|o| {
                                    o.commands
                                        .push(OutputCommand::CopyText(entry.text.to_owned()));
                                });
                                app.copied_feedback = Some((*idx, CopyKind::Text));
                                app.feedback_timer = 2.0;
                            }
                        });

                        // Copy code button
                        render_cell(ui, COL_WIDTHS[5], ROW_HEIGHT, true, None, |ui| {
                            let copied_code = app
                                .copied_feedback
                                .as_ref()
                                .is_some_and(|(id, kind)| *id == *idx && *kind == CopyKind::Code);
                            let btn_label = if copied_code {
                                "✅编码"
                            } else {
                                "📋编码"
                            };
                            let btn_response =
                                ui.add_sized([64.0, 24.0], egui::Button::new(btn_label));
                            if btn_response.clicked() {
                                ctx.output_mut(|o| {
                                    o.commands
                                        .push(OutputCommand::CopyText(entry.code.to_owned()));
                                });
                                app.copied_feedback = Some((*idx, CopyKind::Code));
                                app.feedback_timer = 2.0;
                            }
                        });
                    });
                }
            } else {
                ui.label("无匹配结果");
            }
        });
}
