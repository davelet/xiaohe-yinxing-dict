use std::sync::Arc;

use rig_core::client::CompletionClient;
use rig_core::completion::Prompt;
use rig_core::providers::openai;
use tokio::sync::mpsc;

use crate::ai::config::AiConfig;
use crate::ai::tools;
use crate::search::SearchEngine;
use crate::dict::DictEntry;

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
"#;

/// 创建 OpenAI 客户端
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
) -> rig_core::agent::Agent<openai::responses_api::GenericResponsesCompletionModel> {
    let tools = tools::create_tools(engine);

    client
        .agent(&config.model)
        .preamble(SYSTEM_PROMPT)
        .tools(tools)
        .max_tokens(config.max_tokens as u64)
        .build()
}

/// 发送非流式请求
pub async fn send_message(
    agent: &rig_core::agent::Agent<openai::responses_api::GenericResponsesCompletionModel>,
    message: &str,
) -> Result<String, String> {
    let response = agent
        .prompt(message)
        .await
        .map_err(|e| format!("请求失败: {e}"))?;

    Ok(response.to_string())
}

/// 流式消息接收
pub enum StreamMessage {
    /// 文本块
    Text(String),
    /// 流结束
    Complete,
    /// 出错
    Error(String),
}

/// 发送流式请求，通过 channel 返回文本块
pub async fn send_message_stream(
    agent: &rig_core::agent::Agent<openai::responses_api::GenericResponsesCompletionModel>,
    message: &str,
    tx: mpsc::UnboundedSender<StreamMessage>,
) -> Result<(), String> {
    // 使用非流式 API 作为后备
    // rig-core 0.39 的流式 API 需要更复杂的设置
    match send_message(agent, message).await {
        Ok(response) => {
            // 模拟流式输出，逐字发送
            let chars: Vec<char> = response.chars().collect();
            let mut current = String::new();
            for (i, &ch) in chars.iter().enumerate() {
                current.push(ch);
                if tx.send(StreamMessage::Text(current.clone())).is_err() {
                    return Ok(());
                }
                // 小延迟模拟流式效果
                if i % 10 == 0 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
            let _ = tx.send(StreamMessage::Complete);
        }
        Err(e) => {
            let _ = tx.send(StreamMessage::Error(e));
        }
    }
    Ok(())
}
