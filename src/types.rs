/// 视图模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// 默认数据（内置词典）
    Dict,
    /// 输入法数据（本地 Rime 词典）
    Manager,
    /// AI 对话（子窗口激活时主窗口高亮用）
    Chat,
}

/// AI 子窗口标签页
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatTab {
    /// AI 对话
    Conversation,
    /// AI 设置
    Settings,
}
