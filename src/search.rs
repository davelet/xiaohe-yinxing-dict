use crate::dict::{Category, DictEntry};
use crate::trie::Trie;
use std::collections::HashSet;

/// 匹配类型及匹配区域（用于高亮）
#[derive(Debug, Clone)]
pub enum MatchKind {
    /// 文本匹配，带匹配字符范围
    Text(std::ops::Range<usize>),
    /// 编码匹配，带匹配字符范围
    Code(std::ops::Range<usize>),
}

/// 搜索引擎：支持正查（文字→编码）和反查（编码→文字）
#[derive(Debug)]
pub struct SearchEngine {
    entries: Vec<DictEntry>,
    trie: Trie,
}

impl SearchEngine {
    /// 从静态数据构建搜索引擎
    pub fn build(entries: &[DictEntry]) -> Self {
        let mut engine = SearchEngine {
            entries: entries.to_vec(),
            trie: Trie::new(),
        };

        // 构建 Trie：将编码索引插入前缀树
        for (idx, entry) in entries.iter().enumerate() {
            engine.trie.insert(&entry.code.to_lowercase(), idx);
        }

        engine
    }

    /// 执行搜索，返回 (条目索引, 匹配类型) 列表
    pub fn search(
        &self,
        query: &str,
        category_filter: Option<Category>,
    ) -> Vec<(usize, MatchKind)> {
        if query.is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();

        // 判断是否纯字母（编码查询）
        let is_code_query = query_lower
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == ';');

        let candidates: Vec<(usize, MatchKind)> = if is_code_query {
            self.search_by_code(&query_lower)
        } else {
            self.search_by_text(query, &query_lower)
        };

        // 按分类过滤
        let results: Vec<(usize, MatchKind)> = if let Some(cat) = category_filter {
            candidates
                .into_iter()
                .filter(|(idx, _)| self.entries[*idx].category == cat)
                .collect()
        } else {
            candidates
        };

        // 排序：精确 > 前缀 > 子串
        self.sort_results(results, query_lower)
    }

    /// 按编码搜索（反查）
    fn search_by_code(&self, query: &str) -> Vec<(usize, MatchKind)> {
        let mut seen = HashSet::new();
        let mut results = Vec::new();

        // 1. 精确匹配
        for idx in self.trie.exact_search(query) {
            if seen.insert(idx) {
                results.push((idx, MatchKind::Code(0..query.len())));
            }
        }

        // 2. 前缀匹配
        for idx in self.trie.prefix_search(query) {
            if seen.insert(idx) {
                let code = &self.entries[idx].code.to_lowercase();
                let match_end = query.len().min(code.len());
                results.push((idx, MatchKind::Code(0..match_end)));
            }
        }

        // 3. 编码子串匹配
        for (idx, entry) in self.entries.iter().enumerate() {
            if seen.contains(&idx) {
                continue;
            }
            let code = entry.code.to_lowercase();
            if let Some(pos) = code.find(query) {
                seen.insert(idx);
                results.push((idx, MatchKind::Code(pos..pos + query.len())));
            }
        }

        results
    }

    /// 按文字搜索（正查）
    fn search_by_text(&self, query: &str, query_lower: &str) -> Vec<(usize, MatchKind)> {
        let mut results = Vec::new();

        for (idx, entry) in self.entries.iter().enumerate() {
            // 精确匹配文字
            if entry.text == query {
                results.push((idx, MatchKind::Text(0..entry.text.len())));
                continue;
            }

            // 不区分大小写的精确匹配
            if entry.text.to_lowercase() == query_lower {
                results.push((idx, MatchKind::Text(0..entry.text.len())));
                continue;
            }

            // 子串匹配
            if let Some(pos) = entry.text.find(query) {
                results.push((idx, MatchKind::Text(pos..pos + query.len())));
                continue;
            }

            // 不区分大小写子串匹配
            let text_lower = entry.text.to_lowercase();
            if let Some(pos) = text_lower.find(query_lower) {
                results.push((idx, MatchKind::Text(pos..pos + query_lower.len())));
                continue;
            }

            // 文字任意位置包含查询词
            if entry.text.contains(query) {
                let pos = entry.text.find(query).unwrap();
                results.push((idx, MatchKind::Text(pos..pos + query.len())));
            }
        }

        results
    }

    /// 排序：精确匹配 > 前缀匹配 > 子串匹配
    fn sort_results(
        &self,
        mut results: Vec<(usize, MatchKind)>,
        query: String,
    ) -> Vec<(usize, MatchKind)> {
        results.sort_by(|a, b| {
            let entry_a = &self.entries[a.0];
            let entry_b = &self.entries[b.0];

            let score_a = self.match_score(entry_a, &query, &a.1);
            let score_b = self.match_score(entry_b, &query, &b.1);

            // 得分高的排前面
            score_b
                .cmp(&score_a)
                .then(entry_a.code.len().cmp(&entry_b.code.len()))
                .then(entry_a.text.len().cmp(&entry_b.text.len()))
                .then(entry_a.is_secondary.cmp(&entry_b.is_secondary))
        });

        results.truncate(200);
        results
    }

    fn match_score(&self, entry: &DictEntry, query: &str, kind: &MatchKind) -> u32 {
        match kind {
            MatchKind::Text(range) => {
                let matched = &entry.text[range.start..range.end];
                if matched == query {
                    100
                } else if entry.text.starts_with(query) {
                    80
                } else if matched.len() == query.len() {
                    60
                } else {
                    40
                }
            }
            MatchKind::Code(_range) => {
                let code = &entry.code.to_lowercase();
                if code == query {
                    90
                } else if code.starts_with(query) {
                    70
                } else {
                    50
                }
            }
        }
    }

    /// 获取全部条目引用
    pub fn entries(&self) -> &[DictEntry] {
        &self.entries
    }
}
