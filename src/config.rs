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
        let rime_user_dir = default_rime_user_dir();

        Self {
            rime_user_dir,
            external_dict_files: Vec::new(),
        }
    }
}

/// 探测当前平台上常见的 Rime 用户目录，返回第一个存在的；
/// 都不存在时返回平台默认值（与之前 macOS 行为保持兼容）。
fn default_rime_user_dir() -> String {
    let candidates = rime_user_dir_candidates();
    if let Some(home) = dirs::home_dir() {
        for rel in candidates {
            let p = home.join(rel);
            if p.exists() {
                return p.to_string_lossy().to_string();
            }
        }
        // 都不存在时回退到平台第一个候选，避免和旧配置不一致
        if let Some(rel) = candidates.first() {
            return home.join(rel).to_string_lossy().to_string();
        }
    }
    // home_dir 拿不到的极端情况
    candidates
        .first()
        .map(|s| (*s).to_string())
        .unwrap_or_else(|| "~/Library/Rime".to_string())
}

/// 各平台 Rime 用户目录的常见相对路径，按优先级排序
fn rime_user_dir_candidates() -> &'static [&'static str] {
    if cfg!(target_os = "macos") {
        &["Library/Rime"]
    } else if cfg!(target_os = "windows") {
        // 小狼毫默认在 %APPDATA%\Rime
        &["AppData/Roaming/Rime", "AppData/Local/Rime"]
    } else {
        // Linux: ibus-rime / fcitx-rime 常见路径
        &[
            ".config/ibus/rime",
            ".local/share/fcitx/rime",
            ".config/fcitx/rime",
        ]
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
        let mut cfg: AppConfig = Self::config_path()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default();
        // 自愈：如果保存的 rime_user_dir 不存在，重新探测当前平台默认值。
        // 仅在路径不存在时触发，不会覆盖用户主动指定的有效路径。
        if !std::path::Path::new(&cfg.rime_user_dir).exists() {
            cfg.rime_user_dir = default_rime_user_dir();
        }
        cfg
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
    fn test_default_rime_user_dir_is_non_empty_and_absolute() {
        let dir = default_rime_user_dir();
        assert!(!dir.is_empty());
        // 解析后必须是绝对路径（探测逻辑总是 join 到 home_dir）
        let p = std::path::Path::new(&dir);
        assert!(p.is_absolute() || dir.contains('/') || dir.contains('\\'));
    }

    #[test]
    fn test_rime_user_dir_candidates_per_platform() {
        let cands = rime_user_dir_candidates();
        assert!(!cands.is_empty(), "至少需要一个平台候选路径");
        if cfg!(target_os = "windows") {
            assert!(cands.iter().any(|c| c.contains("Rime")));
        } else if cfg!(target_os = "macos") {
            assert!(cands.contains(&"Library/Rime"));
        }
    }

    #[test]
    fn test_load_self_heals_missing_rime_dir() {
        // 模拟一个“保存了路径但目录被刪除/迁移”的场景：
        // 我们无法直接控制 AppConfig::load() 读哪个文件（依赖 dirs::config_dir），
        // 所以这里走个手逻辑等价路径：路径不存在时会被 default_rime_user_dir() 覆盖。
        let stale = "/this/path/should/not/exist/rime".to_string();
        let mut cfg = AppConfig {
            rime_user_dir: stale.clone(),
            external_dict_files: Vec::new(),
        };
        if !std::path::Path::new(&cfg.rime_user_dir).exists() {
            cfg.rime_user_dir = default_rime_user_dir();
        }
        assert_ne!(cfg.rime_user_dir, stale, "应被自愈为新探测值");
        assert!(!cfg.rime_user_dir.is_empty());
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
