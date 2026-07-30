use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::ai::chat::ChatMessage;

/// 单个对话的元数据（列表用，不含消息体）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMeta {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// 完整对话（含消息体）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub messages: Vec<ChatMessage>,
}

impl Conversation {
    /// 从消息列表生成标题（首条用户消息前 20 字）
    pub fn title_from_messages(messages: &[ChatMessage]) -> String {
        for msg in messages {
            if msg.role == crate::ai::chat::Role::User {
                let title: String = msg.content.chars().take(20).collect();
                return if title.is_empty() {
                    "新对话".to_string()
                } else {
                    title
                };
            }
        }
        "新对话".to_string()
    }
}

/// 对话存储管理器
pub struct ConversationStore {
    base_dir: PathBuf,
}

impl ConversationStore {
    /// 创建存储管理器，目录为 config_dir/conversations/
    pub fn new() -> Self {
        let base_dir = Self::conversations_dir();
        std::fs::create_dir_all(&base_dir).ok();
        Self { base_dir }
    }

    /// 对话文件存放目录（供打开所在文件夹等用途）
    pub fn dir(&self) -> &PathBuf {
        &self.base_dir
    }

    fn conversations_dir() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("xiaohe-yinxing-dict");
        config_dir.join("conversations")
    }

    /// 列出所有对话元数据（按更新时间倒序）
    pub fn list_conversations(&self) -> Vec<ConversationMeta> {
        let mut metas = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json")
                    && let Ok(content) = std::fs::read_to_string(&path)
                    && let Ok(conv) = serde_json::from_str::<Conversation>(&content)
                {
                    metas.push(ConversationMeta {
                        id: conv.id,
                        title: conv.title,
                        created_at: conv.created_at,
                        updated_at: conv.updated_at,
                    });
                }
            }
        }
        metas.sort_by_key(|m| std::cmp::Reverse(m.updated_at));
        metas
    }

    /// 保存对话到文件
    pub fn save_conversation(&self, conv: &Conversation) -> Result<(), String> {
        let path = self.base_dir.join(format!("{}.json", conv.id));
        let json = serde_json::to_string_pretty(conv).map_err(|e| format!("序列化失败: {e}"))?;
        std::fs::write(&path, json).map_err(|e| format!("写入文件失败: {e}"))
    }

    /// 加载对话
    pub fn load_conversation(&self, id: &str) -> Result<Conversation, String> {
        let path = self.base_dir.join(format!("{id}.json"));
        let content = std::fs::read_to_string(&path).map_err(|e| format!("读取文件失败: {e}"))?;
        serde_json::from_str(&content).map_err(|e| format!("解析失败: {e}"))
    }

    /// 删除对话
    pub fn delete_conversation(&self, id: &str) -> Result<(), String> {
        let path = self.base_dir.join(format!("{id}.json"));
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| format!("删除失败: {e}"))
        } else {
            Ok(())
        }
    }

    /// 清空所有对话
    pub fn clear_all(&self) -> Result<(), String> {
        if let Ok(entries) = std::fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
        Ok(())
    }

    /// 计算对话文件总磁盘占用（字节）
    pub fn total_disk_usage(&self) -> u64 {
        let mut total: u64 = 0;
        if let Ok(entries) = std::fs::read_dir(&self.base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json")
                    && let Ok(meta) = std::fs::metadata(&path)
                {
                    total += meta.len();
                }
            }
        }
        total
    }

    /// 执行保留策略：同时限制对话数量和磁盘占用，超出时删除最旧的对话
    pub fn enforce_limits(&self, max_count: usize, max_disk_mb: u64) {
        let metas = self.list_conversations();
        let max_bytes = max_disk_mb * 1024 * 1024;

        // 先按数量裁剪
        if metas.len() > max_count {
            for meta in metas.iter().skip(max_count) {
                let _ = self.delete_conversation(&meta.id);
            }
        }

        // 再按磁盘空间裁剪（从最旧的开始删除）
        let mut disk_used = self.total_disk_usage();
        if disk_used > max_bytes {
            let mut remaining_count = self.list_conversations().len();
            loop {
                if disk_used <= max_bytes || remaining_count <= 1 {
                    break;
                }
                let remaining = self.list_conversations();
                let Some(oldest) = remaining.last() else {
                    break;
                };
                let path = self.base_dir.join(format!("{}.json", oldest.id));
                let mut deleted = false;
                if let Ok(file_meta) = std::fs::metadata(&path) {
                    let file_size = file_meta.len();
                    if self.delete_conversation(&oldest.id).is_ok() {
                        disk_used = disk_used.saturating_sub(file_size);
                        deleted = true;
                    }
                }
                if !deleted {
                    break;
                }
                remaining_count -= 1;
            }
        }
    }

    /// 导出对话为 Markdown
    pub fn export_markdown(conv: &Conversation) -> String {
        let mut md = format!("# {}\n\n", conv.title);
        for msg in &conv.messages {
            match msg.role {
                crate::ai::chat::Role::User => {
                    md.push_str(&format!("**用户:** {}\n\n", msg.content));
                }
                crate::ai::chat::Role::AI => {
                    md.push_str(&format!("**助手:** {}\n\n", msg.content));
                    if !msg.tool_calls.is_empty() {
                        md.push_str("<details><summary>工具调用详情</summary>\n\n");
                        for tc in &msg.tool_calls {
                            md.push_str(&format!(
                                "- **{}**: `{}` → {}\n",
                                tc.tool_name, tc.arguments, tc.result
                            ));
                        }
                        md.push_str("</details>\n\n");
                    }
                }
                _ => {}
            }
        }
        md
    }
}

impl Default for ConversationStore {
    fn default() -> Self {
        Self::new()
    }
}

/// 生成对话 ID（纳秒时间戳，避免快速创建时冲突）
pub fn generate_conversation_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{ts}")
}
