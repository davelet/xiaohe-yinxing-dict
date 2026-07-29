use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

/// 提供商类型（仅内置国内厂商，OpenAI/Anthropic/Gemini 等境外服务不在内置列表）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Provider {
    /// 深度求索 (DeepSeek)
    DeepSeek,
    /// 智谱 (GLM)
    Zhipu,
    /// 月之暗面 (Moonshot)
    Moonshot,
    /// 通义千问 (Qwen)
    Qwen,
    /// 零一万物 (Yi)
    Yi,
    /// 自定义 OpenAI 兼容端点
    #[default]
    Custom,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeepSeek => write!(f, "深度求索 (DeepSeek)"),
            Self::Zhipu => write!(f, "智谱 (GLM)"),
            Self::Moonshot => write!(f, "月之暗面 (Moonshot)"),
            Self::Qwen => write!(f, "通义千问 (Qwen)"),
            Self::Yi => write!(f, "零一万物 (Yi)"),
            Self::Custom => write!(f, "自定义"),
        }
    }
}

/// 单个模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// 唯一 ID（UUID v4）
    pub id: String,
    /// 用户可读名称
    pub name: String,
    /// 提供商类型
    pub provider: Provider,
    /// API 端点（内置厂商为默认值，用户可覆盖）
    pub api_url: String,
    /// 模型名
    pub model: String,
    /// 生成温度
    pub temperature: f32,
    /// 最大输出 token 数
    pub max_tokens: u32,
    /// API Key（明文存储于配置文件，与 opencode / hermes-agent 等工具一致）
    #[serde(default)]
    pub api_key: String,
    /// 创建时间（Unix 秒时间戳）
    #[serde(default)]
    pub created_at: u64,
    /// 最近修改时间（Unix 秒时间戳，与 created_at 相同表示未修改过）
    #[serde(default)]
    pub updated_at: u64,
}

impl ModelConfig {
    /// 创建新模型配置
    pub fn new(
        name: String,
        provider: Provider,
        api_url: String,
        model: String,
        temperature: f32,
        max_tokens: u32,
    ) -> Self {
        let now = now_secs();
        let id = Uuid::new_v4().to_string();
        Self {
            id,
            name,
            provider,
            api_url,
            model,
            temperature,
            max_tokens,
            api_key: String::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// 标记为已修改，更新修改时间戳
    pub fn touch(&mut self) {
        self.updated_at = now_secs();
    }

    /// 读取 API Key（直接返回字段值）
    pub fn get_api_key(&self) -> Result<String, String> {
        if self.api_key.is_empty() {
            Err("API Key 未配置".to_string())
        } else {
            Ok(self.api_key.clone())
        }
    }

    /// 设置 API Key
    pub fn set_api_key(&mut self, key: &str) -> Result<(), String> {
        self.api_key = key.to_string();
        Ok(())
    }

    /// 清除 API Key
    pub fn delete_api_key(&mut self) -> Result<(), String> {
        self.api_key.clear();
        Ok(())
    }

    /// 时间戳展示文本。如果未修改过（updated_at == created_at）只显示创建时间
    pub fn formatted_time_display(&self) -> String {
        let created = fmt_timestamp(self.created_at);
        let updated = fmt_timestamp(self.updated_at);
        if self.updated_at > self.created_at && self.updated_at > 0 {
            format!("创建于 {}  |  修改于 {}", created, updated)
        } else if self.created_at > 0 {
            format!("创建于 {}", created)
        } else {
            String::new()
        }
    }
}

/// 当前 Unix 秒时间戳
fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Unix 秒时间戳 → "YYYY-MM-DD HH:mm:ss"（东八区）
fn fmt_timestamp(secs: u64) -> String {
    if secs == 0 {
        return "未知".to_string();
    }
    // 东八区偏移 8 小时
    let local_secs = secs + 8 * 3600;
    let days = local_secs / 86400;
    let time_secs = local_secs % 86400;

    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let month_days: &[i64] = if is_leap(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 1u32;
    for (i, &md) in month_days.iter().enumerate() {
        if remaining < md {
            m = (i + 1) as u32;
            break;
        }
        remaining -= md;
    }
    if m > 12 {
        m = 12;
    }
    let d = (remaining + 1) as u32;

    let hour = (time_secs / 3600) as u32;
    let min = ((time_secs % 3600) / 60) as u32;
    let sec = (time_secs % 60) as u32;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y, m, d, hour, min, sec
    )
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self::new(
            "默认配置".to_string(),
            Provider::default(),
            String::new(),
            String::new(),
            0.7,
            2048,
        )
    }
}

/// AI 配置（持久化至 ai_config.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// 是否启用外部词典工具
    pub enable_external_dict_tool: bool,
    /// 是否已确认隐私提示
    pub privacy_acknowledged: bool,
    /// 上下文保留轮数
    pub history_rounds: u32,
    /// 最大工具调用轮次（防止 AI 无限循环调用工具）
    pub max_tool_turns: u32,
    /// 用户是否已完成初始配置（保存过设置即设为 true，用于启动时跳过未配置提示）
    #[serde(default)]
    pub configured: bool,
    /// 是否持久化已关闭的对话历史（关闭时不记录、不提供切换，但不主动删除已有记录）
    #[serde(default)]
    pub persist_conversations: bool,

    /// 多模型列表
    #[serde(default)]
    pub models: Vec<ModelConfig>,
    /// 当前激活的模型 ID
    #[serde(default)]
    pub active_model_id: Option<String>,

    // 兼容旧配置保留字段，迁移后不再使用
    #[serde(skip_serializing)]
    pub keyring_user: Option<String>,
    #[serde(skip_serializing)]
    pub api_url: Option<String>,
    #[serde(skip_serializing)]
    pub model: Option<String>,
    #[serde(skip_serializing)]
    pub provider: Option<Provider>,
    #[serde(skip_serializing)]
    pub temperature: Option<f32>,
    #[serde(skip_serializing)]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing)]
    pub keyring_service: Option<String>,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            enable_external_dict_tool: false,
            privacy_acknowledged: false,
            history_rounds: 10,
            max_tool_turns: 10,
            configured: false,
            persist_conversations: false,
            models: Vec::new(),
            active_model_id: None,
            keyring_user: None,
            api_url: None,
            model: None,
            provider: None,
            temperature: None,
            max_tokens: None,
            keyring_service: None,
        }
    }
}

impl AiConfig {
    /// 配置文件路径（与主 config.json 同目录）
    fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("xiaohe-yinxing-dict");
        config_dir.join("ai_config.json")
    }

    /// 获取当前激活的模型配置
    pub fn active_model(&self) -> Option<&ModelConfig> {
        match &self.active_model_id {
            Some(id) => self.models.iter().find(|m| m.id == *id),
            None => self.models.first(),
        }
    }

    /// 获取当前激活的模型配置（可变引用）
    pub fn active_model_mut(&mut self) -> Option<&mut ModelConfig> {
        match &self.active_model_id {
            Some(id) => self.models.iter_mut().find(|m| m.id == *id),
            None => self.models.first_mut(),
        }
    }

    /// 根据 ID 查找模型
    pub fn find_model(&self, id: &str) -> Option<&ModelConfig> {
        self.models.iter().find(|m| m.id == id)
    }

    /// 根据 ID 查找模型（可变引用）
    pub fn find_model_mut(&mut self, id: &str) -> Option<&mut ModelConfig> {
        self.models.iter_mut().find(|m| m.id == id)
    }

    /// 删除模型
    pub fn delete_model(&mut self, id: &str) -> Option<ModelConfig> {
        let pos = self.models.iter().position(|m| m.id == id);
        let deleted = pos.map(|p| self.models.remove(p));
        // 如果删除的是当前激活模型，重新选择激活
        if Some(id) == self.active_model_id.as_deref() {
            if self.models.is_empty() {
                self.active_model_id = None;
                self.configured = false;
            } else {
                self.active_model_id = Some(self.models[0].id.clone());
            }
        }
        deleted
    }

    /// 复制模型
    pub fn duplicate_model(&mut self, source_id: &str) -> Option<ModelConfig> {
        const MAX_NAME_LEN: usize = 50;
        let source = self.find_model(source_id).cloned();
        if let Some(mut source) = source {
            let new_id = Uuid::new_v4().to_string();
            let suffix = " (副本)";
            let new_name = if source.name.len() + suffix.len() <= MAX_NAME_LEN {
                format!("{}{}", source.name, suffix)
            } else {
                source.name.clone()
            };
            let now = now_secs();
            source.id = new_id;
            source.name = new_name;
            source.created_at = now;
            source.updated_at = now;
            self.models.push(source.clone());
            Some(source)
        } else {
            None
        }
    }

    /// 校验并修复 active_model_id，确保指向有效模型
    pub fn validate_and_fix(&mut self) {
        if self.models.is_empty() {
            self.active_model_id = None;
            self.configured = false;
            return;
        }

        if let Some(active_id) = &self.active_model_id {
            if !self.models.iter().any(|m| m.id == *active_id) {
                self.active_model_id = Some(self.models[0].id.clone());
            }
        } else {
            self.active_model_id = Some(self.models[0].id.clone());
        }

        self.configured = self
            .models
            .iter()
            .any(|m| !m.api_url.trim().is_empty() && !m.model.trim().is_empty());
    }

    /// 加载配置，文件不存在时返回默认配置
    pub fn load() -> Self {
        let path = Self::config_path();
        let mut config: Self = match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        };

        // 旧配置迁移（keyring 时代 -> 明文文件）
        if config.models.is_empty()
            && let Some(provider) = config.provider.take()
        {
            let name = provider.to_string();
            let api_url = config.api_url.take().unwrap_or_else(|| match provider {
                Provider::DeepSeek => "https://api.deepseek.com".to_string(),
                Provider::Zhipu => "https://open.bigmodel.cn".to_string(),
                Provider::Moonshot => "https://api.moonshot.cn".to_string(),
                Provider::Qwen => "https://dashscope.aliyuncs.com".to_string(),
                Provider::Yi => "https://api.lingyiwanwu.com".to_string(),
                Provider::Custom => String::new(),
            });
            let model = config.model.take().unwrap_or_default();
            let temperature = config.temperature.take().unwrap_or(0.7);
            let max_tokens = config.max_tokens.take().unwrap_or(2048);

            let mut migrated =
                ModelConfig::new(name, provider, api_url, model, temperature, max_tokens);
            // 固定 id 为 migrated（与历史逻辑一致）
            migrated.id = "migrated".to_string();
            // 旧 key 存在钥匙串里无法读取（keyring v3 在 macOS 15 有 bug），api_key 留空让用户重填

            config.models.push(migrated);
            config.active_model_id = Some("migrated".to_string());
            config.configured = true;
        }

        // 校验修复
        config.validate_and_fix();

        config
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("序列化失败: {e}"))?;
        std::fs::write(&path, json).map_err(|e| format!("写入文件失败: {e}"))
    }
}
