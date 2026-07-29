use std::time::Instant;

use egui_commonmark::CommonMarkCache;
use tokio::runtime::Runtime;
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};

use crate::ai::chat::ChatState;
use crate::ai::client::StreamMessage;
use crate::ai::config::{AiConfig, Provider};
use crate::ai::history::ConversationStore;
use crate::types::ChatTab;

/// 模型编辑表单草稿（不属于持久化配置，独立于 ChatUiState 以便整体重置）
#[derive(Default)]
pub struct ModelEditorDraft {
    pub editing_id: Option<String>,
    pub show_dialog: bool,
    pub name: String,
    pub provider: Provider,
    pub api_url: String,
    pub model: String,
    pub api_key: String,
    pub temperature: f32,
    pub max_tokens: u32,
    pub error_message: String,
}

/// AI 对话相关的全部 UI 状态（从 DictApp 抽取，共 26 个字段）。
pub struct ChatUiState {
    pub show_viewport: bool,
    pub tab: ChatTab,
    pub input: String,
    pub state: ChatState,
    pub tokio_runtime: Option<Runtime>,
    pub test_response: String,
    pub show_privacy_dialog: bool,
    pub md_cache: CommonMarkCache,
    pub rx: Option<UnboundedReceiver<StreamMessage>>,
    pub test_rx: Option<oneshot::Receiver<String>>,
    pub test_in_progress: bool,
    pub save_response: String,
    pub last_send_time: Option<Instant>,
    pub conversation_store: ConversationStore,
    pub model_editor: ModelEditorDraft,
    /// 删除确认弹窗状态 (model_id, model_name)
    pub confirm_delete: Option<(String, String)>,
    /// 清空所有对话确认弹窗状态
    pub confirm_clear_conversations: bool,
    /// AI 配置（加载一次，跨帧保持修改状态）
    pub ai_config: AiConfig,
}

impl Default for ChatUiState {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatUiState {
    pub fn new() -> Self {
        Self {
            show_viewport: false,
            tab: ChatTab::Conversation,
            input: String::new(),
            state: ChatState::default(),
            tokio_runtime: None,
            test_response: String::new(),
            show_privacy_dialog: false,
            md_cache: CommonMarkCache::default(),
            rx: None,
            test_rx: None,
            test_in_progress: false,
            save_response: String::new(),
            last_send_time: None,
            conversation_store: ConversationStore::new(),
            model_editor: ModelEditorDraft::default(),
            confirm_delete: None,
            confirm_clear_conversations: false,
            ai_config: AiConfig::load(),
        }
    }
}

/// chat_viewport 产生的跨模块副作用（写回 DictApp 主搜索状态）。
/// 由 ui/mod.rs 消费并执行跳转/关闭动作。
#[derive(Debug, Clone)]
pub enum ChatAction {
    /// 关闭 AI 子窗口，回到词典视图
    Close,
    /// 关闭 AI 子窗口，跳转到词典并填入搜索词
    JumpToDict(String),
}
