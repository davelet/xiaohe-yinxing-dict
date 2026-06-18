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

    /// 获取分类的真实条目数量
    pub fn count_by_category(&self, category_filter: Option<Category>) -> usize {
        match category_filter {
            Some(cat) => self.entries.iter().filter(|e| e.category == cat).count(),
            None => self.entries.len(),
        }
    }

    /// 按分类获取所有条目（最多100条）
    pub fn get_by_category(&self, category_filter: Option<Category>) -> Vec<(usize, MatchKind)> {
        let mut results = Vec::new();
        for (idx, entry) in self.entries.iter().enumerate() {
            match category_filter {
                Some(cat) if entry.category == cat => {
                    results.push((idx, MatchKind::Code(0..0)));
                }
                None => {
                    results.push((idx, MatchKind::Code(0..0)));
                }
                _ => {}
            }
            if results.len() >= 100 {
                break;
            }
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dict::Category;
    use crate::dict::DictEntry;

    fn make_entry(text: &'static str, code: &'static str, category: Category) -> DictEntry {
        DictEntry {
            text,
            code,
            category,
            is_secondary: false,
        }
    }

    fn build_engine(entries: &[DictEntry]) -> SearchEngine {
        SearchEngine::build(entries)
    }

    #[test]
    fn test_search_by_code_exact() {
        let entries = vec![
            make_entry("一", "yi", Category::YiJiJianMa),
            make_entry("二", "er", Category::ErChongJianMa),
            make_entry("三", "sa", Category::SanMaTianKong),
        ];
        let engine = build_engine(&entries);
        let results = engine.search("yi", None);
        assert_eq!(results.len(), 1);
        assert_eq!(engine.entries[results[0].0].text, "一");
    }

    #[test]
    fn test_search_by_code_prefix() {
        let entries = vec![
            make_entry("起", "q", Category::YiJiJianMa),
            make_entry("情", "qb", Category::ErChongJianMa),
            make_entry("请", "qc", Category::ErChongJianMa),
        ];
        let engine = build_engine(&entries);
        let results = engine.search("q", None);
        // Should find all entries starting with "q"
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_by_text() {
        let entries = vec![
            make_entry("你好", "nih", Category::SiMaQuanMaCi),
            make_entry("你们", "nim", Category::SiMaQuanMaCi),
        ];
        let engine = build_engine(&entries);
        let results = engine.search("你好", None);
        assert_eq!(results.len(), 1);
        assert_eq!(engine.entries[results[0].0].text, "你好");
    }

    #[test]
    fn test_search_by_text_substring() {
        let entries = vec![
            make_entry("中华人民共和国", "vhrg", Category::SiMaQuanMaCi),
            make_entry("美国", "mg", Category::SiMaQuanMaCi),
        ];
        let engine = build_engine(&entries);
        let results = engine.search("人民", None);
        assert_eq!(results.len(), 1);
        assert_eq!(engine.entries[results[0].0].text, "中华人民共和国");
    }

    #[test]
    fn test_category_filter() {
        let entries = vec![
            make_entry("起", "q", Category::YiJiJianMa),
            make_entry("情", "qb", Category::ErChongJianMa),
            make_entry("请", "qc", Category::ErChongJianMa),
        ];
        let engine = build_engine(&entries);
        let results = engine.search("q", Some(Category::ErChongJianMa));
        assert_eq!(results.len(), 2);
        assert!(
            results
                .iter()
                .all(|(idx, _)| engine.entries[*idx].category == Category::ErChongJianMa)
        );
    }

    #[test]
    fn test_empty_query() {
        let entries = vec![make_entry("一", "yi", Category::YiJiJianMa)];
        let engine = build_engine(&entries);
        let results = engine.search("", None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_count_by_category() {
        let entries = vec![
            make_entry("起", "q", Category::YiJiJianMa),
            make_entry("情", "qb", Category::ErChongJianMa),
            make_entry("请", "qc", Category::ErChongJianMa),
        ];
        let engine = build_engine(&entries);
        assert_eq!(engine.count_by_category(Some(Category::YiJiJianMa)), 1);
        assert_eq!(engine.count_by_category(Some(Category::ErChongJianMa)), 2);
        assert_eq!(engine.count_by_category(None), 3);
    }

    #[test]
    fn test_get_by_category() {
        let entries = vec![
            make_entry("起", "q", Category::YiJiJianMa),
            make_entry("情", "qb", Category::ErChongJianMa),
            make_entry("请", "qc", Category::ErChongJianMa),
        ];
        let engine = build_engine(&entries);
        let results = engine.get_by_category(Some(Category::YiJiJianMa));
        assert_eq!(results.len(), 1);
        assert_eq!(engine.entries[results[0].0].text, "起");
    }

    #[test]
    fn test_code_substring_search() {
        let entries = vec![
            make_entry("装", "vdh", Category::SiMaQuanMaZi),
            make_entry("问", "wfh", Category::SiMaQuanMaZi),
        ];
        let engine = build_engine(&entries);
        // Search for "dh" should match "vdh"
        let results = engine.search("dh", None);
        assert!(
            results
                .iter()
                .any(|(idx, _)| engine.entries[*idx].text == "装")
        );
    }

    #[test]
    fn test_case_insensitive_search() {
        let entries = vec![make_entry("你好", "nih", Category::SiMaQuanMaCi)];
        let engine = build_engine(&entries);
        let results = engine.search("NIH", None);
        assert!(!results.is_empty());
        assert_eq!(engine.entries[results[0].0].text, "你好");
    }
}
