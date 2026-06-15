use std::collections::HashMap;

/// 前缀树节点
#[derive(Debug, Default)]
pub struct TrieNode {
    pub children: HashMap<char, Box<TrieNode>>,
    /// 记录以当前节点为终点的条目索引
    pub entries: Vec<usize>,
}

/// 前缀树，用于编码（键序列）的快速匹配
#[derive(Debug, Default)]
pub struct Trie {
    root: TrieNode,
}

impl Trie {
    pub fn new() -> Self {
        Trie {
            root: TrieNode::default(),
        }
    }

    /// 插入一个编码及其对应的条目索引
    pub fn insert(&mut self, key: &str, entry_idx: usize) {
        let mut node = &mut self.root;
        for ch in key.chars() {
            node = node
                .children
                .entry(ch)
                .or_insert_with(|| Box::new(TrieNode::default()));
        }
        node.entries.push(entry_idx);
    }

    /// 精确查找：返回完全匹配的条目索引列表
    pub fn exact_search(&self, key: &str) -> Vec<usize> {
        let mut node = &self.root;
        for ch in key.chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return Vec::new(),
            }
        }
        node.entries.clone()
    }

    /// 前缀搜索：返回以给定前缀开头的所有条目索引列表
    pub fn prefix_search(&self, prefix: &str) -> Vec<usize> {
        let mut node = &self.root;
        for ch in prefix.chars() {
            match node.children.get(&ch) {
                Some(next) => node = next,
                None => return Vec::new(),
            }
        }
        // 收集该节点下所有条目
        let mut results = Vec::new();
        collect_all_entries(node, &mut results);
        results
    }
}

fn collect_all_entries(node: &TrieNode, results: &mut Vec<usize>) {
    results.extend_from_slice(&node.entries);
    for child in node.children.values() {
        collect_all_entries(child, results);
    }
}
