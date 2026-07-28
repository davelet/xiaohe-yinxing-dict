use crate::config::AppConfig;
use crate::dict::ExternalDictEntry;
use crate::external_dict_cache::{load_external_entries, save_external_cache};
use crate::rime_loader::{self, DictFileInfo, RimeLoader};
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
    /// 是否显示手动添加文件路径输入
    pub show_manual_add: bool,
    /// 是否显示添加新词对话框
    pub show_add_word_dialog: bool,
    /// 添加新词对话框是否需要在打开时自动聚焦（仅首帧）
    pub add_word_dialog_auto_focus: bool,
    /// 添加新词 - 文字输入
    pub new_word_text: String,
    /// 添加新词 - 编码输入
    pub new_word_code: String,
    /// 添加新词反馈消息 (消息, 是否成功)
    pub add_word_feedback: Option<(String, bool)>,
    /// 反馈消息计时器
    pub add_word_timer: f32,
}

impl Default for ManagerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ManagerState {
    /// 创建新的输入法数据视图状态
    pub fn new() -> Self {
        let mut config = AppConfig::load();
        let rime_loader = RimeLoader::new(&config.rime_user_dir);
        let discovered_files = rime_loader.scan_dict_files();

        // 尝试从缓存加载，失败则从文件加载
        let (external_entries, file_mtimes, load_errors, entry_counts) =
            load_external_entries(&config, &rime_loader);

        // 更新 entry_count
        for file in config.external_dict_files.iter_mut() {
            file.entry_count = entry_counts.get(&file.path).copied();
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
            show_manual_add: false,
            show_add_word_dialog: false,
            add_word_dialog_auto_focus: false,
            status_message: None,
            status_timer: 0.0,
            external_copied_feedback: None,
            external_feedback_timer: 0.0,
            load_errors,
            file_mtimes,
            new_word_text: String::new(),
            new_word_code: String::new(),
            add_word_feedback: None,
            add_word_timer: 0.0,
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

    /// 移除所有手动添加的词典（不在默认扫描目录下的）
    pub fn remove_manual_dicts(&mut self) {
        let rime_user_dir = self.config.rime_user_dir.clone();
        let count = self
            .config
            .external_dict_files
            .iter()
            .filter(|f| {
                let rime_dir = std::path::Path::new(&rime_user_dir);
                !std::path::Path::new(&f.path).starts_with(rime_dir)
            })
            .count();
        if count == 0 {
            self.set_status("没有手动添加的词典");
            return;
        }
        self.config.remove_manual_dicts(&rime_user_dir);
        if let Err(e) = self.config.save() {
            self.set_status(format!("保存配置失败: {}", e));
            return;
        }
        self.reload_external_dicts();
        self.set_status(format!("已移除 {} 个手动添加的词典", count));
    }

    /// 重新加载所有外部词典
    pub fn reload_external_dicts(&mut self) {
        self.external_entries.clear();
        self.file_mtimes.clear();
        self.load_errors.clear();

        // 加载所有文件（包含禁用的），缓存需要全部数据
        let mut all_entries: Vec<ExternalDictEntry> = Vec::new();
        let mut entry_counts: HashMap<String, usize> = HashMap::new();
        for dict_file in self.config.external_dict_files.iter() {
            if let Ok(entries) = self.rime_loader.load_dict_file(&dict_file.path) {
                entry_counts.insert(dict_file.path.clone(), entries.len());
                if dict_file.is_enabled {
                    self.external_entries.extend(entries.iter().cloned());
                }
                all_entries.extend(entries);
                if let Ok(metadata) = std::fs::metadata(&dict_file.path)
                    && let Ok(mtime) = metadata.modified()
                {
                    self.file_mtimes.insert(dict_file.path.clone(), mtime);
                }
            } else {
                self.load_errors.push(dict_file.path.clone());
            }
        }
        for file in self.config.external_dict_files.iter_mut() {
            file.entry_count = entry_counts.get(&file.path).copied();
        }

        self.rebuild_external_engine();
        save_external_cache(&all_entries, &self.file_mtimes, &entry_counts);
    }

    /// 检查文件是否有变更，如果有则重新加载
    pub fn check_and_reload_changed_files(&mut self) -> bool {
        let mtime_seconds = |path: &str| -> Option<u64> {
            let meta = std::fs::metadata(path).ok()?;
            let mtime = meta.modified().ok()?;
            Some(mtime.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs())
        };
        let stored_seconds = |mtime: &SystemTime| -> u64 {
            mtime
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        };

        let mut changed = false;
        let enabled_paths: Vec<String> = self
            .config
            .enabled_external_dicts()
            .iter()
            .map(|f| f.path.clone())
            .collect();

        for path in &enabled_paths {
            let Some(current) = mtime_seconds(path) else {
                continue;
            };
            let stored = self.file_mtimes.get(path).map(stored_seconds);
            if stored != Some(current) {
                changed = true;
                break;
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

    /// 添加新词到小鹤音形自定义词典
    pub fn add_new_word(&mut self) {
        let text = self.new_word_text.trim().to_string();
        let code = self.new_word_code.trim().to_string();

        if text.is_empty() || code.is_empty() {
            self.add_word_feedback = Some(("文字和编码不能为空".to_string(), false));
            self.add_word_timer = 10.0;
            return;
        }

        if !code.chars().all(|c| c.is_ascii_lowercase()) {
            self.add_word_feedback = Some(("编码只能包含小写字母 a-z".to_string(), false));
            self.add_word_timer = 10.0;
            return;
        }

        // 检查是否已存在相同文字+编码的条目（仅检查 flypy_custom 来源）
        let already_exists = self
            .external_entries
            .iter()
            .any(|e| e.text == text && e.code == code && e.source == "flypy_custom");
        if already_exists {
            self.add_word_feedback = Some((
                format!("词 \"{}\" ({}) 已存在于自定义词典中", text, code),
                false,
            ));
            self.add_word_timer = 10.0;
            return;
        }

        let type_label = if text.chars().count() == 1 {
            "单字"
        } else {
            "词组"
        };

        // 步骤 1：写 flypy_custom.dict.yaml
        let dict_path = match rime_loader::append_entry_to_custom_dict(
            &self.config.rime_user_dir,
            &text,
            &code,
        ) {
            Ok(p) => p,
            Err(e) => {
                let friendly_msg = if e.contains("创建") {
                    "创建自定义词典文件失败，请检查目录权限"
                } else if e.contains("打开") || e.contains("写入") {
                    "写入词典文件失败，请检查文件权限"
                } else {
                    &e
                };
                self.add_word_feedback = Some((friendly_msg.to_string(), false));
                self.add_word_timer = 10.0;
                return;
            }
        };

        // 步骤 2：让自定义词典生效
        // 本软件专用于小鹤音形，固定把 flypy_custom 合并进 flypy 主词典。
        // 使用 import_tables（Rime 官方推荐的词典合并方式，跨平台通用）。
        // 注意：旧版在 macOS 上用 .custom.yaml 把 translator/dictionary
        // 写成列表形式，会导致 table_translator 编译失败（小鹤方案无法输入）。
        let patch_result = match self.flypy_main_dict_path() {
            Some(dict_path) => {
                // 清理旧版写入的损坏 .custom.yaml（translator/dictionary 列表）
                self.cleanup_legacy_custom_yaml_patch();
                rime_loader::ensure_import_tables_in_dict(&dict_path).map(|_| dict_path)
            }
            None => {
                Err("未找到小鹤主词典（flypy.dict.yaml），请检查 rime_user_dir 设置".to_string())
            }
        };
        let patch_msg = match &patch_result {
            Ok(_) => None,
            Err(e) => Some(format!("已写文件但 patch schema 失败: {}", e)),
        };

        // 步骤 3：触发 rime 重新部署
        let deploy_result = rime_loader::trigger_rime_deploy();
        let deploy_msg = match &deploy_result {
            Ok(_) => None,
            Err(e) => Some(format!("已写文件但 rime 重新部署失败: {}", e)),
        };

        // 步骤 4：加进 software 内部配置
        let already_added = self
            .config
            .external_dict_files
            .iter()
            .any(|f| f.path == dict_path);
        if !already_added {
            self.add_external_dict(dict_path.clone(), "flypy_custom".to_string());
        } else {
            self.reload_external_dicts();
        }

        // 反馈信息
        let main_msg = format!("已添加{}「{}」(编码: {})", type_label, text, code);
        let final_msg = match (patch_msg, deploy_msg) {
            (None, None) => format!("{}，已自动部署", main_msg),
            (None, Some(d)) => format!("{}，{}", main_msg, d),
            (Some(p), None) => format!("{}，{}", main_msg, p),
            (Some(p), Some(d)) => format!("{}，{}；{}", main_msg, p, d),
        };
        let success = patch_result.is_ok() && deploy_result.is_ok();
        if success {
            self.set_status(final_msg);
            self.new_word_text.clear();
            self.new_word_code.clear();
        } else {
            self.add_word_feedback = Some((final_msg, false));
            self.add_word_timer = 10.0;
        }
    }

    /// 清理旧版 macOS 写入的 .custom.yaml（translator/dictionary 列表会导致
    /// table_translator 编译失败，使小鹤方案无法输入）。新版改用主词典头部的
    /// import_tables 合并 flypy_custom，因此这里移除旧的损坏 patch。
    /// 本软件专用于小鹤音形，固定清理 flypy 的 .custom.yaml。
    fn cleanup_legacy_custom_yaml_patch(&mut self) {
        const FLYPY_SCHEMA: &str = "flypy";
        let schema_path = std::path::Path::new(&self.config.rime_user_dir)
            .join(format!("{}.schema.yaml", FLYPY_SCHEMA))
            .to_string_lossy()
            .to_string();
        if !std::path::Path::new(&schema_path).exists() {
            return;
        }
        let _ = rime_loader::cleanup_translator_dictionary_patch(&schema_path);
    }

    /// 解析小鹤（flypy）主词典文件路径（如 flypy.dict.yaml）。
    /// 本软件专用于小鹤音形，固定使用 flypy schema：
    /// 读取 flypy.schema.yaml 的 translator/dictionary 得到主词典名，
    /// 拼出 <dict>.dict.yaml。找不到时返回 None。
    fn flypy_main_dict_path(&self) -> Option<String> {
        const FLYPY_SCHEMA: &str = "flypy";
        let rime_dir = std::path::Path::new(&self.config.rime_user_dir);
        let schema_path = rime_dir.join(format!("{}.schema.yaml", FLYPY_SCHEMA));
        if !schema_path.exists() {
            return None;
        }
        let dict_name = rime_loader::read_schema_dictionary(&schema_path.to_string_lossy())
            .unwrap_or_else(|| "flypy".to_string());
        let dict_path = rime_dir.join(format!("{}.dict.yaml", dict_name));
        Some(dict_path.to_string_lossy().to_string())
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
        if self.add_word_timer > 0.0 {
            self.add_word_timer -= dt;
            if self.add_word_timer <= 0.0 {
                self.add_word_feedback = None;
                self.add_word_timer = 0.0;
            }
        }
    }

    /// 清除状态消息
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
        self.status_timer = 0.0;
    }
}
