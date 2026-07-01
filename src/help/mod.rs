use std::cell::{Cell, RefCell};
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

// 搜索高亮查询词，渲染前由 ui/help.rs 设置
thread_local! {
    static SEARCH_QUERY: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub(crate) fn set_search_query(query: &str) {
    let q = query.trim().to_string();
    SEARCH_QUERY.with(|sq| {
        *sq.borrow_mut() = if q.is_empty() { None } else { Some(q) };
    });
}

pub(crate) fn get_search_query() -> Option<String> {
    SEARCH_QUERY.with(|sq| sq.borrow().clone())
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
    /// 章节全文检索用句子数组（每个元素是一个完整句子）
    pub search_text: &'static [&'static str],
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
                search_text: README_SEARCH_TEXT,
            },
            HelpChapter {
                id: "xh",
                title: "1 入门概述",
                parent_id: Some("readme"),
                render: render_xh,
                search_text: XH_SEARCH_TEXT,
            },
            HelpChapter {
                id: "up",
                title: "1.1 双拼",
                parent_id: Some("xh"),
                render: render_up,
                search_text: UP_SEARCH_TEXT,
            },
            HelpChapter {
                id: "ux",
                title: "1.2 双形（鹤形）",
                parent_id: Some("xh"),
                render: render_ux,
                search_text: UX_SEARCH_TEXT,
            },
            HelpChapter {
                id: "gz",
                title: "1.2.1 规则",
                parent_id: Some("ux"),
                render: render_gz,
                search_text: GZ_SEARCH_TEXT,
            },
            HelpChapter {
                id: "zg",
                title: "1.2.2 字根",
                parent_id: Some("ux"),
                render: render_zg,
                search_text: ZG_SEARCH_TEXT,
            },
            HelpChapter {
                id: "yy",
                title: "2 输入法应用",
                parent_id: Some("readme"),
                render: render_yy,
                search_text: YY_SEARCH_TEXT,
            },
            HelpChapter {
                id: "jm",
                title: "2.1 简码",
                parent_id: Some("yy"),
                render: render_jm,
                search_text: JM_SEARCH_TEXT,
            },
            HelpChapter {
                id: "fh",
                title: "2.2 符号编码",
                parent_id: Some("yy"),
                render: render_fh,
                search_text: FH_SEARCH_TEXT,
            },
            HelpChapter {
                id: "pc",
                title: "2.3 Win版 指南",
                parent_id: Some("yy"),
                render: render_pc,
                search_text: PC_SEARCH_TEXT,
            },
            HelpChapter {
                id: "sj",
                title: "2.4 安卓版 指南",
                parent_id: Some("yy"),
                render: render_sj,
                search_text: SJ_SEARCH_TEXT,
            },
            HelpChapter {
                id: "gj",
                title: "2.5 挂接第三方",
                parent_id: Some("yy"),
                render: render_gj,
                search_text: GJ_SEARCH_TEXT,
            },
            HelpChapter {
                id: "wv",
                title: "3 相关文章",
                parent_id: Some("readme"),
                render: render_wv,
                search_text: WV_SEARCH_TEXT,
            },
            HelpChapter {
                id: "wt",
                title: "4 常见问题解答",
                parent_id: Some("readme"),
                render: render_wt,
                search_text: WT_SEARCH_TEXT,
            },
            HelpChapter {
                id: "vy",
                title: "5 学习指引",
                parent_id: Some("readme"),
                render: render_vy,
                search_text: VY_SEARCH_TEXT,
            },
            HelpChapter {
                id: "gy",
                title: "6 关于小鹤",
                parent_id: Some("readme"),
                render: render_gy,
                search_text: GY_SEARCH_TEXT,
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

    /// 在帮助文档中搜索，按句子匹配，返回匹配的章节及匹配句子（含在 search_text 中的索引）
    /// 只搜索汉字、字母和数字，忽略标点符号
    pub fn search(&self, query: &str) -> Vec<(&HelpChapter, Vec<(usize, &'static str)>)> {
        if query.is_empty() {
            return Vec::new();
        }
        let q = query.to_lowercase();
        let q_filtered = filter_chinese_and_letters(&q);

        if q_filtered.is_empty() {
            return Vec::new();
        }

        self.chapters
            .iter()
            .filter_map(|c| {
                let mut matches = Vec::new();
                for (idx, sentence) in c.search_text.iter().enumerate() {
                    let segments = split_search_segments(sentence);
                    let any_match = segments.iter().any(|(_, seg)| {
                        let seg_lower = seg.to_lowercase();
                        let seg_filtered = filter_chinese_and_letters(&seg_lower);
                        seg_filtered.contains(&q_filtered)
                    });
                    if any_match {
                        matches.push((idx, *sentence));
                    }
                }
                let title_lower = c.title.to_lowercase();
                let title_filtered = filter_chinese_and_letters(&title_lower);
                let title_hit = title_filtered.contains(&q_filtered);
                if title_hit || !matches.is_empty() {
                    Some((c, matches))
                } else {
                    None
                }
            })
            .collect()
    }
}

// ──────────────────── 搜索辅助函数（模块级） ────────────────────

/// 判断字符是否为可搜索的文本字符（汉字、字母、数字，含拼音声调符号）
pub(crate) fn is_text_char(c: char) -> bool {
    // 汉字
    ('\u{4e00}'..='\u{9fff}').contains(&c) ||
    // ASCII 字母和数字
    c.is_ascii_alphanumeric() ||
    // 带声调的拉丁字母（常见拼音字母）
    matches!(c,
        'ā' | 'á' | 'ǎ' | 'à' | 'ō' | 'ó' | 'ǒ' | 'ò' | 'ē' | 'é' | 'ě' | 'è' |
        'ī' | 'í' | 'ǐ' | 'ì' | 'ū' | 'ú' | 'ǔ' | 'ù' | 'ǖ' | 'ǘ' | 'ǚ' | 'ǜ' | 'ü' |
        'ê'
    ) ||
    // 处理组合字符：m̀ 实际上是 'm' + '\u{300}'，我们只保留 'm'
    c == 'm' || c == 'n'
}

/// 过滤字符串，只保留汉字、字母和数字
pub(crate) fn filter_chinese_and_letters(s: &str) -> String {
    s.chars().filter(|c| is_text_char(*c)).collect()
}

/// 按非文本字符拆分字符串为搜索元素，返回每个元素的起始位置和内容。
/// 例如 "小·鹤" → [(0, "小"), (4, "鹤")]。
/// 搜索时不会跨元素匹配，从而避免标点分隔导致误匹配。
pub(crate) fn split_search_segments(s: &str) -> Vec<(usize, String)> {
    let mut segments = Vec::new();
    let mut seg_start = 0usize;
    let mut current = String::new();

    for (i, c) in s.char_indices() {
        if is_text_char(c) {
            if current.is_empty() {
                seg_start = i;
            }
            current.push(c);
        } else if !current.is_empty() {
            segments.push((seg_start, current.clone()));
            current.clear();
        }
    }
    if !current.is_empty() {
        segments.push((seg_start, current));
    }

    segments
}

// ──────────────────── 搜索高亮 ────────────────────

/// 在 text 中大小写不敏感地查找 query 的所有出现位置（字节偏移），
/// 使用与搜索一致的标点感知匹配逻辑（split_search_segments + filter_chinese_and_letters）
pub(crate) fn find_all_matches(text: &str, query: &str) -> Vec<(usize, usize)> {
    let q_filtered = filter_chinese_and_letters(&query.to_lowercase());
    if q_filtered.is_empty() {
        return vec![];
    }

    let mut results = Vec::new();
    for (seg_start, seg) in &split_search_segments(text) {
        let seg_lower = seg.to_lowercase();
        let seg_filtered = filter_chinese_and_letters(&seg_lower);
        let qf = &q_filtered;

        let mut search_from = 0;
        while let Some(pos_in_filtered) = seg_filtered[search_from..].find(qf) {
            let abs_pos = search_from + pos_in_filtered;
            let char_index = seg_filtered[..abs_pos].chars().count();

            // 将 filtered 中的字符索引映射回原始 seg 的字节偏移
            let mut fc_count = 0;
            let mut seg_byte_start = 0;
            for (bi, c) in seg.char_indices() {
                if is_text_char(c) {
                    if fc_count == char_index {
                        seg_byte_start = bi;
                        break;
                    }
                    fc_count += 1;
                }
            }

            // 计算匹配结束字节位置
            let q_char_count = qf.chars().count();
            let mut mc_count = 0;
            let mut seg_byte_end = seg_byte_start;
            for (bi, c) in seg[seg_byte_start..].char_indices() {
                if mc_count == q_char_count {
                    break;
                }
                if is_text_char(c) {
                    mc_count += 1;
                }
                seg_byte_end = seg_byte_start + bi + c.len_utf8();
            }

            results.push((seg_start + seg_byte_start, seg_start + seg_byte_end));
            search_from = abs_pos + qf.len();
        }
    }

    // 合并重叠匹配
    if results.len() <= 1 {
        return results;
    }
    let mut merged = vec![results[0]];
    for &m in &results[1..] {
        let last = merged.last_mut().unwrap();
        if m.0 < last.1 {
            last.1 = last.1.max(m.1);
        } else {
            merged.push(m);
        }
    }
    merged
}

const HIGHLIGHT_BG: Color32 = Color32::from_rgb(255, 250, 150);
const HIGHLIGHT_FG: Color32 = Color32::BLACK;

/// 在已有的 horizontal_wrapped 上下文中渲染文本及高亮
fn render_hl_inline(ui: &mut Ui, text: &str, mk: impl Fn(&str) -> RichText) {
    let query = get_search_query();
    let query = match query.as_ref() {
        Some(q) if !q.is_empty() => q.as_str(),
        _ => {
            ui.label(mk(text));
            return;
        }
    };
    let matches = find_all_matches(text, query);
    render_hl_matches(ui, text, mk, matches);
}

/// 渲染高亮片段（不另做查询，直接使用已计算的 matches）
fn render_hl_matches(
    ui: &mut Ui,
    text: &str,
    mk: impl Fn(&str) -> RichText,
    matches: Vec<(usize, usize)>,
) {
    if matches.is_empty() {
        ui.label(mk(text));
        return;
    }
    let mut cursor = 0;
    for &(start, end) in &matches {
        if cursor < start {
            ui.label(mk(&text[cursor..start]));
        }
        ui.label(
            mk(&text[start..end])
                .background_color(HIGHLIGHT_BG)
                .color(HIGHLIGHT_FG),
        );
        cursor = end;
    }
    if cursor < text.len() {
        ui.label(mk(&text[cursor..]));
    }
}

// ── 内联高亮辅助（用于 ui.horizontal / ui.horizontal_wrapped 内） ──

/// 在已有的 horizontal/horizontal_wrapped 上下文中渲染带高亮的纯文本标签
pub(crate) fn label_hl(ui: &mut Ui, text: &str) {
    render_hl_inline(ui, text, |s| RichText::new(s));
}

/// 在已有的 horizontal/horizontal_wrapped 上下文中渲染带高亮的自定义样式文本标签
pub(crate) fn label_hl_mk(ui: &mut Ui, text: &str, mk: impl Fn(&str) -> RichText) {
    render_hl_inline(ui, text, mk);
}

/// 在垂直布局中渲染带高亮的文本
fn render_hl(ui: &mut Ui, text: &str, mk: impl Fn(&str) -> RichText) {
    let query = get_search_query();
    let query = match query.as_ref() {
        Some(q) if !q.is_empty() => q.as_str(),
        _ => {
            ui.horizontal_wrapped(|ui| {
                ui.label(mk(text));
            });
            return;
        }
    };
    let matches = find_all_matches(text, query);
    ui.horizontal_wrapped(|ui| {
        if matches.is_empty() {
            ui.label(mk(text));
        } else {
            render_hl_matches(ui, text, mk, matches);
        }
    });
}

// ──────────────────────────── 渲染辅助函数 ────────────────────────────

fn h1(ui: &mut Ui, t: &str) {
    ui.add_space(8.0);
    render_hl(ui, t, |s| RichText::new(s).size(18.0).strong());
    ui.add_space(2.0);
}
fn h2(ui: &mut Ui, t: &str) {
    ui.add_space(6.0);
    render_hl(ui, t, |s| RichText::new(s).size(15.0).strong());
    ui.add_space(2.0);
}
fn h3(ui: &mut Ui, t: &str) {
    ui.add_space(4.0);
    render_hl(ui, t, |s| RichText::new(s).size(13.0).strong());
}
fn h4(ui: &mut Ui, t: &str) {
    ui.add_space(3.0);
    render_hl(ui, t, |s| RichText::new(s).size(12.0).strong());
}

fn p(ui: &mut Ui, t: &str) {
    render_hl(ui, t, |s| RichText::new(s));
}
fn sp(ui: &mut Ui) {
    ui.add_space(4.0);
}
fn hr(ui: &mut Ui) {
    ui.separator();
    ui.add_space(4.0);
}

fn bul(ui: &mut Ui, t: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label("  •");
        render_hl_inline(ui, t, |s| RichText::new(s));
    });
}

fn num(ui: &mut Ui, n: &str, t: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("  {}.", n));
        render_hl_inline(ui, t, |s| RichText::new(s));
    });
}

fn qt(ui: &mut Ui, t: &str) {
    render_hl(ui, &format!("  {}", t), |s| {
        RichText::new(s).italics().color(GRAY)
    });
}

fn red(ui: &mut Ui, t: &str) {
    render_hl(ui, t, |s| RichText::new(s).color(RED));
}
fn blue(ui: &mut Ui, t: &str) {
    render_hl(ui, t, |s| RichText::new(s).color(BLUE));
}

fn code(ui: &mut Ui, t: &str) {
    render_hl(ui, t, |s| {
        RichText::new(s)
            .monospace()
            .background_color(CODE_BG)
            .color(CODE_FG)
    });
}

fn lnk(ui: &mut Ui, nav: &mut HelpNav, text: &str, target: &'static str) {
    if ui.link(text).clicked() {
        nav.goto = Some(target);
    }
}

fn ext_link(ui: &mut Ui, text: &str, url: &str) {
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
fn navrow(
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
mod search_text;
use search_text::*;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    // ── 共享辅助 ──

    fn chapter_mapping() -> HashMap<&'static str, &'static str> {
        HashMap::from([
            ("readme", "render_readme"),
            ("xh", "render_xh"),
            ("up", "render_up"),
            ("ux", "render_ux"),
            ("gz", "render_gz"),
            ("zg", "render_zg"),
            ("yy", "render_yy"),
            ("jm", "render_jm"),
            ("fh", "render_fh"),
            ("pc", "render_pc"),
            ("sj", "render_sj"),
            ("gj", "render_gj"),
            ("wv", "render_wv"),
            ("wt", "render_wt"),
            ("vy", "render_vy"),
            ("gy", "render_gy"),
        ])
    }

    fn const_name_to_id(name: &str) -> Option<&'static str> {
        match name {
            "README_SEARCH_TEXT" => Some("readme"),
            "XH_SEARCH_TEXT" => Some("xh"),
            "UP_SEARCH_TEXT" => Some("up"),
            "UX_SEARCH_TEXT" => Some("ux"),
            "GZ_SEARCH_TEXT" => Some("gz"),
            "ZG_SEARCH_TEXT" => Some("zg"),
            "YY_SEARCH_TEXT" => Some("yy"),
            "JM_SEARCH_TEXT" => Some("jm"),
            "FH_SEARCH_TEXT" => Some("fh"),
            "PC_SEARCH_TEXT" => Some("pc"),
            "SJ_SEARCH_TEXT" => Some("sj"),
            "GJ_SEARCH_TEXT" => Some("gj"),
            "WV_SEARCH_TEXT" => Some("wv"),
            "WT_SEARCH_TEXT" => Some("wt"),
            "VY_SEARCH_TEXT" => Some("vy"),
            "GY_SEARCH_TEXT" => Some("gy"),
            _ => None,
        }
    }

    /// 解析 search_text.rs，返回 (章节id, 文本条目) 列表
    fn parse_search_texts(src: &str) -> Vec<(String, Vec<String>)> {
        let mut result: Vec<(String, Vec<String>)> = Vec::new();
        let mut id: Option<String> = None;
        let mut texts: Vec<String> = Vec::new();

        for raw_line in src.lines() {
            let line = raw_line.trim();
            if line.starts_with("pub(super) const") {
                if let Some(prev) = id.take() {
                    result.push((prev, std::mem::take(&mut texts)));
                }
                id = line
                    .split_whitespace()
                    .nth(2)
                    .map(|s| s.trim_end_matches(':'))
                    .and_then(const_name_to_id)
                    .map(|s| s.to_string());
                continue;
            }
            if id.is_some() {
                if line.starts_with("];") {
                    if let Some(prev) = id.take() {
                        result.push((prev, std::mem::take(&mut texts)));
                    }
                    continue;
                }
                if let Some(s) = extract_string_literal(line)
                    && !s.is_empty()
                {
                    texts.push(s);
                }
            }
        }
        if let Some(prev) = id.take() {
            result.push((prev, texts));
        }
        result
    }

    /// 从 renderers.rs 提取各 render 函数中的文本内容。
    /// 扫描 h1/h2/h3/h4/p/bul/num/qt/red/blue/code/lnk/ext_link 调用，
    /// 处理单行和多行调用两种形式。
    fn extract_renderer_texts(src: &str) -> Vec<(String, Vec<String>)> {
        let map = chapter_mapping();
        let fn_to_id: HashMap<&str, &str> = map.iter().map(|(&id, &f)| (f, id)).collect();

        const HELPERS: &[&str] = &[
            "h1(",
            "h2(",
            "h3(",
            "h4(",
            "p(",
            "bul(",
            "num(",
            "qt(",
            "red(",
            "blue(",
            "code(",
            "lnk(",
            "ext_link(",
        ];

        let mut result: Vec<(String, Vec<String>)> = Vec::new();
        let mut cur_id: Option<&str> = None;
        let mut texts: Vec<String> = Vec::new();
        let mut pending = false; // 上一个 helper 调用尚未闭合

        for raw_line in src.lines() {
            let t = raw_line.trim();

            // 函数边界
            if t.starts_with("pub(super) fn render_") {
                if let Some(id) = cur_id.take() {
                    result.push((id.to_string(), std::mem::take(&mut texts)));
                }
                pending = false;
                for (fn_name, &id) in &fn_to_id {
                    if t.contains(&format!("fn {fn_name}(")) {
                        cur_id = Some(id);
                        break;
                    }
                }
                continue;
            }
            if cur_id.is_none() {
                continue;
            }

            // 多行调用的延续行：以 "..." 开头的行
            if pending && t.starts_with('"') {
                if let Some(s) = extract_string_literal(t)
                    && s.chars().count() >= 2
                {
                    texts.push(s);
                }
                // 检查本行是否闭合
                pending = open_parens_after_first(t) > close_parens_in(t);
                continue;
            }
            pending = false;

            // 检查是否以文本 helper 开头
            for &helper in HELPERS {
                if t.starts_with(helper) {
                    // 提取 helper 括号后的第一个字符串
                    let after = &t[t.find(helper).unwrap() + helper.len()..];
                    if let Some(s) = extract_string_literal(after)
                        && s.chars().count() >= 2
                    {
                        texts.push(s);
                    }
                    // 检查调用是否跨行（括号未闭合）
                    let open = open_parens_after_first(t);
                    let close = close_parens_in(t);
                    if open > close {
                        pending = true;
                    }
                    break;
                }
            }
        }
        if let Some(id) = cur_id.take() {
            result.push((id.to_string(), texts));
        }
        result
    }

    /// 计算一行中在首个 ( 之后的开括号数
    fn open_parens_after_first(line: &str) -> usize {
        let mut in_str = false;
        let mut escape = false;
        let mut found_first = false;
        let mut count = 0usize;
        for ch in line.chars() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' && in_str {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_str = !in_str;
                continue;
            }
            if in_str {
                continue;
            }
            if ch == '(' {
                if !found_first {
                    found_first = true;
                } else {
                    count += 1;
                }
            }
        }
        count
    }

    /// 计算一行中（字符串外）的闭括号数
    fn close_parens_in(line: &str) -> usize {
        let mut in_str = false;
        let mut escape = false;
        let mut count = 0usize;
        for ch in line.chars() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' && in_str {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_str = !in_str;
                continue;
            }
            if !in_str && ch == ')' {
                count += 1;
            }
        }
        count
    }

    // ── 正向测试（暂时禁用） ──
    // search_text.rs 与 renderers.rs 的文本粒度不同（短语 vs 整句），
    // 导致大量误报。未来若统一粒度可重新启用。
    //
    // #[test]
    // fn search_text_matches_renderers() { ... }

    // ── 反向测试：renderers.rs 中新增的文本内容应被 search_text 覆盖 ──
    // 用基线方式运行：记录当前已知的不匹配数量，只在数量增加时失败。
    // 新增渲染内容后请同步更新 search_text.rs，并下调 BASELINE。
    #[test]
    fn renderers_content_in_search_text() {
        const BASELINE: usize = 237; // 当前已知的不匹配数（2026-06-23 基线）

        let root = env!("CARGO_MANIFEST_DIR");
        let renderers_src = std::fs::read_to_string(format!("{root}/src/help/renderers.rs"))
            .expect("cannot read renderers.rs");
        let search_src = std::fs::read_to_string(format!("{root}/src/help/search_text.rs"))
            .expect("cannot read search_text.rs");

        let search_entries = parse_search_texts(&search_src);
        let search_map: HashMap<&str, &[String]> = search_entries
            .iter()
            .map(|(id, texts)| (id.as_str(), texts.as_slice()))
            .collect();

        let renderer_entries = extract_renderer_texts(&renderers_src);
        let mut missing: Vec<String> = Vec::new();

        for (chapter_id, texts) in &renderer_entries {
            let search_texts = match search_map.get(chapter_id.as_str()) {
                Some(t) => *t,
                None => continue,
            };
            let all_search = search_texts.join("\n");

            for text in texts {
                let trimmed = text.trim();
                if trimmed.chars().count() < 4 {
                    continue;
                }
                if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    continue;
                }
                if trimmed.chars().all(|c| c.is_ascii_digit() || c == '.') {
                    continue;
                }

                if !all_search.contains(trimmed) {
                    missing.push(format!("  [{chapter_id}] \"{trimmed}\""));
                }
            }
        }

        assert!(
            missing.len() <= BASELINE,
            "renderers.rs 中有 {} 条文本未被 search_text 覆盖（基线 {}），\
             新增渲染内容后请同步更新 search_text.rs 并下调 BASELINE。\n\
             新增的不匹配项：\n{}",
            missing.len(),
            BASELINE,
            missing[BASELINE..].join("\n"),
        );
    }

    /// 从一行 Rust 源码中提取第一个 "..." 字符串字面量的内容。
    fn extract_string_literal(line: &str) -> Option<String> {
        let chars: Vec<(usize, char)> = line.char_indices().collect();
        let first_quote = chars.iter().position(|&(_, c)| c == '"')?;

        let mut result = String::new();
        let mut idx = first_quote + 1;
        while idx < chars.len() {
            let (_, ch) = chars[idx];
            match ch {
                '"' => return Some(result),
                '\\' if idx + 1 < chars.len() => {
                    let (_, next) = chars[idx + 1];
                    match next {
                        '"' => result.push('"'),
                        '\\' => result.push('\\'),
                        'n' => result.push('\n'),
                        't' => result.push('\t'),
                        _ => {
                            result.push('\\');
                            result.push(next);
                        }
                    }
                    idx += 2;
                }
                _ => {
                    result.push(ch);
                    idx += 1;
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod hl_tests {
    use super::*;

    #[test]
    fn test_find_all_matches_basic() {
        // 基本中文匹配
        let result = find_all_matches("小鹤音形帮助文档", "音形");
        assert_eq!(result, vec![(6, 12)]); // "音形" 在 bytes 6-12
    }

    #[test]
    fn test_find_all_matches_multiple() {
        // 多次出现
        let result = find_all_matches("好好学习学习", "学习");
        // "学习" appears at bytes 6 and 12 in "好好学习学习"
        assert_eq!(result, vec![(6, 12), (12, 18)]);
    }

    #[test]
    fn test_find_all_matches_with_punctuation() {
        // 标点分隔的文本
        let result = find_all_matches("双拼＋双形，组合", "双拼");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], (0, 6));
    }

    #[test]
    fn test_find_all_matches_with_quotes() {
        // 引号内的匹配
        let result = find_all_matches("\u{201c}语\u{201d}拆分", "语");
        assert_eq!(result, vec![(3, 6)]); // 左引号3字节+语3字节
    }

    #[test]
    fn test_find_all_matches_case_insensitive() {
        // 大小写不敏感
        let result = find_all_matches("Hello World hello", "hello");
        assert_eq!(result, vec![(0, 5), (12, 17)]);
    }

    #[test]
    fn test_find_all_matches_filtered_query() {
        // 查询词中的非文本字符被过滤
        let result = find_all_matches("小鹤音形", "小·鹤");
        // query被过滤后变成 "小鹤"
        assert_eq!(result, vec![(0, 6)]);
    }

    #[test]
    fn test_find_all_matches_adjacent() {
        // 非重叠匹配不应合并
        let result = find_all_matches("aaabbb", "aa");
        assert_eq!(result.len(), 1); // "aa" at 0-2 only, search_from advances past it
    }

    #[test]
    fn test_find_all_matches_overlapping_merge() {
        let result = find_all_matches("ababa", "aba");
        assert_eq!(result.len(), 1); // "aba" at 0 and 2 overlap -> merged
    }

    #[test]
    fn test_find_all_matches_number_mixed() {
        // 数字与文字混合
        let result = find_all_matches("第1条 第2条", "1");
        assert_eq!(result, vec![(3, 4)]);
    }

    #[test]
    fn test_find_all_matches_number_mixed_2() {
        let result = find_all_matches("abc123def456", "23");
        assert_eq!(result, vec![(4, 6)]);
    }

    #[test]
    fn test_find_actual_search_text() {
        // 使用实际的 search_text 数据测试
        let text = "单字以\"双拼＋双形\"组合的标准四码音形类输入方案";
        let result = find_all_matches(text, "音形");
        println!("text: '{text}'");
        for (s, e) in &result {
            println!("  match {s}-{e}: '{}'", &text[*s..*e]);
        }
        assert!(!result.is_empty(), "Should find '音形' in text");
    }

    #[test]
    fn test_find_actual_search_text_sppx() {
        // 双拼+双形 - 跨标点不匹配（预期行为）
        let text = "单字以\"双拼＋双形\"组合的标准四码音形类输入方案";
        // query "双拼双形" filtered -> "双拼双形"
        // split: "单字以", "双拼", "双形", "组合的标准四码音形类输入方案"
        // "双拼双形" 不在任何 segment 中
        let result = find_all_matches(text, "双拼双形");
        assert!(result.is_empty(), "Should NOT match across punctuation");
    }

    #[test]
    fn test_find_all_matches_empty_query() {
        let result = find_all_matches("hello", "");
        assert!(result.is_empty());
    }

    #[test]
    fn test_find_all_matches_query_all_punct() {
        let result = find_all_matches("hello", " ,.");
        assert!(result.is_empty());
    }
}
