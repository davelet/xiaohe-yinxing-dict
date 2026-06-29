use crate::dict::{Category, ExternalDictEntry};
use std::fs;
use std::path::Path;

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

            // 解析 TSV 格式：文字\t编码
            if let Some((text, code)) = trimmed.split_once('\t') {
                let text = text.trim().to_string();
                let code = code.trim().to_string();

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
}
