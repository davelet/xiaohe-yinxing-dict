use eframe::egui::{self, RichText, ScrollArea, Ui};

use super::{
    BLUE, CODE_BG, CODE_FG, GRAY, HelpNav, ORANGE, RED, next_table_idx, render_hl, render_hl_inline,
};

pub(in crate::help) fn h1(ui: &mut Ui, t: &str) {
    ui.add_space(8.0);
    render_hl(ui, t, |s| RichText::new(s).size(18.0).strong());
    ui.add_space(2.0);
}
pub(in crate::help) fn h2(ui: &mut Ui, t: &str) {
    ui.add_space(6.0);
    render_hl(ui, t, |s| RichText::new(s).size(15.0).strong());
    ui.add_space(2.0);
}
pub(in crate::help) fn h3(ui: &mut Ui, t: &str) {
    ui.add_space(4.0);
    render_hl(ui, t, |s| RichText::new(s).size(13.0).strong());
}
pub(in crate::help) fn h4(ui: &mut Ui, t: &str) {
    ui.add_space(3.0);
    render_hl(ui, t, |s| RichText::new(s).size(12.0).strong());
}

pub(in crate::help) fn p(ui: &mut Ui, t: &str) {
    render_hl(ui, t, |s| RichText::new(s));
}
pub(in crate::help) fn sp(ui: &mut Ui) {
    ui.add_space(4.0);
}
pub(in crate::help) fn hr(ui: &mut Ui) {
    ui.separator();
    ui.add_space(4.0);
}

pub(in crate::help) fn bul(ui: &mut Ui, t: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label("  •");
        render_hl_inline(ui, t, |s| RichText::new(s));
    });
}

pub(in crate::help) fn num(ui: &mut Ui, n: &str, t: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("  {}.", n));
        render_hl_inline(ui, t, |s| RichText::new(s));
    });
}

pub(in crate::help) fn qt(ui: &mut Ui, t: &str) {
    render_hl(ui, &format!("  {}", t), |s| {
        RichText::new(s).italics().color(GRAY)
    });
}

pub(in crate::help) fn red(ui: &mut Ui, t: &str) {
    render_hl(ui, t, |s| RichText::new(s).color(RED));
}
pub(in crate::help) fn blue(ui: &mut Ui, t: &str) {
    render_hl(ui, t, |s| RichText::new(s).color(BLUE));
}

pub(in crate::help) fn code(ui: &mut Ui, t: &str) {
    render_hl(ui, t, |s| {
        RichText::new(s)
            .monospace()
            .background_color(CODE_BG)
            .color(CODE_FG)
    });
}

pub(in crate::help) fn lnk(ui: &mut Ui, nav: &mut HelpNav, text: &str, target: &'static str) {
    if ui.link(text).clicked() {
        nav.goto = Some(target);
    }
}

pub(in crate::help) fn ext_link(ui: &mut Ui, text: &str, url: &str) {
    let label = ui.add(
        egui::Label::new(
            RichText::new(text)
                .color(ui.visuals().hyperlink_color)
                .underline(),
        )
        .sense(egui::Sense::click()),
    );
    if label.clicked() {
        let _ = open::that(url);
    }
}

/// 上一篇 / 下一篇 导航行
pub(in crate::help) fn navrow(
    ui: &mut Ui,
    nav: &mut HelpNav,
    prev: Option<(&str, &'static str)>,
    next: Option<(&str, &'static str)>,
) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if let Some((t, id)) = prev {
            label_hl(ui, "上一篇：");
            lnk(ui, nav, t, id);
        }
        ui.separator();
        if let Some((t, id)) = next {
            label_hl(ui, "下一篇：");
            lnk(ui, nav, t, id);
        }
    });
}

/// 四列元组表格（用于拆分例字等静态数据，避免每帧分配）
pub(in crate::help) fn tbl4(
    ui: &mut Ui,
    col_width: f32,
    headers: &[&str; 4],
    rows: &[(&str, &str, &str, &str)],
) {
    let idx = next_table_idx();
    ScrollArea::horizontal()
        .id_salt(("help_tbl4", idx))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    for hd in headers {
                        ui.add_sized(
                            [col_width, 18.0],
                            egui::Label::new(
                                RichText::new(*hd)
                                    .monospace()
                                    .size(11.0)
                                    .strong()
                                    .color(ORANGE),
                            ),
                        );
                    }
                });
                for (a, b, c, d) in rows {
                    ui.horizontal(|ui| {
                        for cell in [a, b, c, d] {
                            ui.add_sized(
                                [col_width, 18.0],
                                egui::Label::new(RichText::new(*cell).monospace().size(11.0)),
                            );
                        }
                    });
                }
            });
        });
    ui.add_space(4.0);
}

/// 短数据表格（固定列宽，单元格不换行）
pub(in crate::help) fn tbl(ui: &mut Ui, col_width: f32, headers: &[&str], rows: &[&[&str]]) {
    if headers.is_empty() {
        return;
    }
    let idx = next_table_idx();
    ScrollArea::horizontal()
        .id_salt(("help_tbl", idx))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    for hd in headers {
                        ui.add_sized(
                            [col_width, 18.0],
                            egui::Label::new(
                                RichText::new(*hd)
                                    .monospace()
                                    .size(11.0)
                                    .strong()
                                    .color(ORANGE),
                            ),
                        );
                    }
                });
                for row in rows {
                    ui.horizontal(|ui| {
                        for cell in row.iter() {
                            ui.add_sized(
                                [col_width, 18.0],
                                egui::Label::new(RichText::new(*cell).monospace().size(11.0)),
                            );
                        }
                    });
                }
            });
        });
    ui.add_space(4.0);
}

pub(in crate::help) fn label_hl(ui: &mut Ui, t: &str) {
    render_hl(ui, t, |s| RichText::new(s).strong());
}
