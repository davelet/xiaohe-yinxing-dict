use crate::config::AppConfig;
use crate::dict::ExternalDictEntry;
use crate::rime_loader::RimeLoader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

pub(crate) type ExternalLoadResult = (
    Vec<ExternalDictEntry>,
    HashMap<String, SystemTime>,
    Vec<String>,
    HashMap<String, usize>,
);

type ExternalCacheResult = (
    Vec<ExternalDictEntry>,
    HashMap<String, SystemTime>,
    HashMap<String, usize>,
);

/// 外部词典缓存（序列化为 JSON）
#[derive(Serialize, Deserialize)]
struct ExternalDictCache {
    /// 文件路径 → 缓存时的修改时间（秒）
    file_mtimes: HashMap<String, u64>,
    /// 所有已加载的外部词典条目
    entries: Vec<ExternalDictEntry>,
    /// 文件路径 → 条目数
    #[serde(default)]
    entry_counts: HashMap<String, usize>,
}

fn cache_path() -> Option<PathBuf> {
    let p = AppConfig::config_path()?;
    Some(p.parent()?.join("external_dicts.cache"))
}

/// 尝试从缓存加载条目（校验所有文件 mtime）
/// 返回 (entries, file_mtimes, entry_counts)
fn try_load_external_cache(config: &AppConfig) -> Option<ExternalCacheResult> {
    let cache_path = cache_path()?;
    let data = std::fs::read_to_string(cache_path).ok()?;
    let cache: ExternalDictCache = serde_json::from_str(&data).ok()?;

    // 检查所有文件（启用+禁用）的修改时间
    for dict_file in config.external_dict_files.iter() {
        let meta = std::fs::metadata(&dict_file.path).ok()?;
        let mtime = meta.modified().ok()?;
        let secs = mtime.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
        if cache.file_mtimes.get(&dict_file.path) != Some(&secs) {
            return None;
        }
    }

    // 检查缓存中没有多余/缺失的文件
    let cached_paths: std::collections::HashSet<&str> =
        cache.file_mtimes.keys().map(|s| s.as_str()).collect();
    let config_paths: std::collections::HashSet<&str> = config
        .external_dict_files
        .iter()
        .map(|f| f.path.as_str())
        .collect();
    if cached_paths != config_paths {
        return None;
    }

    // 缓存有效，转换 mtime 为 SystemTime
    let file_mtimes: HashMap<String, SystemTime> = cache
        .file_mtimes
        .iter()
        .map(|(path, secs)| {
            let duration = std::time::Duration::from_secs(*secs);
            let mtime = std::time::UNIX_EPOCH + duration;
            (path.clone(), mtime)
        })
        .collect();

    Some((cache.entries, file_mtimes, cache.entry_counts))
}

/// 保存外部词典缓存
pub(crate) fn save_external_cache(
    entries: &[ExternalDictEntry],
    file_mtimes: &HashMap<String, SystemTime>,
    entry_counts: &HashMap<String, usize>,
) {
    let Some(cache_path) = cache_path() else {
        return;
    };
    let mtimes: HashMap<String, u64> = file_mtimes
        .iter()
        .filter_map(|(path, mtime)| {
            mtime
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| (path.clone(), d.as_secs()))
        })
        .collect();

    let cache = ExternalDictCache {
        file_mtimes: mtimes,
        entries: entries.to_vec(),
        entry_counts: entry_counts.clone(),
    };

    if let Ok(data) = serde_json::to_string(&cache) {
        let _ = std::fs::write(&cache_path, data);
    }
}

/// 加载外部词典条目（优先使用缓存）
/// 返回的 entries 仅包含已启用文件，file_mtimes 包含所有文件
pub(crate) fn load_external_entries(
    config: &AppConfig,
    rime_loader: &RimeLoader,
) -> ExternalLoadResult {
    // 辅助函数：从 entries 中过滤出已启用文件对应的条目
    let filter_enabled = |entries: &[ExternalDictEntry]| -> Vec<ExternalDictEntry> {
        let enabled_sources: std::collections::HashSet<String> = config
            .enabled_external_dicts()
            .iter()
            .filter_map(|f| {
                std::path::Path::new(&f.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.replace(".dict.yaml", "").replace(".txt", ""))
            })
            .collect();
        entries
            .iter()
            .filter(|e| enabled_sources.contains(e.source.as_str()))
            .cloned()
            .collect()
    };

    if let Some((all_entries, mtimes, entry_counts)) = try_load_external_cache(config) {
        let entries = filter_enabled(&all_entries);
        return (entries, mtimes, Vec::new(), entry_counts);
    }

    // 缓存失效，重新从所有文件加载（包含禁用的，缓存需要它们的数据）
    let mut all_entries = Vec::new();
    let mut mtimes = HashMap::new();
    let mut load_errors = Vec::new();
    let mut entry_counts = HashMap::new();

    for dict_file in config.external_dict_files.iter() {
        match rime_loader.load_dict_file(&dict_file.path) {
            Ok(loaded) => {
                entry_counts.insert(dict_file.path.clone(), loaded.len());
                all_entries.extend(loaded);
                if let Ok(metadata) = std::fs::metadata(&dict_file.path)
                    && let Ok(mtime) = metadata.modified()
                {
                    mtimes.insert(dict_file.path.clone(), mtime);
                }
            }
            Err(_) => load_errors.push(dict_file.path.clone()),
        }
    }

    let entries = filter_enabled(&all_entries);

    // 保存缓存供下次启动使用
    save_external_cache(&all_entries, &mtimes, &entry_counts);
    (entries, mtimes, load_errors, entry_counts)
}
