use crate::dict::{Category, ExternalDictEntry};
use std::fs;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Rime 词典文件解析器
pub struct RimeLoader {
    rime_user_dir: String,
}

impl RimeLoader {
    /// 创建新的 RimeLoader
    pub fn new(rime_user_dir: &str) -> Self {
        Self {
            rime_user_dir: rime_user_dir.to_string(),
        }
    }

    /// 扫描 Rime 目录下的词典文件
    pub fn scan_dict_files(&self) -> Vec<DictFileInfo> {
        let mut files = Vec::new();
        let rime_dir = Path::new(&self.rime_user_dir);

        if !rime_dir.exists() {
            return files;
        }

        // 扫描主目录下的 .dict.yaml 文件
        if let Ok(entries) = fs::read_dir(rime_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file()
                    && let Some(name) = path.file_name()
                {
                    let name_str = name.to_string_lossy().to_string();
                    if name_str.ends_with(".dict.yaml") || name_str.ends_with(".txt") {
                        let dict_name = name_str.replace(".dict.yaml", "").replace(".txt", "");
                        let entry_count = self.count_entries(&path);
                        files.push(DictFileInfo {
                            path: path.to_string_lossy().to_string(),
                            name: dict_name,
                            entry_count,
                        });
                    }
                }
            }
        }

        // 扫描子目录（如 flypy/）
        if let Ok(entries) = fs::read_dir(rime_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().unwrap().to_string_lossy().to_string();
                    // 跳过隐藏目录和特殊目录
                    if dir_name.starts_with('.')
                        || dir_name == "build"
                        || dir_name == "flypy.userdb"
                    {
                        continue;
                    }
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file()
                                && let Some(name) = sub_path.file_name()
                            {
                                let name_str = name.to_string_lossy().to_string();
                                if name_str.ends_with(".dict.yaml") || name_str.ends_with(".txt") {
                                    let dict_name =
                                        name_str.replace(".dict.yaml", "").replace(".txt", "");
                                    let entry_count = self.count_entries(&sub_path);
                                    files.push(DictFileInfo {
                                        path: sub_path.to_string_lossy().to_string(),
                                        name: dict_name,
                                        entry_count,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        files
    }

    /// 加载词典文件
    pub fn load_dict_file(&self, path: &str) -> Result<Vec<ExternalDictEntry>, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("读取文件失败 '{}': {}", path, e))?;

        // 从文件路径提取来源名称（文件名 stem）
        let source = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .replace(".dict.yaml", "")
            .replace(".txt", "");

        let entries = self.parse_rime_content(&content, &source)?;
        Ok(entries)
    }

    /// 统计词典文件条目数（不加载到内存）
    fn count_entries(&self, path: &Path) -> Option<usize> {
        let content = fs::read_to_string(path).ok()?;
        let mut count = 0;
        let mut in_yaml_header = false;
        let mut yaml_header_started = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed == "---" {
                if !yaml_header_started {
                    in_yaml_header = true;
                    yaml_header_started = true;
                } else {
                    in_yaml_header = false;
                }
                continue;
            }

            if trimmed == "..." {
                in_yaml_header = false;
                continue;
            }

            if in_yaml_header {
                continue;
            }

            if !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.contains('\t') {
                count += 1;
            }
        }

        Some(count)
    }

    /// 解析 Rime 词典内容
    fn parse_rime_content(
        &self,
        content: &str,
        source: &str,
    ) -> Result<Vec<ExternalDictEntry>, String> {
        let lines = content.lines();
        let mut in_yaml_header = false;
        let mut yaml_header_started = false;
        let mut data_lines = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            // 检查 YAML 头部开始
            if trimmed == "---" {
                if !yaml_header_started {
                    in_yaml_header = true;
                    yaml_header_started = true;
                    continue;
                } else {
                    // 第二个 ---，YAML 头部结束
                    in_yaml_header = false;
                    continue;
                }
            }

            // 检查 YAML 头部结束（...）
            if trimmed == "..." {
                in_yaml_header = false;
                continue;
            }

            // 跳过 YAML 头部内容
            if in_yaml_header {
                continue;
            }

            // 收集数据行
            data_lines.push(line);
        }

        // 解析数据行
        self.parse_data_lines(&data_lines, source)
    }

    /// 解析数据行
    fn parse_data_lines(
        &self,
        lines: &[&str],
        source: &str,
    ) -> Result<Vec<ExternalDictEntry>, String> {
        let mut entries = Vec::new();

        for line in lines {
            let trimmed = line.trim();

            // 跳过空行和注释行
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // 解析 TSV 格式：文字	编码	[权重]
            let parts: Vec<&str> = trimmed.split('\t').collect();
            if parts.len() >= 2 {
                let text = parts[0].trim().to_string();
                let code = parts[1].trim().to_string();

                // 跳过表头行（如 "文字	编码	权重"）
                if (text == "文字" || text == "字") && (code == "编码" || code == "码") {
                    continue;
                }

                if !text.is_empty() && !code.is_empty() {
                    entries.push(ExternalDictEntry::new(
                        text,
                        code,
                        Category::External,
                        false,
                        source.to_string(),
                    ));
                }
            }
        }

        Ok(entries)
    }
}

/// 向小鹤音形自定义词典追加条目
/// 返回文件路径
pub fn append_entry_to_custom_dict(
    rime_dir: &str,
    text: &str,
    code: &str,
) -> Result<String, String> {
    let custom_path = Path::new(rime_dir).join("flypy_custom.dict.yaml");
    let path_str = custom_path.to_string_lossy().to_string();

    let line = format!("{}\t{}\t500", text, code);

    if !custom_path.exists() {
        // 创建新文件，写入 YAML 头部 + 首条记录
        let header = format!(
            r#"# 小鹤音形自定义词典，通过小鹤音形词典软件添加

---
name: flypy_custom
version: "1.0"
sort: by_weight
...

{}
"#,
            line
        );
        fs::write(&custom_path, &header).map_err(|e| format!("创建自定义词典文件失败: {}", e))?;
    } else {
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&custom_path)
            .map_err(|e| format!("打开文件失败: {}", e))?;

        let entry = format!("\n{}", line);
        file.write_all(entry.as_bytes())
            .map_err(|e| format!("写入文件失败: {}", e))?;
    }

    Ok(path_str)
}

/// 探测 Rime 用户目录下的所有 schema 文件，返回 (文件名, 完整路径) 列表。
/// 只在顶层目录扫，跳过 hidden / build / flypy.userdb。
pub fn find_schema_files(rime_user_dir: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir(rime_user_dir) else {
        return out;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with('.') || !name.ends_with(".schema.yaml") {
            continue;
        }
        // stem: "flypy.schema.yaml" -> "flypy"
        let stem = name.trim_end_matches(".schema.yaml").to_string();
        out.push((stem, path.to_string_lossy().to_string()));
    }
    // 稳定排序：flypy 优先，其次双拼，然后按字母序
    out.sort_by(|a, b| {
        let score = |s: &str| -> u8 {
            if s == "flypy" {
                0
            } else if s.contains("double_pinyin") || s.contains("flypy") {
                1
            } else {
                2
            }
        };
        score(&a.0).cmp(&score(&b.0)).then(a.0.cmp(&b.0))
    });
    out
}

/// 从 schema.yaml 文件中读取 translator/dictionary 的值。
/// 用纯文本扫描，找到 `translator:` 下的 `dictionary:` 行。
pub fn read_schema_dictionary(schema_path: &str) -> Option<String> {
    let content = fs::read_to_string(schema_path).ok()?;
    let mut in_translator = false;
    let mut translator_indent = 0;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - trimmed.len();
        if !in_translator {
            if trimmed == "translator:" {
                in_translator = true;
                translator_indent = indent;
            }
            continue;
        }
        // 遇到同级或更外层 key，退出 translator 段
        if indent <= translator_indent && !trimmed.is_empty() {
            break;
        }
        // 精确匹配 "dictionary:"，排除 "dictionaries:" 等
        if let Some(val) = trimmed.strip_prefix("dictionary:") {
            let val = val.trim();
            // 去掉行内注释（如 "flypy  # some comment"）
            let val = val.split('#').next().unwrap_or(val).trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// 迁移旧版 __patch: 格式为标准 patch: 格式。
/// 如果文件使用了 __patch:，将其改为 patch:，
/// 并确保 original_dict 在 translator/dictionary 列表中。
fn migrate_legacy_patch_key(content: &str, original_dict: &str) -> String {
    if !content.contains("__patch:") {
        return content.to_string();
    }
    // 先把 __patch: 替换为 patch:
    let fixed = content.replace("__patch:", "patch:");
    // 再检查 original_dict 是否在列表中，不在则追加
    if !custom_already_references(&fixed, original_dict) {
        if let Some(patched) = append_to_translator_dictionary(&fixed, original_dict) {
            return patched;
        }
    }
    fixed
}

/// 在指定 schema 同名 .custom.yaml 里幂等地追加 flypy_custom 到
/// `patch:/translator/dictionary` 列表。已存在则不修改。
/// 如果 `.custom.yaml` 不存在则创建。
/// `original_dict` 为 schema 原始的 translator/dictionary 名称（如 "flypy"），
/// 新建 .custom.yaml 时会同时列出原词典和 flypy_custom。
/// 成功返回 patch 后的 .custom.yaml 路径。
pub fn ensure_flypy_custom_in_schema(
    schema_path: &str,
    original_dict: &str,
) -> Result<String, String> {
    let schema = Path::new(schema_path);
    let Some(stem) = schema
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.trim_end_matches(".schema.yaml"))
    else {
        return Err(format!("无法解析 schema 文件名: {}", schema_path));
    };
    let Some(parent) = schema.parent() else {
        return Err(format!("无法获取 schema 父目录: {}", schema_path));
    };
    let custom_path = parent.join(format!("{}.custom.yaml", stem));
    let custom_str = custom_path.to_string_lossy().to_string();

    if custom_path.exists() {
        let original_content = fs::read_to_string(&custom_path)
            .map_err(|e| format!("读取 {} 失败: {}", custom_str, e))?;
        // 兼容旧版：如果用的是 __patch: 而非 patch:，迁移为标准格式
        let content = migrate_legacy_patch_key(&original_content, original_dict);
        // 确保 original_dict 和 flypy_custom 都在列表中
        let mut patched = content.clone();
        if !custom_already_references(&patched, original_dict) {
            patched = append_to_translator_dictionary(&patched, original_dict).unwrap_or(patched);
        }
        if !custom_already_references(&patched, "flypy_custom") {
            patched =
                append_to_translator_dictionary(&patched, "flypy_custom").ok_or_else(|| {
                    "无法在 translator/dictionary 列表中插入 flypy_custom".to_string()
                })?;
        }
        if patched != original_content {
            fs::write(&custom_path, &patched)
                .map_err(|e| format!("写入 {} 失败: {}", custom_str, e))?;
        }
        return Ok(custom_str);
    } else {
        // 新建一个标准 patch 文件
        let header = format!(
            "# {} 用户定制（自动生成，请勿手动删除 flypy_custom 段）\n\
             patch:\n\
             \x20\x20translator/dictionary:\n\
             \x20\x20\x20\x20- {}\n\
             \x20\x20\x20\x20- flypy_custom\n",
            stem, original_dict
        );
        fs::write(&custom_path, header).map_err(|e| format!("创建 {} 失败: {}", custom_str, e))?;
    }

    Ok(custom_str)
}

/// 确保指定词典文件的 YAML 头部包含 `import_tables: - flypy_custom`。
/// 这是 Windows 平台推荐的自定义词典合并方式——直接在主词典头声明 import，
/// 避免 .custom.yaml 中 translator/dictionary 数组导致 table_translator 编译失败。
/// 幂等：已存在则跳过。返回 dict 文件路径。
pub fn ensure_import_tables_in_dict(dict_path: &str) -> Result<String, String> {
    let path = Path::new(dict_path);
    if !path.exists() {
        return Err(format!("词典文件不存在: {}", dict_path));
    }

    let content =
        fs::read_to_string(dict_path).map_err(|e| format!("读取 {} 失败: {}", dict_path, e))?;

    // 已存在则幂等跳过
    if content.contains("import_tables:") && content.contains("flypy_custom") {
        return Ok(dict_path.to_string());
    }

    // 在 `sort: original` 之后插入 `import_tables:`
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut insert_at: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("sort:") {
            insert_at = Some(i + 1);
            break;
        }
    }

    if let Some(pos) = insert_at {
        // 确保 sort 行后有空行分隔
        if pos < lines.len() && lines[pos].trim().is_empty() {
            // 空行后插入
            lines.insert(pos + 1, format!("import_tables:\n  - flypy_custom"));
        } else {
            // 没有空行则补空行再插入
            lines.insert(pos, String::new());
            lines.insert(pos + 1, format!("import_tables:\n  - flypy_custom"));
        }
    } else {
        return Err(format!("未在 {} 中找到 sort: 字段", dict_path));
    }

    let trailing_nl = content.ends_with('\n');
    let mut out = lines.join("\n");
    if trailing_nl {
        out.push('\n');
    }

    fs::write(dict_path, &out).map_err(|e| format!("写入 {} 失败: {}", dict_path, e))?;

    Ok(dict_path.to_string())
}

/// 检查 YAML 内容里是否已在某个 translator/dictionary 列表里引用了指定 dict。
/// 用纯文本扫描，避免引入 serde_yaml 依赖。
/// 同时支持 `patch:` 和 `__patch:` 两种 key。
fn custom_already_references(content: &str, dict_name: &str) -> bool {
    let mut in_patch = false;
    let mut in_dict = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        if !in_patch && (trimmed == "patch:" || trimmed == "__patch:") {
            in_patch = true;
            continue;
        }
        // 遇下一个顶层 key 退出 patch
        if in_patch && !trimmed.is_empty() && !line.starts_with(' ') && !line.starts_with('\t') {
            in_patch = false;
            in_dict = false;
        }
        if in_patch && trimmed == "translator/dictionary:" {
            in_dict = true;
            continue;
        }
        // 下一个同级列表项跳出 dictionary
        if in_dict {
            if !line.starts_with(' ') && !line.starts_with('\t') && !trimmed.is_empty() {
                in_dict = false;
            } else if trimmed == format!("- {}", dict_name) {
                return true;
            }
        }
    }
    false
}

/// 在 YAML 文本中把指定 dict 追加到 `patch:/translator/dictionary` 列表。
/// 逻辑：找到 `patch:` 或 `__patch:` 段、找到 `translator/dictionary:` 子项、找到列表结束位置插入。
/// 如果没有 patch 段则追加在文件末尾。
fn append_to_translator_dictionary(content: &str, dict_name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut patch_start: Option<usize> = None;
    let mut dict_start: Option<usize> = None;
    let mut dict_indent: usize = 0;
    let mut dict_end: Option<usize> = None;
    let mut list_indent: Option<usize> = None;
    let mut patch_end: Option<usize> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        if patch_start.is_none() {
            if trimmed == "patch:" || trimmed == "__patch:" {
                patch_start = Some(i);
            }
            continue;
        }
        // 已经在 patch 段内
        if dict_start.is_some() && dict_end.is_none() {
            // 还在 dict 列表里收集
            if let Some(li) = list_indent {
                if indent == li && trimmed.starts_with("- ") {
                    // 仍是列表项
                } else {
                    // 第一个非列表项的行——既是 dict_end 又是 patch_end
                    dict_end = Some(i);
                    patch_end = Some(i);
                }
            } else {
                // dict_start 已设但 list_indent 还没设——第一个非空行
                if trimmed.starts_with("- ") {
                    list_indent = Some(indent);
                } else {
                    // 空字典列表
                    dict_end = Some(i);
                    patch_end = Some(i);
                }
            }
        } else if patch_end.is_none() {
            // dict_start 未设 或 dict_end 已设且 patch 还没结束
            if indent == 0
                && trimmed != "patch:"
                && trimmed != "__patch:"
                && !trimmed.starts_with("translator/dictionary:")
            {
                // patch 段结束
                patch_end = Some(i);
            }
        }
        if dict_start.is_none() && trimmed == "translator/dictionary:" {
            dict_start = Some(i);
            dict_indent = indent;
            continue;
        }
    }

    // 如果 dict_start 已设但 dict_end 没设（列表延伸到文件末尾）——追加在末尾
    if dict_start.is_some() && dict_end.is_none() {
        dict_end = Some(lines.len());
    }

    if let (Some(_ds), Some(de)) = (dict_start, dict_end) {
        // 在 de 之前插入
        let indent_str = " ".repeat(dict_indent + 2);
        let new_line = format!("{}{}- {}", indent_str, "", dict_name);
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        new_lines.insert(de, new_line);
        // 保留尾部换行
        let trailing_nl = content.ends_with('\n');
        let mut out = new_lines.join("\n");
        if trailing_nl {
            out.push('\n');
        }
        Some(out)
    } else if let Some(ps) = patch_start {
        // patch 段存在但没有 dictionary 项——在 patch 段后追加
        let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
        new_lines.insert(ps + 1, "  translator/dictionary:".to_string());
        new_lines.insert(ps + 2, format!("    - {}", dict_name));
        let trailing_nl = content.ends_with('\n');
        let mut out = new_lines.join("\n");
        if trailing_nl {
            out.push('\n');
        }
        Some(out)
    } else {
        // 完全没有 patch 段——追加在文件末尾
        let mut out = content.to_string();
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(&format!(
            "\npatch:\n  translator/dictionary:\n    - {}\n",
            dict_name
        ));
        Some(out)
    }
}

/// 跨平台触发 rime 重新部署。
/// macOS: 调 /Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel --reload
/// Windows: 调 weasel-deployer.exe（探测多个常见路径）
/// 其他: 返回 Err，让上层提示用户手动重新部署。
pub fn trigger_rime_deploy() -> Result<(), String> {
    if cfg!(target_os = "macos") {
        let path = "/Library/Input Methods/Squirrel.app/Contents/MacOS/Squirrel";
        if !Path::new(path).exists() {
            return Err(format!("未找到 Squirrel: {}", path));
        }
        let out = Command::new(path)
            .arg("--reload")
            .output()
            .map_err(|e| format!("调用 Squirrel --reload 失败: {}", e))?;
        if !out.status.success() {
            return Err(format!(
                "Squirrel --reload 退出码 {:?}: {}",
                out.status.code(),
                String::from_utf8_lossy(&out.stderr)
            ));
        }
        return Ok(());
    }

    if cfg!(target_os = "windows") {
        if let Some(p) = find_weasel_deployer() {
            // 路径含空格，必须 quotes
            let status = Command::new(&p)
                .arg("/deploy")
                .status()
                .or_else(|_| Command::new(&p).status())
                .map_err(|e| format!("调用 weasel-deployer 失败: {}", e))?;
            if !status.success() {
                return Err(format!("weasel-deployer 退出码 {:?}", status.code()));
            }
            return Ok(());
        }
        return Err(
            "未找到 weasel-deployer.exe，请确认小狼毫已安装；可手动在托盘点「重新部署」"
                .to_string(),
        );
    }

    Err("当前平台未实现自动重新部署，请在输入法托盘手动点「重新部署」".to_string())
}

/// 探测 Windows 小狼毫 weasel-deployer.exe 路径。
fn find_weasel_deployer() -> Option<PathBuf> {
    let candidates = [
        // 版本号子目录（最常见）
        r"C:\Program Files\Rime\weasel-0.17.4",
        r"C:\Program Files\Rime\weasel-0.17.4\",
        r"C:\Program Files\Rime\weasel",
        r"C:\Program Files (x86)\Rime\weasel",
        r"C:\Program Files\Rime",
        r"C:\Program Files (x86)\Rime",
    ];
    // 支持多种文件名（NTFS 不区分大小写，但精确匹配更安全）
    let deployer_names = [
        "WeaselDeployer.exe",
        "weasel-deployer.exe",
        "weaselDeployer.exe",
    ];
    for c in &candidates {
        for name in &deployer_names {
            let p = Path::new(c).join(name);
            if p.exists() {
                return Some(p);
            }
        }
    }
    // 扫描 C:\Program Files\Rime 下的版本子目录
    if let Ok(entries) = fs::read_dir(r"C:\Program Files\Rime") {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                for name in &deployer_names {
                    let exe = p.join(name);
                    if exe.exists() {
                        return Some(exe);
                    }
                }
            }
        }
    }
    // 注册表探测
    #[cfg(target_os = "windows")]
    {
        if let Some(p) = find_weasel_via_registry() {
            return Some(p);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn find_weasel_via_registry() -> Option<PathBuf> {
    // 支持 HKLM（系统级安装）和 HKCU（用户级安装）
    for hive in &["HKLM", "HKCU"] {
        let key = format!("{}\\SOFTWARE\\Rime\\Weasel", hive);
        let out = Command::new("reg")
            .args(["query", &key, "/v", "InstallDir"])
            .output()
            .ok()?;
        if !out.status.success() {
            continue;
        }
        let s = String::from_utf8_lossy(&out.stdout);
        // 输出形如:  "    InstallDir    REG_SZ    C:\Program Files\Rime\weasel"
        // 路径在 "REG_SZ" 之后，需提取完整路径（可能含空格）
        for line in s.lines() {
            if !line.contains("InstallDir") {
                continue;
            }
            // 找到 "REG_SZ" 标记，取其后的完整路径
            if let Some(reg_sz_pos) = line.find("REG_SZ") {
                let path_part = line[reg_sz_pos + 6..].trim();
                if !path_part.is_empty() {
                    let p = Path::new(path_part);
                    if p.exists() {
                        // 探测 deployer.exe
                        for name in &["WeaselDeployer.exe", "weasel-deployer.exe"] {
                            let exe = p.join(name);
                            if exe.exists() {
                                return Some(exe);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// 词典文件信息
#[derive(Debug, Clone)]
pub struct DictFileInfo {
    pub path: String,
    pub name: String,
    pub entry_count: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rime_content_with_yaml_header() {
        let loader = RimeLoader::new("/tmp");
        let content = r#"# Rime dictionary

---
name: test
version: "1.0.0"
...

阿	aaek
锕	aajk
"#;

        let entries = loader.parse_rime_content(content, "test").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "阿");
        assert_eq!(entries[0].code, "aaek");
        assert_eq!(entries[0].source, "test");
        assert_eq!(entries[1].text, "锕");
        assert_eq!(entries[1].code, "aajk");
    }

    #[test]
    fn test_parse_rime_content_without_yaml_header() {
        let loader = RimeLoader::new("/tmp");
        let content = r#"阿	aaek
锕	aajk
"#;

        let entries = loader.parse_rime_content(content, "test").unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "阿");
        assert_eq!(entries[0].code, "aaek");
    }

    #[test]
    fn test_parse_rime_content_with_comments() {
        let loader = RimeLoader::new("/tmp");
        let content = r#"# 这是注释
阿	aaek
# 另一个注释
锕	aajk
"#;

        let entries = loader.parse_rime_content(content, "test").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_parse_rime_content_with_empty_lines() {
        let loader = RimeLoader::new("/tmp");
        let content = r#"阿	aaek

锕	aajk

"#;

        let entries = loader.parse_rime_content(content, "test").unwrap();
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_parse_rime_content_with_dots_header() {
        let loader = RimeLoader::new("/tmp");
        let content = r#"---
name: test
...

阿	aaek
"#;

        let entries = loader.parse_rime_content(content, "test").unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].text, "阿");
    }

    #[test]
    fn test_custom_already_references() {
        let content = r#"patch:
  translator/dictionary:
    - flypy
    - flypydz
    - flypy_custom
  other_key: yes
"#;
        assert!(custom_already_references(content, "flypy_custom"));
        assert!(!custom_already_references(content, "missing"));
    }

    #[test]
    fn test_custom_already_references_no_patch() {
        let content = "translator:\n  dictionary: flypy\n";
        assert!(!custom_already_references(content, "flypy_custom"));
    }

    #[test]
    fn test_append_to_translator_dictionary_existing_list() {
        let content = "patch:\n  translator/dictionary:\n    - flypy\n    - flypydz\n";
        let out = append_to_translator_dictionary(content, "flypy_custom").unwrap();
        assert!(out.contains("- flypy_custom"));
        // 幂等原列表顺序不变
        let flypy_pos = out.find("- flypy").unwrap();
        let custom_pos = out.find("- flypy_custom").unwrap();
        assert!(flypy_pos < custom_pos);
    }

    #[test]
    fn test_append_to_translator_dictionary_no_patch() {
        let content = "translator:\n  dictionary: flypy\n";
        let out = append_to_translator_dictionary(content, "flypy_custom").unwrap();
        assert!(out.contains("patch:"));
        assert!(out.contains("translator/dictionary:"));
        assert!(out.contains("- flypy_custom"));
    }

    #[test]
    fn test_append_to_translator_dictionary_patch_without_dict() {
        let content = "patch:\n  other_key: 1\n";
        let out = append_to_translator_dictionary(content, "flypy_custom").unwrap();
        assert!(out.contains("translator/dictionary:"));
        assert!(out.contains("- flypy_custom"));
    }

    #[test]
    fn test_ensure_flypy_custom_in_schema_creates_custom_file() {
        let tmp = std::env::temp_dir().join("xhyxd_test_ensure_creates");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let schema = tmp.join("flypy.schema.yaml");
        fs::write(
            &schema,
            "schema:\n  schema_id: flypy\ntranslator:\n  dictionary: flypy\n",
        )
        .unwrap();
        let custom = ensure_flypy_custom_in_schema(&schema.to_string_lossy(), "flypy").unwrap();
        assert!(Path::new(&custom).exists());
        let body = fs::read_to_string(&custom).unwrap();
        assert!(body.contains("- flypy_custom"));
        assert!(body.contains("- flypy"));
        assert!(body.contains("patch:"));
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn test_ensure_flypy_custom_in_schema_idempotent() {
        let tmp = std::env::temp_dir().join("xhyxd_test_ensure_idem");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let schema = tmp.join("flypy.schema.yaml");
        fs::write(&schema, "translator:\n  dictionary: flypy\n").unwrap();
        let path_str = schema.to_string_lossy().to_string();

        // 第一次写入
        let custom = ensure_flypy_custom_in_schema(&path_str, "flypy").unwrap();
        let body1 = fs::read_to_string(&custom).unwrap();
        // 第二次应该幂等
        let custom2 = ensure_flypy_custom_in_schema(&path_str, "flypy").unwrap();
        let body2 = fs::read_to_string(&custom2).unwrap();
        assert_eq!(body1, body2, "重复调用必须幂等");
        // 统计 - flypy_custom 只出现一次
        assert_eq!(body2.matches("- flypy_custom").count(), 1);
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn test_find_schema_files_filters_hidden() {
        let tmp = std::env::temp_dir().join("xhyxd_test_find_schema");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        fs::write(tmp.join("flypy.schema.yaml"), "schema: {}").unwrap();
        fs::write(tmp.join("double_pinyin_flypy.schema.yaml"), "schema: {}").unwrap();
        fs::write(tmp.join(".hidden.schema.yaml"), "schema: {}").unwrap();
        fs::write(tmp.join("japanese.schema.yaml"), "schema: {}").unwrap();
        let found = find_schema_files(&tmp.to_string_lossy());
        let names: Vec<&str> = found.iter().map(|(n, _)| n.as_str()).collect();
        // flypy 必须排第一
        assert_eq!(names[0], "flypy");
        assert!(names.contains(&"double_pinyin_flypy"));
        assert!(names.contains(&"japanese"));
        // hidden 不在结果里
        assert!(!names.contains(&".hidden"));
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn test_read_schema_dictionary() {
        let tmp = std::env::temp_dir().join("xhyxd_test_read_schema_dict");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let schema = tmp.join("flypy.schema.yaml");
        fs::write(
            &schema,
            "schema:\n  schema_id: flypy\ntranslator:\n  dictionary: flypy\n",
        )
        .unwrap();
        let dict = read_schema_dictionary(&schema.to_string_lossy()).unwrap();
        assert_eq!(dict, "flypy");
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn test_read_schema_dictionary_missing() {
        let dict = read_schema_dictionary("/nonexistent/path/schema.yaml");
        assert!(dict.is_none());
    }

    #[test]
    fn test_custom_already_references_legacy_patch() {
        let content = r#"__patch:
  translator/dictionary:
    - flypy
    - flypy_custom
"#;
        assert!(custom_already_references(content, "flypy_custom"));
    }

    #[test]
    fn test_migrate_legacy_patch_key() {
        let content = "__patch:\n  translator/dictionary:\n    - flypy_custom\n";
        let migrated = migrate_legacy_patch_key(content, "flypy");
        assert!(migrated.contains("patch:"));
        assert!(!migrated.contains("__patch:"));
        assert!(migrated.contains("- flypy"));
        assert!(migrated.contains("- flypy_custom"));
    }

    #[test]
    fn test_migrate_legacy_patch_key_noop() {
        let content = "patch:\n  translator/dictionary:\n    - flypy_custom\n";
        let migrated = migrate_legacy_patch_key(content, "flypy");
        // 已经是 patch: 格式，不需迁移，原样返回
        assert_eq!(migrated, content);
    }

    #[test]
    fn test_ensure_flypy_custom_migrates_legacy_file() {
        let tmp = std::env::temp_dir().join("xhyxd_test_migrate_legacy");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let schema = tmp.join("flypy.schema.yaml");
        fs::write(&schema, "translator:\n  dictionary: flypy\n").unwrap();
        // 模拟旧版生成的 .custom.yaml（__patch: 格式）
        let custom = tmp.join("flypy.custom.yaml");
        fs::write(
            &custom,
            "__patch:\n  translator/dictionary:\n    - flypy_custom\n",
        )
        .unwrap();
        // 调用后应该迁移为 patch: 并补上 flypy
        let result = ensure_flypy_custom_in_schema(&schema.to_string_lossy(), "flypy").unwrap();
        let body = fs::read_to_string(&result).unwrap();
        assert!(body.contains("patch:"));
        assert!(!body.contains("__patch:"));
        assert!(body.contains("- flypy"));
        assert!(body.contains("- flypy_custom"));
        fs::remove_dir_all(&tmp).ok();
    }

    #[test]
    fn test_ensure_flypy_custom_preserves_existing_dict() {
        let tmp = std::env::temp_dir().join("xhyxd_test_preserve_dict");
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).unwrap();
        let schema = tmp.join("flypy.schema.yaml");
        fs::write(&schema, "translator:\n  dictionary: flypy\n").unwrap();
        // 模拟已有 .custom.yaml 有 patch: 但没有 translator/dictionary:
        let custom = tmp.join("flypy.custom.yaml");
        fs::write(&custom, "patch:\n  menu/page_size: 9\n").unwrap();
        let result = ensure_flypy_custom_in_schema(&schema.to_string_lossy(), "flypy").unwrap();
        let body = fs::read_to_string(&result).unwrap();
        // 必须同时包含 flypy 和 flypy_custom
        assert!(body.contains("- flypy"));
        assert!(body.contains("- flypy_custom"));
        // 保留原有设置
        assert!(body.contains("menu/page_size: 9"));
        fs::remove_dir_all(&tmp).ok();
    }
}
