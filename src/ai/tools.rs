use rig_core::tool::{Tool, ToolDyn};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::search::SearchEngine;
use crate::dict::DictEntry;
use crate::dict::SearchableEntry;
use std::sync::Arc;

// ===== search_text 工具 =====

#[derive(Deserialize)]
pub struct SearchTextArgs {
    pub query: String,
}

#[derive(Debug, thiserror::Error)]
#[error("搜索失败: {0}")]
pub struct SearchError(String);

#[derive(Serialize)]
pub struct SearchResult {
    pub text: String,
    pub code: String,
    pub category: String,
    pub is_secondary: bool,
}

#[derive(Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
}

pub struct SearchTextTool {
    pub engine: Arc<SearchEngine<DictEntry>>,
}

impl Tool for SearchTextTool {
    const NAME: &'static str = "search_text";
    type Error = SearchError;
    type Args = SearchTextArgs;
    type Output = SearchResponse;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: "search_text".to_string(),
            description: "根据汉字或词组查询小鹤音形编码".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "要查询的汉字或词组"
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let (indices, total) = self.engine.search(&args.query, None);

        let results: Vec<SearchResult> = indices
            .into_iter()
            .map(|(idx, _)| {
                let entry = &self.engine.entries()[idx];
                SearchResult {
                    text: entry.text().to_string(),
                    code: entry.code().to_string(),
                    category: entry.category().display_name().to_string(),
                    is_secondary: entry.is_secondary(),
                }
            })
            .collect();

        Ok(SearchResponse {
            total,
            results,
        })
    }
}

// ===== search_code 工具 =====

#[derive(Deserialize)]
pub struct SearchCodeArgs {
    pub code: String,
}

pub struct SearchCodeTool {
    pub engine: Arc<SearchEngine<DictEntry>>,
}

impl Tool for SearchCodeTool {
    const NAME: &'static str = "search_code";
    type Error = SearchError;
    type Args = SearchCodeArgs;
    type Output = SearchResponse;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: "search_code".to_string(),
            description: "根据编码反查小鹤音形的汉字或词组".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "要反查的编码"
                    }
                },
                "required": ["code"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let (indices, total) = self.engine.search(&args.code, None);

        let results: Vec<SearchResult> = indices
            .into_iter()
            .map(|(idx, _)| {
                let entry = &self.engine.entries()[idx];
                SearchResult {
                    text: entry.text().to_string(),
                    code: entry.code().to_string(),
                    category: entry.category().display_name().to_string(),
                    is_secondary: entry.is_secondary(),
                }
            })
            .collect();

        Ok(SearchResponse {
            total,
            results,
        })
    }
}

// ===== 工具注册 =====

pub fn create_tools(engine: Arc<SearchEngine<DictEntry>>) -> Vec<Box<dyn ToolDyn>> {
    vec![
        Box::new(SearchTextTool { engine: engine.clone() }),
        Box::new(SearchCodeTool { engine }),
    ]
}
