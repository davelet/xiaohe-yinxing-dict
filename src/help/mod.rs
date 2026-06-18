use std::cell::Cell;
use std::collections::HashMap;

use eframe::egui::{self, Color32, RichText, ScrollArea, Ui};

// 主题色
const ORANGE: Color32 = Color32::from_rgb(200, 160, 80);
const RED: Color32 = Color32::from_rgb(217, 83, 79);
const BLUE: Color32 = Color32::from_rgb(65, 131, 196);
const GRAY: Color32 = Color32::from_rgb(100, 100, 100);
const CODE_BG: Color32 = Color32::from_gray(40);
const CODE_FG: Color32 = Color32::from_rgb(200, 200, 200);

// 帮助页面内表格计数器：每个章节渲染前重置，tbl/tbl4 调用时自增，
// 用作 ScrollArea 的 id_salt，保证同页多表 id 唯一且跨帧稳定。
thread_local! {
    static TABLE_COUNTER: Cell<usize> = const { Cell::new(0) };
}

/// 在渲染一个章节内容前调用，重置表格计数器。
pub fn reset_table_counter() {
    TABLE_COUNTER.with(|c| c.set(0));
}

fn next_table_idx() -> usize {
    TABLE_COUNTER.with(|c| {
        let v = c.get();
        c.set(v + 1);
        v
    })
}

/// 帮助文档内导航请求（点击章节内链接时产生）
pub struct HelpNav {
    pub goto: Option<&'static str>,
}

/// 帮助文档章节
#[derive(Debug, Clone, Copy)]
pub struct HelpChapter {
    pub id: &'static str,
    pub title: &'static str,
    pub parent_id: Option<&'static str>,
    pub render: fn(&mut Ui, &mut HelpNav),
}

/// 帮助文档管理器
pub struct HelpManager {
    chapters: Vec<HelpChapter>,
    chapter_map: HashMap<&'static str, usize>,
}

impl Default for HelpManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HelpManager {
    pub fn new() -> Self {
        let chapters = vec![
            HelpChapter {
                id: "readme",
                title: "小鹤音形帮助文档",
                parent_id: None,
                render: render_readme,
            },
            HelpChapter {
                id: "xh",
                title: "1 入门概述",
                parent_id: Some("readme"),
                render: render_xh,
            },
            HelpChapter {
                id: "up",
                title: "1.1 双拼",
                parent_id: Some("xh"),
                render: render_up,
            },
            HelpChapter {
                id: "ux",
                title: "1.2 双形（鹤形）",
                parent_id: Some("xh"),
                render: render_ux,
            },
            HelpChapter {
                id: "gz",
                title: "1.2.1 规则",
                parent_id: Some("ux"),
                render: render_gz,
            },
            HelpChapter {
                id: "zg",
                title: "1.2.2 字根",
                parent_id: Some("ux"),
                render: render_zg,
            },
            HelpChapter {
                id: "yy",
                title: "2 应用",
                parent_id: Some("readme"),
                render: render_yy,
            },
            HelpChapter {
                id: "jm",
                title: "2.1 简码",
                parent_id: Some("yy"),
                render: render_jm,
            },
            HelpChapter {
                id: "fh",
                title: "2.2 符号",
                parent_id: Some("yy"),
                render: render_fh,
            },
            HelpChapter {
                id: "pc",
                title: "2.3 Win版",
                parent_id: Some("yy"),
                render: render_pc,
            },
            HelpChapter {
                id: "sj",
                title: "2.4 安卓版",
                parent_id: Some("yy"),
                render: render_sj,
            },
            HelpChapter {
                id: "gj",
                title: "2.5 挂接",
                parent_id: Some("yy"),
                render: render_gj,
            },
            HelpChapter {
                id: "wv",
                title: "3 相关文章",
                parent_id: Some("readme"),
                render: render_wv,
            },
            HelpChapter {
                id: "wt",
                title: "4 常见问题",
                parent_id: Some("readme"),
                render: render_wt,
            },
            HelpChapter {
                id: "vy",
                title: "5 学习指引",
                parent_id: Some("readme"),
                render: render_vy,
            },
            HelpChapter {
                id: "gy",
                title: "6 关于小鹤",
                parent_id: Some("readme"),
                render: render_gy,
            },
        ];

        let mut chapter_map = HashMap::new();
        for (i, chapter) in chapters.iter().enumerate() {
            chapter_map.insert(chapter.id, i);
        }

        Self {
            chapters,
            chapter_map,
        }
    }

    pub fn chapters(&self) -> &[HelpChapter] {
        &self.chapters
    }

    pub fn get_chapter(&self, id: &str) -> Option<&HelpChapter> {
        self.chapter_map.get(id).map(|&i| &self.chapters[i])
    }

    pub fn top_level_chapters(&self) -> Vec<&HelpChapter> {
        self.chapters
            .iter()
            .filter(|c| c.parent_id.is_none())
            .collect()
    }

    pub fn child_chapters(&self, parent_id: &str) -> Vec<&HelpChapter> {
        self.chapters
            .iter()
            .filter(|c| c.parent_id == Some(parent_id))
            .collect()
    }
}

// ──────────────────────────── 渲染辅助函数 ────────────────────────────

fn h1(ui: &mut Ui, t: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(t).size(18.0).strong());
    ui.add_space(2.0);
}
fn h2(ui: &mut Ui, t: &str) {
    ui.add_space(6.0);
    ui.label(RichText::new(t).size(15.0).strong());
    ui.add_space(2.0);
}
fn h3(ui: &mut Ui, t: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(t).size(13.0).strong());
}
fn h4(ui: &mut Ui, t: &str) {
    ui.add_space(3.0);
    ui.label(RichText::new(t).size(12.0).strong());
}

fn p(ui: &mut Ui, t: &str) {
    ui.label(t);
}
fn sp(ui: &mut Ui) {
    ui.add_space(4.0);
}
fn hr(ui: &mut Ui) {
    ui.separator();
    ui.add_space(4.0);
}

fn bul(ui: &mut Ui, t: &str) {
    ui.horizontal(|ui| {
        ui.label("  •");
        ui.label(t);
    });
}

fn num(ui: &mut Ui, n: &str, t: &str) {
    ui.horizontal(|ui| {
        ui.label(format!("  {}.", n));
        ui.label(t);
    });
}

fn qt(ui: &mut Ui, t: &str) {
    ui.colored_label(GRAY, RichText::new(format!("  {}", t)).italics());
}

fn red(ui: &mut Ui, t: &str) {
    ui.colored_label(RED, t);
}
fn blue(ui: &mut Ui, t: &str) {
    ui.colored_label(BLUE, t);
}

fn code(ui: &mut Ui, t: &str) {
    ui.label(
        RichText::new(t)
            .monospace()
            .background_color(CODE_BG)
            .color(CODE_FG),
    );
}

fn img(ui: &mut Ui, t: &str) {
    ui.colored_label(GRAY, format!("[图片：{}]", t));
}

fn lnk(ui: &mut Ui, nav: &mut HelpNav, text: &str, target: &'static str) {
    if ui.link(text).clicked() {
        nav.goto = Some(target);
    }
}

/// 上一篇 / 下一篇 导航行
fn navrow(
    ui: &mut Ui,
    nav: &mut HelpNav,
    prev: Option<(&str, &'static str)>,
    next: Option<(&str, &'static str)>,
) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if let Some((t, id)) = prev {
            ui.label("上一篇：");
            lnk(ui, nav, t, id);
        }
        ui.separator();
        if let Some((t, id)) = next {
            ui.label("下一篇：");
            lnk(ui, nav, t, id);
        }
    });
}

/// 四列元组表格（用于拆分例字等静态数据，避免每帧分配）
fn tbl4(ui: &mut Ui, col_width: f32, headers: &[&str; 4], rows: &[(&str, &str, &str, &str)]) {
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
fn tbl(ui: &mut Ui, col_width: f32, headers: &[&str], rows: &[&[&str]]) {
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

mod renderers;
use renderers::*;
