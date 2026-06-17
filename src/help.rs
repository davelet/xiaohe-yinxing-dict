use std::collections::HashMap;

use eframe::egui::{self, Color32, RichText, ScrollArea, Ui};

/// 帮助文档章节
#[derive(Debug, Clone)]
pub struct HelpChapter {
    pub id: &'static str,
    pub title: &'static str,
    pub content: &'static str,
    pub parent_id: Option<&'static str>,
}

/// 帮助文档管理器
pub struct HelpManager {
    chapters: Vec<HelpChapter>,
    chapter_map: HashMap<&'static str, usize>,
}

impl HelpManager {
    pub fn new() -> Self {
        let chapters = vec![
            HelpChapter {
                id: "readme",
                title: "小鹤音形帮助文档",
                content: include_str!("../help/README.md"),
                parent_id: None,
            },
            HelpChapter {
                id: "xh",
                title: "1 入门概述",
                content: include_str!("../help/xh.md"),
                parent_id: Some("readme"),
            },
            HelpChapter {
                id: "up",
                title: "1.1 双拼",
                content: include_str!("../help/up.md"),
                parent_id: Some("xh"),
            },
            HelpChapter {
                id: "ux",
                title: "1.2 双形（鹤形）",
                content: include_str!("../help/ux.md"),
                parent_id: Some("xh"),
            },
            HelpChapter {
                id: "gz",
                title: "1.2.1 规则",
                content: include_str!("../help/gz.md"),
                parent_id: Some("ux"),
            },
            HelpChapter {
                id: "zg",
                title: "1.2.2 字根",
                content: include_str!("../help/zg.md"),
                parent_id: Some("ux"),
            },
            HelpChapter {
                id: "yy",
                title: "2 应用",
                content: include_str!("../help/yy.md"),
                parent_id: Some("readme"),
            },
            HelpChapter {
                id: "jm",
                title: "2.1 简码",
                content: include_str!("../help/jm.md"),
                parent_id: Some("yy"),
            },
            HelpChapter {
                id: "fh",
                title: "2.2 符号",
                content: include_str!("../help/fh.md"),
                parent_id: Some("yy"),
            },
            HelpChapter {
                id: "pc",
                title: "2.3 Win版",
                content: include_str!("../help/pc.md"),
                parent_id: Some("yy"),
            },
            HelpChapter {
                id: "sj",
                title: "2.4 安卓版",
                content: include_str!("../help/sj.md"),
                parent_id: Some("yy"),
            },
            HelpChapter {
                id: "gj",
                title: "2.5 挂接",
                content: include_str!("../help/gj.md"),
                parent_id: Some("yy"),
            },
            HelpChapter {
                id: "wv",
                title: "3 相关文章",
                content: include_str!("../help/wv.md"),
                parent_id: Some("readme"),
            },
            HelpChapter {
                id: "wt",
                title: "4 常见问题",
                content: include_str!("../help/wt.md"),
                parent_id: Some("readme"),
            },
            HelpChapter {
                id: "vy",
                title: "5 学习指引",
                content: include_str!("../help/vy.md"),
                parent_id: Some("readme"),
            },
            HelpChapter {
                id: "gy",
                title: "6 关于小鹤",
                content: include_str!("../help/gy.md"),
                parent_id: Some("readme"),
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

    /// 获取所有章节
    pub fn chapters(&self) -> &[HelpChapter] {
        &self.chapters
    }

    /// 根据ID获取章节
    pub fn get_chapter(&self, id: &str) -> Option<&HelpChapter> {
        self.chapter_map.get(id).map(|&i| &self.chapters[i])
    }

    /// 获取顶级章节（没有父章节的）
    pub fn top_level_chapters(&self) -> Vec<&HelpChapter> {
        self.chapters
            .iter()
            .filter(|c| c.parent_id.is_none())
            .collect()
    }

    /// 获取指定章节的子章节
    pub fn child_chapters(&self, parent_id: &str) -> Vec<&HelpChapter> {
        self.chapters
            .iter()
            .filter(|c| c.parent_id == Some(parent_id))
            .collect()
    }

    /// 在 UI 中渲染 Markdown 内容（逐行解析，保留格式）
    pub fn render_markdown(ui: &mut Ui, content: &str) {
        let mut lines = content.lines().peekable();
        let mut in_html_table = false;
        let mut html_table_rows: Vec<Vec<String>> = Vec::new();
        let mut table_id_counter: usize = 0;

        while let Some(line) = lines.next() {
            let trimmed = line.trim();

            // ── HTML table handling ──
            if trimmed.contains("<table") {
                in_html_table = true;
                html_table_rows.clear();
                continue;
            }
            if in_html_table && (trimmed.contains("</table>") || trimmed.contains("</div>")) {
                in_html_table = false;
                if !html_table_rows.is_empty() {
                    Self::render_html_table(ui, &html_table_rows, table_id_counter);
                    table_id_counter += 1;
                }
                html_table_rows.clear();
                if trimmed.contains("</div>") {
                    // </div> might be standalone, continue processing
                }
                continue;
            }
            if in_html_table {
                if trimmed.contains("<tr>") || trimmed.contains("</tr>") {
                    continue;
                }
                if trimmed.contains("<th>") || trimmed.contains("<td>") {
                    let cells: Vec<String> = trimmed
                        .split("</th>")
                        .flat_map(|s| s.split("</td>"))
                        .map(|s| {
                            s.replace("<th>", "")
                                .replace("<td>", "")
                                .replace("<p align=\"left\" style=\"color: #D9534F;\">", "")
                                .replace("</p>", "")
                                .replace("<br/>", "")
                                .replace("<br>", "")
                                .trim()
                                .to_string()
                        })
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !cells.is_empty() {
                        html_table_rows.push(cells);
                    }
                }
                continue;
            }

            // ── Skip HTML-only lines ──
            if trimmed.starts_with("<div")
                || trimmed.starts_with("</div>")
                || trimmed.starts_with("<span")
                || trimmed.starts_with("</span>")
                || trimmed.starts_with("<p ")
                || trimmed.starts_with("</p>")
                || trimmed.starts_with("<colgroup")
                || trimmed.starts_with("<col ")
                || trimmed.starts_with("<img")
                || trimmed.starts_with("<br")
                || trimmed.starts_with("<a ")
                || trimmed.starts_with("</a>")
                || trimmed.starts_with("<kbd")
                || trimmed.starts_with("</kbd>")
                || trimmed.starts_with("<span")
            {
                continue;
            }

            // ── Image lines ──
            if trimmed.starts_with("![") {
                // Show placeholder for images
                ui.colored_label(Color32::GRAY, "[图片]");
                continue;
            }

            // ── Empty line → spacing ──
            if trimmed.is_empty() {
                ui.add_space(4.0);
                continue;
            }

            // ── Horizontal rule ──
            if trimmed == "---" || trimmed == "***" || trimmed == "___" {
                ui.separator();
                ui.add_space(4.0);
                continue;
            }

            // ── Headings ──
            if trimmed.starts_with('#') {
                let level = trimmed.chars().take_while(|&c| c == '#').count();
                let title = trimmed[level..].trim();
                if !title.is_empty() {
                    let font_size = match level {
                        1 => 18.0,
                        2 => 15.0,
                        3 => 13.0,
                        _ => 12.0,
                    };
                    ui.add_space(6.0);
                    ui.label(RichText::new(title).size(font_size).strong());
                    ui.add_space(2.0);
                }
                continue;
            }

            // ── Blockquote ──
            if trimmed.starts_with("> ") {
                let quote_text = &trimmed[2..];
                ui.colored_label(
                    Color32::from_rgb(100, 100, 100),
                    RichText::new(format!("  {}", quote_text)).italics(),
                );
                continue;
            }

            // ── Unordered list ──
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                let item = &trimmed[2..];
                ui.horizontal(|ui| {
                    ui.label("  •");
                    Self::render_inline_markdown(ui, item);
                });
                continue;
            }

            // ── Numbered list ──
            if trimmed
                .chars()
                .next()
                .map_or(false, |c| c.is_ascii_digit())
                && trimmed.contains(". ")
            {
                let dot_pos = trimmed.find(". ").unwrap();
                let num = &trimmed[..dot_pos];
                let item = &trimmed[dot_pos + 2..];
                ui.horizontal(|ui| {
                    ui.label(format!("  {}.", num));
                    Self::render_inline_markdown(ui, item);
                });
                continue;
            }

            // ── Markdown table row (| ... |) ──
            if trimmed.contains('|') && !trimmed.starts_with(":-") {
                let cells: Vec<&str> = trimmed
                    .split('|')
                    .map(|c| c.trim())
                    .filter(|c| !c.is_empty())
                    .collect();
                if cells.len() >= 2 {
                    let col_width = 100.0;
                    let total_w = cells.len() as f32 * col_width;
                    ScrollArea::horizontal()
                        .id_salt(format!("md_table_hscroll_{}", table_id_counter))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                for cell in &cells {
                                    ui.add_sized(
                                        [col_width, 18.0],
                                        egui::Label::new(
                                            RichText::new(*cell).monospace().size(11.0),
                                        ),
                                    );
                                }
                            });
                        });
                    table_id_counter += 1;
                    // Also ensure the outer ScrollArea can handle the width
                    ui.allocate_space(egui::vec2(total_w.min(ui.available_width()), 2.0));
                    continue;
                }
            }

            // ── Regular text with inline formatting ──
            Self::render_inline_markdown(ui, trimmed);
        }
    }

    /// Render a line of text with inline markdown (**bold**, `code`, [link](url))
    fn render_inline_markdown(ui: &mut Ui, text: &str) {
        // Simple inline parser: split by ** and `
        let mut in_bold = false;

        // Process **bold** and `code` inline
        let mut segments: Vec<(String, bool, bool)> = Vec::new(); // (text, is_bold, is_code)

        let chars: Vec<char> = text.chars().collect();
        let mut i = 0;
        let mut current = String::new();

        while i < chars.len() {
            if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
                if !current.is_empty() {
                    segments.push((current.clone(), in_bold, false));
                    current.clear();
                }
                in_bold = !in_bold;
                i += 2;
            } else if chars[i] == '`' {
                if !current.is_empty() {
                    segments.push((current.clone(), in_bold, false));
                    current.clear();
                }
                // Read code content
                i += 1;
                let mut code = String::new();
                while i < chars.len() && chars[i] != '`' {
                    code.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() {
                    i += 1; // skip closing `
                }
                if !code.is_empty() {
                    segments.push((code, false, true));
                }
            } else if chars[i] == '[' {
                // Link: [text](url) - extract text only
                if !current.is_empty() {
                    segments.push((current.clone(), in_bold, false));
                    current.clear();
                }
                i += 1;
                let mut link_text = String::new();
                while i < chars.len() && chars[i] != ']' {
                    link_text.push(chars[i]);
                    i += 1;
                }
                if i < chars.len() {
                    i += 1; // skip ]
                }
                // Skip URL part (...)
                if i < chars.len() && chars[i] == '(' {
                    i += 1;
                    while i < chars.len() && chars[i] != ')' {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1; // skip )
                    }
                }
                if !link_text.is_empty() {
                    segments.push((link_text, false, false));
                }
            } else {
                current.push(chars[i]);
                i += 1;
            }
        }
        if !current.is_empty() {
            segments.push((current, in_bold, false));
        }

        if segments.is_empty() {
            ui.label("");
            return;
        }

        ui.horizontal_wrapped(|ui| {
            for (text, is_bold, is_code) in &segments {
                if *is_code {
                    ui.label(
                        RichText::new(text.as_str())
                            .monospace()
                            .background_color(Color32::from_gray(40))
                            .color(Color32::from_rgb(200, 200, 200)),
                    );
                } else if *is_bold {
                    ui.label(RichText::new(text.as_str()).strong());
                } else {
                    ui.label(text.as_str());
                }
            }
        });
    }

    /// Render extracted HTML table as monospace grid
    fn render_html_table(ui: &mut Ui, rows: &[Vec<String>], table_id: usize) {
        if rows.is_empty() {
            return;
        }
        let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(1);
        let col_width = 90.0;
        let total_w = col_count as f32 * col_width;

        ScrollArea::horizontal()
            .id_salt(format!("html_table_hscroll_{}", table_id))
            .show(ui, |ui| {
            ui.vertical(|ui| {
                for (row_idx, row) in rows.iter().enumerate() {
                    ui.horizontal(|ui| {
                        for cell in row {
                            let rt = if row_idx == 0 {
                                RichText::new(cell.as_str())
                                    .monospace()
                                    .size(11.0)
                                    .strong()
                                    .color(Color32::from_rgb(200, 160, 80))
                            } else {
                                RichText::new(cell.as_str()).monospace().size(11.0)
                            };
                            ui.add_sized([col_width, 18.0], egui::Label::new(rt));
                        }
                    });
                }
            });
        });
        ui.allocate_space(egui::vec2(total_w.min(ui.available_width()), 4.0));
        ui.add_space(4.0);
    }
}