use std::sync::Arc;

use futures::StreamExt;
use rig_core::agent::MultiTurnStreamItem;
use rig_core::client::CompletionClient;
use rig_core::completion::Prompt;
use rig_core::providers::openai;
use rig_core::streaming::StreamedAssistantContent;
use rig_core::streaming::StreamingPrompt;
use tokio::sync::mpsc;

use crate::ai::config::AiConfig;
use crate::ai::tools::{create_tools, ExternalDictData};
use crate::dict::DictEntry;
use crate::search::SearchEngine;

const SYSTEM_PROMPT: &str = r#"你是小鹤音形输入法的专业助手。请遵守以下规则：

1. 只回答与小鹤音形输入法相关的问题
2. 使用中文回答
3. 优先调用本地工具查询精确数据，不要凭记忆回答编码问题
4. 编码使用等宽字体格式展示，便于对齐
5. 如果用户问了无关问题，礼貌拒绝并引导回小鹤音形话题
6. 回答要简洁明了，适合快速阅读

可用工具：
- search_text: 根据汉字/词组查询编码
- search_code: 根据编码反查汉字/词组
- get_help: 获取帮助文档内容（可用章节id: readme/xh/up/ux/gz/zg/yy/jm/fh/pc/sj/gj/wv/wt/vy/gy）
- list_categories: 列出所有编码分类及条目数量
- get_category_stats: 获取分类统计信息
"#;

pub type ChatAgent = rig_core::agent::Agent<openai::responses_api::GenericResponsesCompletionModel>;

/// 创建客户端，返回 OpenAI 兼容客户端（所有内置厂商都走 OpenAI 兼容协议）
pub fn create_client(config: &AiConfig, api_key: &str) -> Result<openai::Client, String> {
    openai::Client::builder()
        .api_key(api_key)
        .base_url(&config.api_url)
        .build()
        .map_err(|e| format!("创建客户端失败: {e}"))
}

/// 创建 agent
pub fn build_agent(
    client: openai::Client,
    config: &AiConfig,
    engine: Arc<SearchEngine<DictEntry>>,
    help_manager: Arc<crate::help::HelpManager>,
    external_dict_data: Option<ExternalDictData>,
) -> ChatAgent {
    // 如果禁用了 Function Calling，不传 tools，而是在系统提示词中注入工具描述
    let has_external_dict = external_dict_data.is_some();
    let tools = if config.enable_function_calling {
        create_tools(engine, help_manager, external_dict_data)
    } else {
        vec![]
    };

    let mut preamble = SYSTEM_PROMPT.to_string();
    if !config.enable_function_calling {
        // 注入工具描述作为备选方案
        preamble.push_str("\n\n---\n可用工具（请在需要时按以下格式调用）：\n");
        preamble.push_str("工具调用格式：\n```tool_call\n{\"name\": \"工具名\", \"arguments\": {\"参数名\": \"参数值\"}}\n```\n\n");
        preamble.push_str("工具列表：\n");
        preamble.push_str("- search_text(query): 根据汉字/词组查询编码\n");
        preamble.push_str("- search_code(code): 根据编码反查汉字/词组\n");
        preamble.push_str("- get_help(chapter?): 获取帮助文档（章节可选）\n");
        preamble.push_str("- list_categories(): 列出所有分类\n");
        preamble.push_str("- get_category_stats(category?): 获取分类统计\n");
        if has_external_dict {
            preamble.push_str("- search_external_dict(query): 搜索外部词典\n");
        }
    }

    client
        .agent(&config.model)
        .preamble(&preamble)
        .tools(tools)
        .max_tokens(config.max_tokens as u64)
        .default_max_turns(config.max_tool_turns as usize)
        .build()
}

/// 流式消息接收
pub enum StreamMessage {
    /// 文本增量（逐字流式）
    TextDelta(String),
    /// 流式完成
    Complete,
    /// 出错
    Error(String),
}

/// 发送流式请求，通过 channel 逐字返回响应
pub async fn send_message_stream(
    agent: &ChatAgent,
    message: &str,
    tx: mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), String> {
    let mut stream = agent.stream_prompt(message).await;
    while let Some(item_result) = stream.next().await {
        match item_result {
            Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(text))) => {
                if !text.text.is_empty() {
                    let _ = tx.send(StreamMessage::TextDelta(text.text));
                }
            }
            Ok(_) => {
                // 工具调用、推理、完成调用等事件忽略
            }
            Err(e) => {
                let _ = tx.send(StreamMessage::Error(format!("流式错误: {e}")));
                return Ok(());
            }
        }
    }
    let _ = tx.send(StreamMessage::Complete);
    Ok(())
}

/// 发送非流式请求（用于测试连接等场景）
pub async fn send_message(agent: &ChatAgent, message: &str) -> Result<String, String> {
    agent
        .prompt(message)
        .await
        .map(|r| r.to_string())
        .map_err(|e| format!("请求失败: {e}"))
}
