use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 外部词典文件配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDictFile {
    pub path: String,
    pub name: String,
    pub is_enabled: bool,
    pub entry_count: Option<usize>,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub rime_user_dir: String,
    pub external_dict_files: Vec<ExternalDictFile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        let rime_user_dir = dirs::home_dir()
            .map(|home| home.join("Library/Rime").to_string_lossy().to_string())
            .unwrap_or_else(|| "~/Library/Rime".to_string());

        Self {
            rime_user_dir,
            external_dict_files: Vec::new(),
        }
    }
}

impl AppConfig {
    /// 获取配置文件路径
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|config_dir| {
            let app_dir = config_dir.join("xiaohe-yinxing-dict");
            fs::create_dir_all(&app_dir).ok();
            app_dir.join("config.json")
        })
    }

    /// 加载配置
    pub fn load() -> Self {
        Self::config_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    /// 保存配置
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("无法获取配置文件路径")?;
        let contents =
            serde_json::to_string_pretty(self).map_err(|e| format!("序列化配置失败: {}", e))?;
        fs::write(path, contents).map_err(|e| format!("写入配置文件失败: {}", e))
    }

    /// 添加外部词典文件
    pub fn add_external_dict(&mut self, path: String, name: String) {
        if !self.external_dict_files.iter().any(|f| f.path == path) {
            self.external_dict_files.push(ExternalDictFile {
                path,
                name,
                is_enabled: true,
                entry_count: None,
            });
        }
    }

    /// 移除外部词典文件
    pub fn remove_external_dict(&mut self, path: &str) {
        self.external_dict_files.retain(|f| f.path != path);
    }

    /// 切换外部词典文件启用状态
    pub fn toggle_external_dict(&mut self, path: &str) {
        if let Some(file) = self.external_dict_files.iter_mut().find(|f| f.path == path) {
            file.is_enabled = !file.is_enabled;
        }
    }

    /// 获取启用的外部词典文件
    pub fn enabled_external_dicts(&self) -> Vec<&ExternalDictFile> {
        self.external_dict_files
            .iter()
            .filter(|f| f.is_enabled)
            .collect()
    }

    /// 移除不在默认扫描目录下的所有词典（手动添加的）
    pub fn remove_manual_dicts(&mut self, rime_user_dir: &str) {
        let rime_dir = std::path::Path::new(rime_user_dir);
        self.external_dict_files.retain(|f| {
            let path = std::path::Path::new(&f.path);
            path.starts_with(rime_dir)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert!(!config.rime_user_dir.is_empty());
        assert!(config.external_dict_files.is_empty());
    }

    #[test]
    fn test_add_external_dict() {
        let mut config = AppConfig::default();
        config.add_external_dict(
            "/Users/test/Library/Rime/flypydz.dict.yaml".to_string(),
            "flypydz".to_string(),
        );
        assert_eq!(config.external_dict_files.len(), 1);
        assert!(config.external_dict_files[0].is_enabled);
    }

    #[test]
    fn test_remove_external_dict() {
        let mut config = AppConfig::default();
        config.add_external_dict(
            "/Users/test/Library/Rime/flypydz.dict.yaml".to_string(),
            "flypydz".to_string(),
        );
        config.remove_external_dict("/Users/test/Library/Rime/flypydz.dict.yaml");
        assert!(config.external_dict_files.is_empty());
    }

    #[test]
    fn test_toggle_external_dict() {
        let mut config = AppConfig::default();
        config.add_external_dict(
            "/Users/test/Library/Rime/flypydz.dict.yaml".to_string(),
            "flypydz".to_string(),
        );
        assert!(config.external_dict_files[0].is_enabled);
        config.toggle_external_dict("/Users/test/Library/Rime/flypydz.dict.yaml");
        assert!(!config.external_dict_files[0].is_enabled);
    }
}
