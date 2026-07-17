use rig_core::tool::{Tool, ToolDyn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

use crate::dict::Category;
use crate::dict::DictEntry;
use crate::dict::SearchableEntry;
use crate::help::HelpManager;
use crate::search::SearchEngine;
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

        Ok(SearchResponse { total, results })
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

        Ok(SearchResponse { total, results })
    }
}

// ===== get_help 工具 =====

#[derive(Deserialize)]
pub struct GetHelpArgs {
    pub chapter: Option<String>,
}

#[derive(Serialize)]
pub struct HelpChapterInfo {
    pub id: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub search_text_snippet: String,
}

#[derive(Serialize)]
pub struct HelpResponse {
    pub chapters: Vec<HelpChapterInfo>,
    pub total: usize,
}

pub struct GetHelpTool {
    pub help_manager: Arc<HelpManager>,
}

#[derive(Debug, thiserror::Error)]
#[error("帮助查询失败: {0}")]
pub struct HelpError(String);

impl Tool for GetHelpTool {
    const NAME: &'static str = "get_help";
    type Error = HelpError;
    type Args = GetHelpArgs;
    type Output = HelpResponse;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: "get_help".to_string(),
            description: "获取小鹤音形帮助文档内容。不指定章节时返回全部章节列表，指定章节id时返回该章节的内容摘要。可用章节id: readme, xh, up, ux, gz, zg, yy, jm, fh, pc, sj, gj, wv, wt, vy, gy".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chapter": {
                        "type": "string",
                        "description": "章节id，如 readme/xh/up/ux/gz/zg/yy/jm/fh/pc/sj/gj/wv/wt/vy/gy"
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let chapters: Vec<HelpChapterInfo> = if let Some(ref id) = args.chapter {
            // 返回指定章节及其子章节
            let mut result = Vec::new();
            if let Some(ch) = self.help_manager.get_chapter(id) {
                let snippet: String = ch
                    .search_text
                    .iter()
                    .take(10)
                    .copied()
                    .collect::<Vec<_>>()
                    .join("；");
                result.push(HelpChapterInfo {
                    id: ch.id.to_string(),
                    title: ch.title.to_string(),
                    parent_id: ch.parent_id.map(|s| s.to_string()),
                    search_text_snippet: snippet,
                });
            }
            for child in self.help_manager.child_chapters(id.as_str()) {
                let snippet: String = child
                    .search_text
                    .iter()
                    .take(5)
                    .copied()
                    .collect::<Vec<_>>()
                    .join("；");
                result.push(HelpChapterInfo {
                    id: child.id.to_string(),
                    title: child.title.to_string(),
                    parent_id: child.parent_id.map(|s| s.to_string()),
                    search_text_snippet: snippet,
                });
            }
            result
        } else {
            // 返回顶层章节
            self.help_manager
                .top_level_chapters()
                .iter()
                .map(|ch| {
                    let snippet: String = ch
                        .search_text
                        .iter()
                        .take(3)
                        .copied()
                        .collect::<Vec<_>>()
                        .join("；");
                    HelpChapterInfo {
                        id: ch.id.to_string(),
                        title: ch.title.to_string(),
                        parent_id: ch.parent_id.map(|s| s.to_string()),
                        search_text_snippet: snippet,
                    }
                })
                .collect()
        };

        let total = chapters.len();
        Ok(HelpResponse { chapters, total })
    }
}

// ===== list_categories 工具 =====

#[derive(Serialize)]
pub struct CategoryInfo {
    pub name: String,
    pub count: usize,
    pub description: String,
}

#[derive(Serialize)]
pub struct CategoryListResponse {
    pub categories: Vec<CategoryInfo>,
    pub total: usize,
}

pub struct ListCategoriesTool {
    pub engine: Arc<SearchEngine<DictEntry>>,
}

impl Tool for ListCategoriesTool {
    const NAME: &'static str = "list_categories";
    type Error = SearchError;
    type Args = ();
    type Output = CategoryListResponse;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: "list_categories".to_string(),
            description: "列出小鹤音形所有编码分类及其条目数量".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn call(&self, _args: Self::Args) -> Result<Self::Output, Self::Error> {
        let categories = Category::all();
        let entries = self.engine.entries();

        let mut count_map: HashMap<Category, usize> = HashMap::new();
        for entry in entries {
            *count_map.entry(entry.category()).or_default() += 1;
        }

        let mut cat_infos: Vec<CategoryInfo> = categories
            .iter()
            .map(|cat| {
                let count = count_map.get(cat).copied().unwrap_or(0);
                CategoryInfo {
                    name: cat.display_name().to_string(),
                    count,
                    description: cat.description().to_string(),
                }
            })
            .collect();

        // Sort by count descending
        cat_infos.sort_by_key(|c| std::cmp::Reverse(c.count));

        let total = cat_infos.len();
        Ok(CategoryListResponse {
            categories: cat_infos,
            total,
        })
    }
}

// ===== get_category_stats 工具 =====

#[derive(Deserialize)]
pub struct GetCategoryStatsArgs {
    pub category: Option<String>,
}

#[derive(Serialize)]
pub struct CategoryStatsResponse {
    pub categories: Vec<CategoryInfo>,
    pub total: usize,
}

pub struct GetCategoryStatsTool {
    pub engine: Arc<SearchEngine<DictEntry>>,
}

impl Tool for GetCategoryStatsTool {
    const NAME: &'static str = "get_category_stats";
    type Error = SearchError;
    type Args = GetCategoryStatsArgs;
    type Output = CategoryStatsResponse;

    async fn definition(&self, _prompt: String) -> rig_core::completion::ToolDefinition {
        rig_core::completion::ToolDefinition {
            name: "get_category_stats".to_string(),
            description: "获取分类统计信息。可指定分类名过滤，不指定则返回全部分类".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "category": {
                        "type": "string",
                        "description": "分类名，如 一级简码/二重简码/四码全码（字） 等"
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let categories = Category::all();
        let entries = self.engine.entries();

        let mut count_map: HashMap<Category, usize> = HashMap::new();
        for entry in entries {
            *count_map.entry(entry.category()).or_default() += 1;
        }

        let cat_infos: Vec<CategoryInfo> = categories
            .iter()
            .filter(|cat| {
                if let Some(ref filter) = args.category {
                    cat.display_name().contains(filter.as_str())
                } else {
                    true
                }
            })
            .map(|cat| {
                let count = count_map.get(cat).copied().unwrap_or(0);
                CategoryInfo {
                    name: cat.display_name().to_string(),
                    count,
                    description: cat.description().to_string(),
                }
            })
            .collect();

        let total = cat_infos.len();
        Ok(CategoryStatsResponse {
            categories: cat_infos,
            total,
        })
    }
}

// ===== 工具注册 =====

pub fn create_tools(
    engine: Arc<SearchEngine<DictEntry>>,
    help_manager: Arc<HelpManager>,
) -> Vec<Box<dyn ToolDyn>> {
    vec![
        Box::new(SearchTextTool {
            engine: engine.clone(),
        }),
        Box::new(SearchCodeTool {
            engine: engine.clone(),
        }),
        Box::new(GetHelpTool { help_manager }),
        Box::new(ListCategoriesTool {
            engine: engine.clone(),
        }),
        Box::new(GetCategoryStatsTool { engine }),
    ]
}
