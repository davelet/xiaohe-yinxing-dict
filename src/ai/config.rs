use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 提供商类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Provider {
    OpenAI,
    DeepSeek,
    Ollama,
    Anthropic,
    Gemini,
    Custom,
}

impl Default for Provider {
    fn default() -> Self {
        Self::OpenAI
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenAI => write!(f, "OpenAI"),
            Self::DeepSeek => write!(f, "DeepSeek"),
            Self::Ollama => write!(f, "Ollama"),
            Self::Anthropic => write!(f, "Anthropic"),
            Self::Gemini => write!(f, "Gemini"),
            Self::Custom => write!(f, "自定义"),
        }
    }
}

/// AI 配置（持久化至 ai_config.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// OS 钥匙串服务名
    pub keyring_service: String,
    /// OS 钥匙串用户名
    pub keyring_user: String,
    /// API 端点（仅 Custom 提供商生效）
    pub api_url: String,
    /// 模型名称
    pub model: String,
    /// 提供商类型
    pub provider: Provider,
    /// 生成温度
    pub temperature: f32,
    /// 最大输出 token 数
    pub max_tokens: u32,
    /// 是否启用外部词典工具
    pub enable_external_dict_tool: bool,
    /// 是否已确认隐私提示
    pub privacy_acknowledged: bool,
    /// 上下文保留轮数
    pub history_rounds: u32,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            keyring_service: "xiaohe-yinxing-dict".to_string(),
            keyring_user: "default".to_string(),
            api_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            provider: Provider::OpenAI,
            temperature: 0.7,
            max_tokens: 2048,
            enable_external_dict_tool: false,
            privacy_acknowledged: false,
            history_rounds: 10,
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

    /// 加载配置，文件不存在时返回默认配置
    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
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

    /// 从 OS 钥匙串读取 API Key
    pub fn get_api_key(&self) -> Result<String, String> {
        let entry = keyring::Entry::new(&self.keyring_service, &self.keyring_user)
            .map_err(|e| format!("访问钥匙串失败: {e}"))?;
        entry
            .get_password()
            .map_err(|e| format!("读取 API Key 失败: {e}"))
    }

    /// 将 API Key 存入 OS 钥匙串
    pub fn set_api_key(&self, key: &str) -> Result<(), String> {
        let entry = keyring::Entry::new(&self.keyring_service, &self.keyring_user)
            .map_err(|e| format!("访问钥匙串失败: {e}"))?;
        entry
            .set_password(key)
            .map_err(|e| format!("存储 API Key 失败: {e}"))
    }

    /// 从 OS 钥匙串删除 API Key
    pub fn delete_api_key(&self) -> Result<(), String> {
        let entry = keyring::Entry::new(&self.keyring_service, &self.keyring_user)
            .map_err(|e| format!("访问钥匙串失败: {e}"))?;
        entry
            .delete_credential()
            .map_err(|e| format!("删除 API Key 失败: {e}"))
    }
}
