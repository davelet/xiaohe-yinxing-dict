use crate::config::AppConfig;
use crate::dict::ExternalDictEntry;
use crate::rime_loader::{DictFileInfo, RimeLoader};
use crate::search;
use std::collections::HashMap;
use std::time::SystemTime;

/// 排序字段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortField {
    /// 默认按相关度排序
    Default,
    /// 按文字排序
    Text,
    /// 按编码排序
    Code,
}

/// 排序顺序
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// 视图模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// 默认数据（内置词典）
    Dict,
    /// 输入法数据（本地 Rime 词典）
    Manager,
}

/// 输入法数据视图状态（与默认数据视图完全独立，不共享数据）
pub struct ManagerState {
    /// 应用配置
    pub config: AppConfig,
    /// Rime 加载器
    pub rime_loader: RimeLoader,
    /// 扫描到的词典文件
    pub discovered_files: Vec<DictFileInfo>,
    /// 已加载的外部词典条目
    pub external_entries: Vec<ExternalDictEntry>,
    /// 外部条目搜索引擎
    pub external_engine: search::SearchEngine<ExternalDictEntry>,
    /// 外部条目搜索关键词
    pub external_query: String,
    /// 搜索关键词是否发生变化
    pub search_dirty: bool,
    /// 是否需要在进入视图时自动聚焦搜索框（仅首帧）
    pub search_auto_focus: bool,
    /// 外部条目搜索结果
    pub external_search_results: Vec<(usize, search::MatchKind)>,
    /// 外部条目搜索结果总数
    pub external_total_results: usize,
    /// 排序字段
    pub sort_field: SortField,
    /// 排序顺序
    pub sort_order: SortOrder,
    /// 新增文件路径输入
    pub new_file_path: String,
    /// 加载状态消息
    pub status_message: Option<String>,
    /// 状态消息倒计时（秒）
    pub status_timer: f32,
    /// 复制反馈
    pub external_copied_feedback: Option<(usize, crate::CopyKind)>,
    /// 复制反馈计时
    pub external_feedback_timer: f32,
    /// 初始加载失败的文件路径
    pub load_errors: Vec<String>,
    /// 已加载文件的修改时间（用于检测文件变更）
    pub file_mtimes: HashMap<String, SystemTime>,
}

impl Default for ManagerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ManagerState {
    /// 创建新的输入法数据视图状态
    pub fn new() -> Self {
        let config = AppConfig::load();
        let rime_loader = RimeLoader::new(&config.rime_user_dir);
        let discovered_files = rime_loader.scan_dict_files();

        // 加载已启用的外部词典
        let mut external_entries = Vec::new();
        let mut load_errors = Vec::new();
        let mut file_mtimes = HashMap::new();
        for dict_file in config.enabled_external_dicts() {
            match rime_loader.load_dict_file(&dict_file.path) {
                Ok(entries) => {
                    external_entries.extend(entries);
                    // 记录文件修改时间
                    if let Ok(metadata) = std::fs::metadata(&dict_file.path)
                        && let Ok(mtime) = metadata.modified()
                    {
                        file_mtimes.insert(dict_file.path.clone(), mtime);
                    }
                }
                Err(_) => load_errors.push(dict_file.path.clone()),
            }
        }

        let external_engine = search::SearchEngine::build(&external_entries);

        Self {
            config,
            rime_loader,
            discovered_files,
            external_entries,
            external_engine,
            external_query: String::new(),
            search_dirty: true,
            search_auto_focus: true,
            external_search_results: Vec::new(),
            external_total_results: 0,
            sort_field: SortField::Default,
            sort_order: SortOrder::Ascending,
            new_file_path: String::new(),
            status_message: None,
            status_timer: 0.0,
            external_copied_feedback: None,
            external_feedback_timer: 0.0,
            load_errors,
            file_mtimes,
        }
    }

    /// 添加外部词典文件
    pub fn add_external_dict(&mut self, path: String, name: String) {
        self.config.add_external_dict(path.clone(), name);
        if let Err(e) = self.config.save() {
            self.set_status(format!("保存配置失败: {}", e));
            return;
        }

        // 加载新添加的文件
        if let Ok(entries) = self.rime_loader.load_dict_file(&path) {
            let count = entries.len();
            if let Some(file) = self
                .config
                .external_dict_files
                .iter_mut()
                .find(|f| f.path == path)
            {
                file.entry_count = Some(count);
            }
            self.external_entries.extend(entries);
            // 记录文件修改时间
            if let Ok(metadata) = std::fs::metadata(&path)
                && let Ok(mtime) = metadata.modified()
            {
                self.file_mtimes.insert(path.clone(), mtime);
            }
            self.rebuild_external_engine();
            self.set_status(format!("成功加载 {} 条外部词典条目", count));
        } else {
            self.load_errors.push(path.clone());
            self.set_status(format!("加载文件失败: {}", path));
        }
    }

    /// 移除外部词典文件
    pub fn remove_external_dict(&mut self, path: &str) {
        self.config.remove_external_dict(path);
        if let Err(e) = self.config.save() {
            self.set_status(format!("保存配置失败: {}", e));
            return;
        }

        // 重新加载所有外部词典
        self.reload_external_dicts();
        self.set_status(format!("已移除外部词典: {}", path));
    }

    /// 切换外部词典启用状态
    pub fn toggle_external_dict(&mut self, path: &str) {
        self.config.toggle_external_dict(path);
        if let Err(e) = self.config.save() {
            self.set_status(format!("保存配置失败: {}", e));
            return;
        }

        // 重新加载所有外部词典
        self.reload_external_dicts();
    }

    /// 重新加载所有外部词典
    pub fn reload_external_dicts(&mut self) {
        self.external_entries.clear();
        self.file_mtimes.clear();
        self.load_errors.clear();
        let enabled_paths: Vec<String> = self
            .config
            .enabled_external_dicts()
            .iter()
            .map(|f| f.path.clone())
            .collect();

        for path in &enabled_paths {
            if let Ok(entries) = self.rime_loader.load_dict_file(path) {
                let count = entries.len();
                if let Some(file) = self
                    .config
                    .external_dict_files
                    .iter_mut()
                    .find(|f| &f.path == path)
                {
                    file.entry_count = Some(count);
                }
                self.external_entries.extend(entries);
                // 更新文件修改时间
                if let Ok(metadata) = std::fs::metadata(path)
                    && let Ok(mtime) = metadata.modified()
                {
                    self.file_mtimes.insert(path.clone(), mtime);
                }
            } else {
                self.load_errors.push(path.clone());
            }
        }

        self.rebuild_external_engine();
    }

    /// 检查文件是否有变更，如果有则重新加载
    pub fn check_and_reload_changed_files(&mut self) -> bool {
        let mut changed = false;
        let enabled_paths: Vec<String> = self
            .config
            .enabled_external_dicts()
            .iter()
            .map(|f| f.path.clone())
            .collect();

        for path in &enabled_paths {
            if let Ok(metadata) = std::fs::metadata(path)
                && let Ok(mtime) = metadata.modified()
            {
                let stored_mtime = self.file_mtimes.get(path);
                if stored_mtime.is_none() || stored_mtime.unwrap() != &mtime {
                    changed = true;
                    break;
                }
            }
        }

        if changed {
            self.reload_external_dicts();
            self.set_status("检测到词典文件变更，已自动重新加载");
        }

        changed
    }

    /// 重建外部条目搜索引擎
    fn rebuild_external_engine(&mut self) {
        self.external_engine = search::SearchEngine::build(&self.external_entries);
        self.external_query.clear();
        self.external_search_results.clear();
        self.external_total_results = 0;
    }

    /// 对搜索结果进行排序
    pub fn sort_search_results(&mut self) {
        let entries = &self.external_entries;
        let sort_field = self.sort_field;
        let sort_order = self.sort_order;

        self.external_search_results.sort_by(|a, b| {
            let entry_a = &entries[a.0];
            let entry_b = &entries[b.0];

            let cmp = match sort_field {
                SortField::Default => std::cmp::Ordering::Equal, // 保持原有顺序
                SortField::Text => entry_a.text.cmp(&entry_b.text),
                SortField::Code => entry_a.code.cmp(&entry_b.code),
            };

            match sort_order {
                SortOrder::Ascending => cmp,
                SortOrder::Descending => cmp.reverse(),
            }
        });
    }

    /// 刷新扫描到的词典文件列表
    pub fn refresh_discovered_files(&mut self) {
        self.discovered_files = self.rime_loader.scan_dict_files();
        self.set_status(format!("扫描到 {} 个词典文件", self.discovered_files.len()));
    }

    /// 设置状态消息（5 秒后自动清空）
    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
        self.status_timer = 5.0;
    }

    /// 每帧更新：递减状态消息计时器
    pub fn tick(&mut self, dt: f32) {
        if self.status_timer > 0.0 {
            self.status_timer -= dt;
            if self.status_timer <= 0.0 {
                self.status_message = None;
                self.status_timer = 0.0;
            }
        }
    }

    /// 清除状态消息
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
        self.status_timer = 0.0;
    }
}
