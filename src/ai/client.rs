use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use rig_core::agent::MultiTurnStreamItem;
use rig_core::client::CompletionClient;
use rig_core::completion::message::AssistantContent;
use rig_core::completion::{Message, Prompt};
use rig_core::providers::openai;
use rig_core::streaming::{StreamedAssistantContent, StreamingChat};
use tokio::sync::mpsc;

use crate::ai::config::{AiConfig, ModelConfig};
use crate::ai::tools::{ExternalDictData, create_tools};
use crate::dict::DictEntry;
use crate::search::SearchEngine;

pub const DEFAULT_SYSTEM_PROMPT: &str = r#"你是小鹤音形输入法的专业助手。请遵守以下规则：

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

pub type ChatAgent = rig_core::agent::Agent<openai::completion::CompletionModel>;

/// 创建客户端，返回 OpenAI Chat Completions 兼容客户端
/// （国内厂商 DeepSeek/智谱/通义/月之暗面/火山引擎 等都只兼容 Chat Completions 协议，
/// 不支持 OpenAI 新的 Responses API，因此用 .completions_api() 显式切换）
pub fn create_client(
    model_config: &ModelConfig,
    api_key: &str,
) -> Result<openai::CompletionsClient, String> {
    openai::Client::builder()
        .api_key(api_key)
        .base_url(&model_config.api_url)
        .build()
        .map(|c| c.completions_api())
        .map_err(|e| format!("创建客户端失败: {e}"))
}

/// 创建 agent
pub fn build_agent(
    client: openai::CompletionsClient,
    model_config: &ModelConfig,
    ai_config: &AiConfig,
    engine: Arc<SearchEngine<DictEntry>>,
    help_manager: Arc<crate::help::HelpManager>,
    external_dict_data: Option<ExternalDictData>,
) -> ChatAgent {
    // 总是启用工具调用
    let tools = create_tools(engine, help_manager, external_dict_data);

    client
        .agent(&model_config.model)
        .preamble(DEFAULT_SYSTEM_PROMPT)
        .tools(tools)
        .temperature(model_config.temperature as f64)
        .max_tokens(model_config.max_tokens as u64)
        .default_max_turns(ai_config.max_tool_turns as usize)
        .build()
}

/// 流式消息接收
pub enum StreamMessage {
    /// 文本增量（逐字流式）
    TextDelta(String),
    /// 工具调用事件（工具名 + JSON 参数 + 可选的执行结果文本）
    ToolCall {
        tool_name: String,
        arguments: String,
        result: String,
    },
    /// 流式完成
    Complete,
    /// 出错
    Error(String),
}

/// 请求超时时间
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// 发送流式请求（带多轮上下文），通过 channel 逐字返回响应
pub async fn send_message_stream(
    agent: &ChatAgent,
    message: &str,
    chat_history: Vec<Message>,
    tx: mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), String> {
    let mut stream = agent.stream_chat(message, chat_history).await;
    // 暂存流式中途到达的 tool call（id -> (name, args)）
    let mut pending_calls: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let stream_future = async {
        while let Some(item_result) = stream.next().await {
            match item_result {
                Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(
                    text,
                ))) => {
                    if !text.text.is_empty() {
                        let _ = tx.send(StreamMessage::TextDelta(text.text));
                    }
                }
                Ok(MultiTurnStreamItem::StreamAssistantItem(
                    StreamedAssistantContent::ToolCall { tool_call, .. },
                )) => {
                    // 先缓存，等 FinalResponse 里拿到 result 再一次性发
                    pending_calls.insert(
                        tool_call.id.clone(),
                        (
                            tool_call.function.name.clone(),
                            tool_call.function.arguments.to_string(),
                        ),
                    );
                }
                Ok(MultiTurnStreamItem::FinalResponse(final_resp)) => {
                    // 从 history 里收集 tool call id -> result 文本的映射
                    let mut result_map: std::collections::HashMap<String, String> =
                        std::collections::HashMap::new();
                    if let Some(history) = final_resp.history() {
                        for msg in history {
                            if let Message::User { content } = msg {
                                for uc in content.iter() {
                                    if let rig_core::completion::message::UserContent::ToolResult(
                                        tr,
                                    ) = uc
                                    {
                                        use rig_core::completion::message::ToolResultContent;
                                        let text = tr
                                            .content
                                            .iter()
                                            .filter_map(|c| match c {
                                                ToolResultContent::Text(t) => Some(t.text.as_str()),
                                                _ => None,
                                            })
                                            .collect::<Vec<_>>()
                                            .join("\n");
                                        result_map.insert(tr.id.clone(), text);
                                    }
                                }
                            }
                        }
                    }

                    let emit = |tx: &mpsc::UnboundedSender<StreamMessage>,
                                seen: &mut std::collections::HashSet<String>,
                                id: String,
                                name: String,
                                args: String,
                                result: String| {
                        if seen.insert(id.clone()) {
                            let _ = tx.send(StreamMessage::ToolCall {
                                tool_name: name,
                                arguments: args,
                                result,
                            });
                        }
                    };

                    // 发送流式中缓存的 tool calls（带 result）
                    for (id, (name, args)) in pending_calls.drain() {
                        let result = result_map.get(&id).cloned().unwrap_or_default();
                        emit(&tx, &mut seen, id, name, args, result);
                    }

                    // FinalResponse.content 里可能还有未流式发出的 ToolCall
                    for item in final_resp.assistant_content().iter() {
                        if let AssistantContent::ToolCall(tc) = item {
                            let id = tc.id.clone();
                            let result = result_map.get(&id).cloned().unwrap_or_default();
                            emit(
                                &tx,
                                &mut seen,
                                id,
                                tc.function.name.clone(),
                                tc.function.arguments.to_string(),
                                result,
                            );
                        }
                    }
                }
                Ok(_) => {
                    // ToolCallDelta、Reasoning、CompletionCall 等中间事件不影响 UI
                }
                Err(e) => {
                    let _ = tx.send(StreamMessage::Error(format!("流式错误: {e}")));
                    return;
                }
            }
        }
        let _ = tx.send(StreamMessage::Complete);
    };

    match tokio::time::timeout(REQUEST_TIMEOUT, stream_future).await {
        Ok(()) => Ok(()),
        Err(_) => {
            let _ = tx.send(StreamMessage::Error("请求超时（120 秒无响应）".to_string()));
            Ok(())
        }
    }
}

/// 发送非流式请求（用于测试连接等场景）
pub async fn send_message(agent: &ChatAgent, message: &str) -> Result<String, String> {
    agent
        .prompt(message)
        .await
        .map(|r| r.to_string())
        .map_err(|e| format!("请求失败: {e}"))
}

/// 创建测试用 agent（不带工具，用于模型编辑弹窗中的连接测试）
pub fn build_test_agent(model_config: &ModelConfig, api_key: &str) -> Result<ChatAgent, String> {
    let client = create_client(model_config, api_key)?;
    Ok(client
        .agent(&model_config.model)
        .preamble(DEFAULT_SYSTEM_PROMPT)
        .temperature(model_config.temperature as f64)
        .max_tokens(model_config.max_tokens as u64)
        .build())
}
