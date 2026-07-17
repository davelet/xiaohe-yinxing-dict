use serde::{Deserialize, Serialize};
use tokio::task::AbortHandle;

/// 消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Role {
    User,
    AI,
    System,
    Tool,
}

/// 工具调用记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub arguments: String,
    pub result: String,
}

/// 聊天消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    pub timestamp: u64,
    /// 该消息触发的工具调用记录（仅 AI 消息）
    #[serde(default)]
    pub tool_calls: Vec<ToolCallRecord>,
    /// 是否被中断
    #[serde(default)]
    pub is_interrupted: bool,
}

/// 流式生成状态
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StreamState {
    /// 空闲
    #[default]
    Idle,
    /// 正在生成（包含当前已接收的内容）
    Generating(String),
    /// 流式完成
    Complete(String),
    /// 被中断
    Interrupted(String),
    /// 出错
    Error(String),
}

/// 对话状态
pub struct ChatState {
    /// 消息列表
    pub messages: Vec<ChatMessage>,
    /// 是否正在生成
    pub is_generating: bool,
    /// 流式生成状态
    pub stream_state: StreamState,
    /// 当前生成任务的 AbortHandle（用于中断）
    pub abort_handle: Option<AbortHandle>,
    /// 屏幕宽度不足提示
    pub show_screen_width_warning: bool,
    /// 当前对话 ID（None 表示未关联持久化对话）
    pub current_conversation_id: Option<String>,
}

impl Default for ChatState {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            is_generating: false,
            stream_state: StreamState::Idle,
            abort_handle: None,
            show_screen_width_warning: false,
            current_conversation_id: None,
        }
    }
}

impl ChatState {
    /// 添加用户消息
    pub fn add_user_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: Role::User,
            content,
            timestamp: now_unix(),
            tool_calls: Vec::new(),
            is_interrupted: false,
        });
    }

    /// 添加 AI 消息
    pub fn add_ai_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: Role::AI,
            content,
            timestamp: now_unix(),
            tool_calls: Vec::new(),
            is_interrupted: false,
        });
    }

    /// 添加被中断的 AI 消息
    pub fn add_interrupted_ai_message(&mut self, content: String) {
        self.messages.push(ChatMessage {
            role: Role::AI,
            content,
            timestamp: now_unix(),
            tool_calls: Vec::new(),
            is_interrupted: true,
        });
    }

    /// 开始生成
    pub fn start_generation(&mut self) {
        self.is_generating = true;
        self.stream_state = StreamState::Generating(String::new());
    }

    /// 追加流式内容（逐字累积）
    pub fn append_stream(&mut self, delta: String) {
        if let StreamState::Generating(ref mut content) = self.stream_state {
            content.push_str(&delta);
        }
    }

    /// 完成生成
    pub fn finish_generation(&mut self) {
        self.is_generating = false;
        if let StreamState::Generating(content) = std::mem::take(&mut self.stream_state) {
            self.add_ai_message(content);
        }
        self.abort_handle = None;
    }

    /// 中断生成
    pub fn interrupt_generation(&mut self) {
        if let Some(handle) = self.abort_handle.take() {
            handle.abort();
        }
        self.is_generating = false;
        if let StreamState::Generating(content) = std::mem::take(&mut self.stream_state)
            && !content.is_empty()
        {
            self.add_interrupted_ai_message(content);
        }
    }

    /// 清空对话
    pub fn clear(&mut self) {
        self.messages.clear();
        self.is_generating = false;
        self.stream_state = StreamState::Idle;
        self.abort_handle = None;
    }

    /// 按轮数裁剪历史消息，保留最近 N 轮对话
    /// 一轮 = 1 条用户消息 + 1 条 AI 消息
    pub fn trim_history(&mut self, max_rounds: usize) {
        if max_rounds == 0 || self.messages.is_empty() {
            return;
        }

        // 找到最近 N 轮的用户消息位置
        let mut user_count = 0;
        let mut split_index = 0;

        for (i, msg) in self.messages.iter().enumerate() {
            if msg.role == Role::User {
                user_count += 1;
                if user_count > max_rounds {
                    split_index = i;
                    break;
                }
            }
        }

        // 如果找到了分割点，删除之前的消息
        if split_index > 0 {
            self.messages.drain(..split_index);
        }
    }

    /// 获取估算的 token 数量（粗略估计：1 个汉字约 2 个 token，1 个英文单词约 1.5 个 token）
    pub fn estimate_tokens(&self) -> usize {
        let mut total = 0;
        for msg in &self.messages {
            // 中文字符
            let chinese_chars = msg
                .content
                .chars()
                .filter(|c| (*c as u32) >= 0x4e00)
                .count();
            // 英文和其他字符
            let other_chars = msg.content.chars().filter(|c| (*c as u32) < 0x4e00).count();
            total += chinese_chars * 2 + (other_chars as f64 * 1.5) as usize;

            // 工具调用
            for tc in &msg.tool_calls {
                let tc_chinese = tc
                    .arguments
                    .chars()
                    .filter(|c| (*c as u32) >= 0x4e00)
                    .count();
                let tc_other = tc
                    .arguments
                    .chars()
                    .filter(|c| (*c as u32) < 0x4e00)
                    .count();
                total += tc_chinese * 2 + (tc_other as f64 * 1.5) as usize;

                let res_chinese = tc.result.chars().filter(|c| (*c as u32) >= 0x4e00).count();
                let res_other = tc.result.chars().filter(|c| (*c as u32) < 0x4e00).count();
                total += res_chinese * 2 + (res_other as f64 * 1.5) as usize;
            }
        }
        total
    }

    /// 按 token 预算裁剪历史（工具结果超长时截断）
    /// 保留至少最近 2 轮对话，避免裁剪过度
    pub fn trim_by_token_budget(&mut self, budget: usize) {
        let mut current_tokens = self.estimate_tokens();
        const MIN_ROUNDS: usize = 2;

        while current_tokens > budget {
            let user_count = self
                .messages
                .iter()
                .filter(|m| m.role == Role::User)
                .count();
            if user_count <= MIN_ROUNDS {
                break;
            }
            if let Some(oldest) = self.messages.first() {
                let msg_tokens = {
                    let chinese_chars = oldest
                        .content
                        .chars()
                        .filter(|c| (*c as u32) >= 0x4e00)
                        .count();
                    let other_chars = oldest
                        .content
                        .chars()
                        .filter(|c| (*c as u32) < 0x4e00)
                        .count();
                    chinese_chars * 2 + (other_chars as f64 * 1.5) as usize
                };
                current_tokens = current_tokens.saturating_sub(msg_tokens);
                self.messages.remove(0);
            } else {
                break;
            }
        }
    }
}

/// 获取当前 Unix 时间戳（秒）
fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
