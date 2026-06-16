use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo::rerun-if-changed=小鹤音形使用说明.md");

    let content = fs::read_to_string("小鹤音形使用说明.md").unwrap();
    let entries = parse_all(&content);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_dict.rs");

    let mut code = String::new();
    code.push_str("#[allow(non_upper_case_globals, clippy::large_const_arrays)]\n");
    // NOTE: no `use` imports here; included via `include!` into main.rs which has its own imports.
    code.push_str(&format!(
        "pub const DICT_ENTRIES: [crate::dict::DictEntry; {}] = [\n",
        entries.len()
    ));

    for (text, code_str, cat_name, is_secondary) in &entries {
        let escaped_text = text.replace('\\', "\\\\").replace('"', "\\\"");
        let escaped_code = code_str.replace('\\', "\\\\").replace('"', "\\\"");
        code.push_str(&format!(
            "    crate::dict::DictEntry {{ text: \"{}\", code: \"{}\", category: crate::dict::Category::{cat_name}, is_secondary: {is_secondary} }},\n",
            escaped_text, escaped_code,
        ));
    }
    code.push_str("];\n");

    fs::write(&dest_path, &code).unwrap();
    println!(
        "cargo::warning=Generated {} dictionary entries.",
        entries.len()
    );
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TableFormat {
    TwoCol,
    KeyCharPairs,
}

fn category_name_from_header(header: &str) -> &'static str {
    let h = header.trim();
    if h.contains("一级简码") {
        "YiJiJianMa"
    } else if h.contains("二重简码") {
        "ErChongJianMa"
    } else if h.contains("三码填空") {
        "SanMaTianKong"
    } else if h.contains("四码全码（字") {
        "SiMaQuanMaZi"
    } else if h.contains("四码全码（词）置顶") {
        "SiMaQuanMaCiZhiDing"
    } else if h.contains("四码全码（词）") {
        "SiMaQuanMaCi"
    } else if h.contains("快符") {
        "KuaiFu"
    } else if h.contains("符号（o") {
        "FuHao"
    } else if h.contains("部首部件") {
        "BuShouBuJian"
    } else if h.contains("Emoji") {
        "Emoji"
    } else if h.contains("微信表情") {
        "WeiXinBiaoQing"
    } else if h.contains("网站直达") {
        "WangZhanZhiDa"
    } else if h.contains("随心所欲") {
        "SuiXinSuoYu"
    } else if h.contains("二简（次选字") {
        "ErJianCiXuan"
    } else if h.contains("四码全码（次选词") {
        "SiMaCiXuan"
    } else if h.contains("首选四码全码") {
        "ShouXuanSiMa"
    } else {
        ""
    }
}

fn is_secondary_for_category(cat_name: &str) -> bool {
    matches!(cat_name, "ErJianCiXuan" | "SiMaCiXuan")
}

fn table_format_from_header(header: &str) -> Option<TableFormat> {
    let h = header.trim();
    if h.contains("一级简码") {
        Some(TableFormat::KeyCharPairs)
    } else {
        Some(TableFormat::TwoCol)
    }
}

fn is_separator_row(parts: &[&str]) -> bool {
    parts.iter().all(|p| {
        let trimmed = p.trim();
        trimmed.is_empty() || trimmed.chars().all(|c| c == '-')
    })
}

fn parse_all(content: &str) -> Vec<(String, String, &'static str, bool)> {
    let mut entries = Vec::new();
    let mut current_category = "";
    let mut current_format: Option<TableFormat> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("## ") {
            let header = &trimmed[3..];
            let cat_name = category_name_from_header(header);
            if !cat_name.is_empty() {
                current_category = cat_name;
                current_format = table_format_from_header(header);
            } else {
                current_category = "";
                current_format = None;
            }
            continue;
        }

        if current_category.is_empty() || current_format.is_none() {
            continue;
        }
        if !line.trim_start().starts_with('|') {
            continue;
        }

        let raw = line.trim();
        if raw.len() < 3 || !raw.starts_with('|') || !raw.ends_with('|') {
            continue;
        }

        let inner = &raw[1..raw.len() - 1];
        let parts: Vec<&str> = inner.split('|').collect();

        if parts.is_empty() {
            continue;
        }

        if is_separator_row(&parts) {
            continue;
        }

        let is_secondary = is_secondary_for_category(current_category);

        match current_format.unwrap() {
            TableFormat::TwoCol => {
                if parts.len() < 2 {
                    continue;
                }
                let text = parts[0].trim();
                let code = parts[1].trim();
                if text.is_empty() || code.is_empty() {
                    continue;
                }
                entries.push((
                    text.to_string(),
                    code.to_string(),
                    current_category,
                    is_secondary,
                ));
            }
            TableFormat::KeyCharPairs => {
                if parts.len() < 2 || parts.len() % 2 != 0 {
                    continue;
                }
                for chunk in parts.chunks(2) {
                    let key = chunk[0].trim();
                    let ch = chunk[1].trim();
                    if key.is_empty() || ch.is_empty() {
                        continue;
                    }
                    entries.push((
                        ch.to_string(),
                        key.to_string(),
                        current_category,
                        is_secondary,
                    ));
                }
            }
        }
    }

    entries
}
