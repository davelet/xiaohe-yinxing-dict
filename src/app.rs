use crate::config::AppConfig;
use crate::dict::ExternalDictEntry;
use crate::rime_loader::{DictFileInfo, RimeLoader};
use crate::search;

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
    /// 外部条目搜索结果
    pub external_search_results: Vec<(usize, search::MatchKind)>,
    /// 外部条目搜索结果总数
    pub external_total_results: usize,
    /// 新增文件路径输入
    pub new_file_path: String,
    /// 加载状态消息
    pub status_message: Option<String>,
    /// 复制反馈
    pub external_copied_feedback: Option<(usize, crate::CopyKind)>,
    /// 复制反馈计时
    pub external_feedback_timer: f32,
}

impl ManagerState {
    /// 创建新的输入法数据视图状态
    pub fn new() -> Self {
        let config = AppConfig::load();
        let rime_loader = RimeLoader::new(&config.rime_user_dir);
        let discovered_files = rime_loader.scan_dict_files();

        // 加载已启用的外部词典
        let mut external_entries = Vec::new();
        for dict_file in config.enabled_external_dicts() {
            if let Ok(entries) = rime_loader.load_dict_file(&dict_file.path) {
                external_entries.extend(entries);
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
            external_search_results: Vec::new(),
            external_total_results: 0,
            new_file_path: String::new(),
            status_message: None,
            external_copied_feedback: None,
            external_feedback_timer: 0.0,
        }
    }

    /// 添加外部词典文件
    pub fn add_external_dict(&mut self, path: String, name: String) {
        self.config.add_external_dict(path.clone(), name);
        if let Err(e) = self.config.save() {
            self.status_message = Some(format!("保存配置失败: {}", e));
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
            self.status_message = Some(format!("成功加载 {} 条外部词典条目", count));
        } else {
            self.status_message = Some(format!("加载文件失败: {}", path));
        }
    }

    /// 移除外部词典文件
    pub fn remove_external_dict(&mut self, path: &str) {
        self.config.remove_external_dict(path);
        if let Err(e) = self.config.save() {
            self.status_message = Some(format!("保存配置失败: {}", e));
            return;
        }

        // 重新加载所有外部词典
        self.reload_external_dicts();
        self.status_message = Some(format!("已移除外部词典: {}", path));
    }

    /// 切换外部词典启用状态
    pub fn toggle_external_dict(&mut self, path: &str) {
        self.config.toggle_external_dict(path);
        if let Err(e) = self.config.save() {
            self.status_message = Some(format!("保存配置失败: {}", e));
            return;
        }

        // 重新加载所有外部词典
        self.reload_external_dicts();
    }

    /// 重新加载所有外部词典
    fn reload_external_dicts(&mut self) {
        self.external_entries.clear();
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
            }
        }

        self.rebuild_external_engine();
    }

    /// 重建外部条目搜索引擎
    fn rebuild_external_engine(&mut self) {
        self.external_engine = search::SearchEngine::build(&self.external_entries);
        self.external_query.clear();
        self.external_search_results.clear();
        self.external_total_results = 0;
    }

    /// 刷新扫描到的词典文件列表
    pub fn refresh_discovered_files(&mut self) {
        self.discovered_files = self.rime_loader.scan_dict_files();
        self.status_message = Some(format!("扫描到 {} 个词典文件", self.discovered_files.len()));
    }

    /// 清除状态消息
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }
}
