# 小鹤音形词典 2.0 — AI 对话功能需求文档

> 版本: 1.0 | 日期: 2026-07-09 | 状态: 草案 | 实现框架: rig-core

---

## 1. 产品概述

### 1.1 目标

为小鹤音形词典增加 AI 对话能力，用户绑定自己的大模型 API Key 后，可通过自然语言与 AI 交互，获取关于小鹤音形输入法的专业解答。AI 将调用本地词典工具获取精确数据，仅回答与小鹤音形相关的问题。

### 1.2 技术选型

**rig-core** — Rust LLM 应用框架

选择理由：
- 统一 API 支持 20+ 模型提供商（OpenAI、DeepSeek、Ollama 等）
- 内置 Agent 模式，原生支持 Function Calling
- 流式响应支持
- 类型安全的 Tool 定义（`#[derive(Tool)]` 宏）
- 对话历史管理（ConversationMemory）
- 活跃维护（最新版 0.39.0，2026-06-19）

### 1.3 核心价值

| 场景 | 现状 | 2.0 体验 |
|------|------|----------|
| 查询编码 | 手动输入搜索框 | "帮我查'赢'字的编码" → AI 自动查询并返回 |
| 理解规则 | 阅读帮助文档 | "鹤形拆分规则是什么" → AI 总结+举例 |
| 记忆方法 | 自行摸索 | "怎么快速记忆二简词" → AI 给出记忆技巧 |
| 输入法配置 | 搜索网络 | "如何配置鼠须管" → AI 基于内置文档回答 |
| 造句练习 | 无 | "用'我'的编码造个句子" → AI 生成+标注编码 |

---

## 2. 用户画像

- **目标用户**: 小鹤音形输入法学习者和使用者
- **技术水平**: 从初学者到高级用户
- **使用场景**: 桌面端日常使用，查询编码、学习规则、解决问题

---

## 3. 功能需求

### 3.1 AI 配置管理

#### 3.1.1 设置入口

- 新增 `ViewMode::Chat`，AI 设置复用现有设置入口（不新增独立 `ViewMode::AiSettings`），在设置面板中新增 AI 配置区块
- **配置持久化**：AI 配置写入 `ai_config.json`（与主 `config.json` 同目录），不混入主配置。理由：
  - 可单独排除出云同步，避免敏感信息泄露
  - 主配置结构（`AppConfig`：`rime_user_dir` / `external_dict_files`）无需变更，降低耦合
- **API Key 安全存储**：API Key 不存入 JSON 文件，改用 OS 钥匙串（`keyring` crate）存储：
  - macOS: Keychain
  - Windows: Credential Manager
  - Linux: Secret Service (gnome-keyring / kwallet)
  - `ai_config.json` 中仅存 `keyring_service` / `keyring_user` 标识符，运行时通过 `keyring` crate 读取实际 Key
  - 提供"清除 API Key"按钮，同时删除钥匙串条目和配置文件中的标识符
- 新增 `AiConfig` 结构体，提供独立的 `load()` / `save()` 方法，路径同 `AppConfig::config_path()` 的兄弟文件 `ai_config.json`

#### 3.1.2 配置项

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `keyring_service` | String | `xiaohe-yinxing-dict` | OS 钥匙串服务名 |
| `keyring_user` | String | `default` | OS 钥匙串用户名 |
| `api_url` | String | `https://api.openai.com/v1` | API 端点（仅 Custom 提供商生效，见 3.1.4） |
| `model` | String | `gpt-4o-mini` | 模型名称 |
| `provider` | enum | OpenAI | 提供商类型（OpenAI/DeepSeek/Ollama/Anthropic/Gemini/Custom） |
| `temperature` | f32 | 0.7 | 生成温度 |
| `max_tokens` | u32 | 2048 | 最大输出 token 数（上限 8192） |
| `enable_external_dict_tool` | bool | false | 是否启用外部词典工具（查询会上传本地词库片段） |
| `privacy_acknowledged` | bool | false | 是否已确认隐私提示 |
| `history_rounds` | u32 | 10 | 上下文保留轮数（范围 4-30） |

#### 3.1.3 系统提示词（Preamble）

使用 rig-core 的 Agent 构建接口设置提示词，告知 AI 仅回答小鹤音形相关问题，使用中文，优先调用本地工具查询精确数据，编码使用等宽字体格式展示。

#### 3.1.4 Provider 支持

rig-core 原生支持的提供商，无需额外适配：

| 提供商 | rig-core crate | 说明 |
|--------|----------------|------|
| OpenAI | `rig::providers::openai` | GPT-4o, GPT-4o-mini 等 |
| DeepSeek | `rig::providers::deepseek` | deepseek-chat, deepseek-reasoner |
| Ollama | `rig::providers::ollama` | 本地部署，免费 |
| Anthropic | `rig::providers::anthropic` | Claude 系列 |
| Gemini | `rig::providers::gemini` | Google Gemini |
| Custom | 实现 `CompletionModel` trait | 兼容 OpenAI 格式的任意 API |

**`api_url` 说明**：每个内置 provider 自带默认 base URL，`api_url` 配置项**仅对 Custom 提供商生效**。使用内置 provider 时忽略此字段，避免用户误配导致请求失败。**URL 校验**：保存设置时校验 URL 格式——必须是 `http://` 或 `https://` 开头，不能为空字符串，不能包含空格；校验失败时在按钮旁显示红色错误提示，阻止保存。

**reqwest 依赖说明**：rig-core 内部自带 HTTP client（基于 reqwest），与项目现有 `reqwest`（blocking，用于词典搜索等同步 IO）互不干扰，**两者共存**，无需移除现有依赖。AI 相关请求全部走 rig-core 的异步 client。

#### 3.1.5 隐私与成本提示

- **数据外发告知**：使用 AI 对话时，用户的提问及工具调用返回的数据会发送到所配置的第三方大模型服务。必须在首次进入对话视图时弹出一次性隐私提示，并记录"已知晓"状态到 `ai_config.json`，避免重复打扰。
- **外部词典特别提示**：`search_external_dict` 工具会将用户本地 Rime 词典的查询片段上传给模型。在设置界面为该工具单独提供开关（`enable_external_dict_tool`，默认关闭），关闭后 Agent 不注册此工具。
- **成本控制**：在设置界面展示当前模型的大致单价说明（静态文案，非实时计费），并限制 `max_tokens` 上限为 8192，避免误操作产生高额费用。**请求频率控制**：在设置中增加可选的最小请求间隔（默认 500ms），防止用户连续快速误操作触发大量 API 调用产生高额费用。在 `ChatState` 中记录上次请求时间戳，发送前校验。

### 3.2 AI 对话视图

#### 3.2.1 视图切换

- 新增 `ViewMode::Chat`，通过顶部导航栏进入
- 快捷键: `Ctrl/Cmd + J` 切换到对话视图（避开 `Ctrl/Cmd+N`，该组合在多数应用中为"新建窗口"）
- 保留原有词典查询视图，三视图可自由切换
- **未配置 API Key 时的空状态**：进入对话视图显示引导卡片，包含简短说明 + "去设置"按钮，点击跳转到设置面板的 AI 配置区块。不显示输入框，避免用户输入后才报错。
- **隐私提示首次弹窗**：首次进入对话视图时（`ai_config.json` 中 `privacy_acknowledged` 为 false），弹出一次性隐私提示（见 3.1.5），用户确认后不再出现。
- **对话历史列表空状态**：对话历史列表为空时显示引导文本："暂无对话历史，点击「新建对话」开始与小鹤助手交流"。
- **快捷键说明**：`Ctrl/Cmd + J` 切换到对话视图。注意：macOS 中 `Cmd+J` 默认用于"显示查看选项"，如冲突可改用 `Cmd+Shift+M`。

#### 3.2.2 界面布局

```
┌─────────────────────────────────────────────────────────────┐
│  [词典]  [管理]  [AI 对话]           [设置] [关于]          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ 🤖 你好！我是小鹤音形助手，有什么可以帮你的？        │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ 👤 帮我查一下"赢"字的编码                            │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ 🤖 正在查询...                                       │  │
│  │                                                       │  │
│  │ "赢"字的编码是 ylv，属于四码全码分类。               │  │
│  │                                                       │  │
│  │ 拆解：                                                │  │
│  │ - y: 亡（yáng wáng）                                │  │
│  │ - l: 月（yuè）                                      │  │
│  │ - v: 贝（bèi）                                      │  │
│  │                                                       │  │
│  │ 💡 记忆技巧：亡+月+贝=赢，记住"亡月贝"即可          │  │
│  │                                                       │  │
│  │ [📋 复制编码] [🔍 在词典中查看]                      │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                             │
├─────────────────────────────────────────────────────────────┤
│  输入消息...                                   [发送] 📎   │
└─────────────────────────────────────────────────────────────┘
```

#### 3.2.3 消息展示

- 用户消息: 右侧气泡，灰色背景
- AI 消息: 左侧气泡，白色背景
- AI 消息支持 Markdown 渲染（使用现有 `egui_commonmark`）
- 代码块使用等宽字体，适合展示编码对齐
- 工具调用过程显示 loading 动画
- 工具调用结果可折叠/展开

#### 3.2.4 操作按钮

每条 AI 回复下方提供快捷操作：
- **复制编码**: 从消息关联的 `tool_calls` 结果中提取编码复制到剪贴板（而非从自然语言中抽取，避免不可靠）
- **在词典中查看**: 跳转到词典视图并自动搜索
- **复制回复**: 复制整条 AI 回复
- **重新生成**: 重新发送上一条用户消息

#### 3.2.5 对话管理

- **新建对话**: 清空当前对话历史
- **历史对话**: 保存最近 50 个会话（可配置）
- **删除单个会话**: 对话历史侧栏中，每个会话条目提供右键菜单或悬停删除按钮，支持删除单个会话。删除时同步删除 `conversations/` 目录下的对应 JSON 文件
- **导出对话**: 导出为 Markdown 文件（格式见附录 8.4）
- **清空历史**: 删除所有对话记录
- **生成中重入策略**: 生成进行中时，输入框禁用，发送按钮变为"⏹ 停止"按钮（见 3.4.2）。用户只能选择中断当前生成（保留已接收内容）或等待完成，不支持排队或并发多条请求。这样避免上下文错乱和多线程竞争同一 `ChatState`。
- **生成中切换会话策略**: 如果生成进行中用户点击另一个历史会话，自动 abort 当前任务，保留已接收内容并标注"已中断"。不允许在生成中切换到新会话并保留旧任务运行（资源浪费、状态混乱）。

### 3.3 本地工具系统

#### 3.3.1 工具定义

使用 rig-core 的 Tool 机制定义本地工具，每个工具包含输入参数类型和输出结果类型。工具实现中调用本地词典搜索引擎进行查询。

#### 3.3.2 完整工具列表

| 工具名 | 描述 | 参数 | 返回值 |
|--------|------|------|--------|
| `search_text` | 根据汉字/词组查询编码 | `query: String, category: Option<String>` | 编码列表（上限 20 条） |
| `search_code` | 根据编码反查汉字/词组 | `code: String` | 词条列表（上限 20 条） |
| `get_help` | 获取帮助文档内容 | `chapter: Option<String>` | 帮助文档 |
| `list_categories` | 列出所有编码分类 | 无 | 分类列表 |
| `get_category_stats` | 获取分类统计信息 | `category: Option<String>` | 统计信息 |
| `search_external_dict` | 查询用户本地的 Rime 词典文件（如个人扩展词库） | `query: String` | 外部词典词条列表（上限 20 条） |

**工具返回值 schema**（AI 实际接收的 JSON 结构，直接影响回答质量）：

```jsonc
// search_text
{
  "results": [
    { "text": "赢", "code": "ylv", "category": "四码全码", "is_secondary": false }
  ],
  "total": 1
}

// search_code
{
  "results": [
    { "text": "赢", "code": "ylv", "category": "四码全码", "is_secondary": false }
  ],
  "total": 1
}

// list_categories
{
  "categories": [
    { "name": "一级简码", "count": 26 },
    { "name": "四码全码", "count": 14938 }
  ]
}

// search_external_dict（注意：此数据会随工具调用上传至模型，见 3.1.5）
{
  "results": [
    { "text": "自定义词", "code": "xxx", "source_file": "我的词库.dict.yaml" }
  ],
  "total": 1
}
```

字段说明：`is_secondary` 表示是否为容错码。所有列表工具统一返回 `{ results: [...], total: N }` 结构，便于 AI 判断是否有更多结果。**工具返回条数上限 20 条**，超出时 `total` 告知完整数量，AI 可通过细化查询获取更多。

#### 3.3.3 工具注册

通过 rig-core Agent 构建器注册所有本地工具，包括编码查询、反查、帮助文档、分类列表、统计信息、本地 Rime 词典查询等工具。

**`get_help` 工具说明**：帮助文档通过 `HelpManager` 提供查询能力（`chapters()` / `get_chapter(id)` / `search()`）。章节靠 `id` 标识而非中文名，需在 preamble 中向 AI 提供章节清单，AI 通过 `id` 查询。AI 模块需通过 `Arc` 共享 App 持有的 `HelpManager` 实例。

#### 3.3.4 工具调用流程

rig-core 自动处理 Function Calling 循环：

```
用户输入 → Agent 分析 → 调用工具 → 执行本地查询 → 结果返回 Agent → Agent 整合回答
```

无需手动管理循环，rig-core 内置处理。

### 3.4 流式响应

#### 3.4.1 流式输出

rig-core 原生支持流式响应，通过 Agent 的流式接口逐 chunk 接收 AI 回复文本和工具调用状态，逐字更新 UI。

**UI 更新节流**：每次收到 token 都调用 `ctx.request_repaint()` 会导致 CPU 使用率飙升。建议每收到 50ms 间隔或每 5 个 chunk 才触发重绘，用 `Instant::now()` 记录上次重绘时间，未达间隔时不调用。需注意 egui 的 `request_repaint()` 本身可能有内部节流机制，需实际测试确认。

#### 3.4.2 中断生成

持有流式任务的 `JoinHandle`（tokio task），中断时直接 `abort()`（drop future），再在 `ChatState` 把已收内容标 `is_interrupted`，并在该消息下提供"重新生成"按钮，点击后以原用户消息重新发起请求。

> **为何不用 AtomicBool 轮询**：rig 的 Function Calling 循环由框架内部驱动，无法在循环里插入轮询点，`AtomicBool` 大概率拦不住。直接 abort task 是可靠方案。

### 3.5 对话历史

#### 3.5.1 rig-core 内置支持

rig-core 提供 `ConversationMemory` trait，通过 Agent 构建器的 memory 方法注入。多轮对话自动维护历史，后续请求自动包含上文上下文。

#### 3.5.2 多会话存储结构

支持 50 个独立会话，每个会话包含：
- `conversation_id: String` - 唯一标识
- `title: String` - 会话标题（自动取首条 user 消息前 20 字）
- `created_at: u64` - 创建时间戳
- `messages: Vec<ChatMessage>` - 消息历史

存储方案：`conversations/` 目录下每会话一个 JSON 文件（`{conversation_id}.json`），便于单独导出和管理。文件名使用 UUID 避免冲突。

#### 3.5.3 持久化

手动将对话历史序列化为 JSON 持久化到本地文件，支持加载和保存。

#### 3.5.4 上下文窗口管理

**方案 A（推荐）：外部管理历史**

不使用 `ConversationMemory`，改为在 `ChatState` 中自行维护消息列表。发送请求前，自行截断消息列表（保留系统提示词 + 最近 N 轮），直接传给 agent builder。完全可控，不依赖 `ConversationMemory` 的内部实现。

**方案 B（备选）：使用 ConversationMemory**

若 rig-core 的 `ConversationMemory` 支持外部截断重提交，则使用框架能力。需在 P0 spike 中验证。

**裁剪策略**（两种方案通用）：

- 保留最近 N 轮对话（默认 N=10，可在设置中调整，范围 4-30）
- 系统提示词（Preamble）始终保留，不计入裁剪
- 工具调用及其结果占较大 token，单独计入预算：若单轮工具结果超过 2000 token，仅保留结果摘要（前 500 字 + 末尾 500 字），完整结果存于本地，AI 需要时可再次调用工具获取

> 未采用"摘要压缩"方案，因额外调用模型生成摘要会增加延迟和成本，且小鹤音形查询场景多为单轮工具调用，长对话需求有限。

---

## 4. 技术设计

### 4.1 模块结构

```
src/
  ai/                          # 新增 AI 模块
    mod.rs                     # 模块入口
    config.rs                  # AI 配置管理（AiConfig，独立 ai_config.json）
    client.rs                  # rig-core 客户端封装
    tools.rs                   # 本地工具定义（#[derive(Tool)]）
    chat.rs                    # 对话状态管理（ChatState）
    history.rs                 # 对话历史持久化
  ui/
    chat_view.rs               # 新增对话视图
    ai_settings.rs             # 新增 AI 设置区块（嵌入现有设置面板，非独立 ViewMode）
```

**与现有代码的关系**：
- `AiConfig` 独立于 `AppConfig`（`src/config.rs`），单独读写 `ai_config.json`，不修改 `AppConfig` 结构
- `ViewMode::Chat` 新增至现有 `ViewMode` 枚举（`src/app.rs:44`），与 `Dict` / `Manager` 并列
- 工具系统复用现有搜索引擎，但需区分对待：
  - **主词典** `SearchEngine<DictEntry>`：只构建一次、不变，可安全 `Arc` 共享
  - **外部词典** `SearchEngine<ExternalDictEntry>`：`ManagerState` 重新加载时会整体重建，需使用 `Arc<RwLock<SearchEngine<ExternalDictEntry>>>`，确保 AI 工具始终访问最新数据

### 4.2 核心数据结构

新增 AI 配置、聊天状态、聊天消息等数据结构。

**`AiConfig`**（持久化至 `ai_config.json`）：
- `keyring_service: String` - OS 钥匙串服务名
- `keyring_user: String` - OS 钥匙串用户名
- `api_url: String` - API 端点（仅 Custom 提供商生效）
- `model: String` - 模型名称
- `provider: enum Provider` - 提供商类型（OpenAI/DeepSeek/Ollama/Anthropic/Gemini/Custom）
- `temperature: f32` - 生成温度
- `max_tokens: u32` - 最大输出 token 数
- `enable_external_dict_tool: bool` - 是否启用外部词典工具（默认 false）
- `privacy_acknowledged: bool` - 是否已确认隐私提示（默认 false）
- `history_rounds: u32` - 上下文保留轮数（默认 10，见 3.5.4）

**`ChatState`**：
- `messages: Vec<ChatMessage>` - 消息列表
- `is_generating: bool` - 是否正在生成
- `current_task: Option<JoinHandle<()>>` - 当前生成任务的句柄（用于 abort 中断）
- `current_conversation_id: Option<String>` - 当前对话 ID

**`ChatMessage`**：
- `role: enum Role`（User/AI/System/Tool）
  - `User` / `AI`：普通对话消息，显示在对话流中
  - `System`：系统消息（由 preamble 管理），不存入消息列表，仅用于 Agent 构建
  - `Tool`：工具调用结果，不显示在对话流中，用于折叠展示工具过程
- `content: String` - 消息内容
- `timestamp: u64` - Unix 时间戳
- `tool_calls: Vec<ToolCallRecord>` - 该消息触发的工具调用记录（仅 AI 消息）
- `is_interrupted: bool` - 是否被中断（见 3.4.2）

### 4.3 工具实现

工具实现中调用本地搜索引擎和帮助文档管理器，封装为 rig-core 兼容的 Tool 类型。每个工具包含描述、参数类型和异步调用方法。

### 4.4 Agent 构建

根据用户配置的提供商类型，创建对应 rig-core 客户端，注入系统提示词和本地工具，构建 Agent 实例。

### 4.5 线程模型

```
UI 线程 (egui)
    │
    ├── Arc<Mutex<ChatState>> ──────────────┐
    │                                        │
    └── ctx.request_repaint()                │
                                             │
后台线程 (std::thread::spawn)                │
    │                                        │
    ├── rig::Agent                           │
    │   ├── .stream_prompt()                 │
    │   │   └── SSE 流式响应                 │
    │   │       ├── 文本 → 更新 ChatState    │
    │   │       └── tool_calls → 自动执行 ───┤
    │   │           └── 结果返回 Agent ──────┤
    │   │                                    │
    │   └── .prompt()                        │
    │       └── 非流式响应                   │
    │                                        │
    └── ctx.request_repaint() ──────────────┘
```

### 4.6 依赖变更

```toml
[dependencies]
# 新增依赖
rig-core = "0.39"                    # LLM Agent 框架
tokio = { version = "1", features = ["macros", "rt", "sync"] }  # rig-core 要求，单后台线程无需 multi-thread
keyring = "3"                        # OS 钥匙串存储 API Key（Linux 需 dbus-secret-service）

# 现有依赖（无需变更）
eframe = "0.34.3"
reqwest = { version = "0.12", features = ["json", "rustls-tls", "blocking"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
egui_commonmark = "0.23"
```

**注意**: rig-core 要求 tokio 运行时，但可以在独立线程中运行，不阻塞 egui 主线程。

### 4.7 tokio 集成方案

由于 egui 是同步框架，rig-core 要求 tokio 异步运行时，方案是在后台线程中启动独立的 tokio runtime，通过共享状态（Arc<Mutex<ChatState>>）与 UI 线程通信，完成后通过 ctx.request_repaint() 触发 UI 刷新。

**tokio runtime 生命周期**：在 `DictApp` 中持有一个 `Option<tokio::runtime::Runtime>`，随 app 启动创建、随 app 销毁。避免每次 AI 请求都创建新 runtime，降低开销。

### 4.8 代理设置复用

项目已引入 `sysproxy`（见 `Cargo.toml`）用于系统代理检测。rig-core 的 HTTP client（基于 reqwest）默认读取系统代理环境变量（`HTTP_PROXY` / `HTTPS_PROXY`）。AI 请求**复用系统代理**，无需额外配置：

- 若检测到系统代理处于开启状态，rig-core client 自动走代理
- 在设置界面展示当前代理状态（只读），告知用户 AI 请求将经过该代理
- 不提供独立的 AI 代理配置项，避免与系统代理冲突

> 如未来需要为 AI 单独配置代理（如绕过系统代理直连），可在 `AiConfig` 中新增 `proxy_url: Option<String>` 字段覆盖，当前版本不实现。

### 4.9 日志与调试

AI 模块的关键事件应输出日志，便于调试：

| 事件 | 日志内容 |
|------|---------|
| 创建 Agent | provider, model, 启用工具列表 |
| 发送请求 | 截断后的消息轮数、总字符数 |
| 工具调用 | 工具名 + 参数 |
| 工具返回 | 结果条数 |
| 流式接收 | 首 token 延迟 |
| 请求完成 | 总耗时、总 token 数（如模型返回） |
| 错误 | 错误类型 + 详细信息 |

使用 `log` crate（与现有 `eframe` 兼容），或直接用 `eprintln!` 输出到 stderr。

---

## 5. UI 设计

### 5.1 视图切换

顶部导航栏新增 "AI 对话" 按钮：

```
[词典]  [管理]  [AI 对话]           [设置] [关于]
         ↑                           ↑
    新增按钮                       配置 API Key
```

### 5.2 对话视图布局

采用三栏布局：

```
┌────────────────────────────────────────────────────────────┐
│                    顶部导航栏                               │
├──────────┬─────────────────────────────────┬───────────────┤
│          │                                 │               │
│  对话    │         主对话区域               │   工具调用    │
│  历史    │                                 │   详情面板    │
│  列表    │   ┌───────────────────────┐     │               │
│          │   │ 🤖 你好！...          │     │  查询: 赢     │
│  对话1   │   └───────────────────────┘     │  结果: ylv    │
│  对话2   │   ┌───────────────────────┐     │  分类: 四码   │
│  对话3   │   │ 👤 帮我查...          │     │               │
│          │   └───────────────────────┘     │               │
│          │   ┌───────────────────────┐     │               │
│          │   │ 🤖 "赢"字编码是...    │     │               │
│          │   └───────────────────────┘     │               │
│          │                                 │               │
│          ├─────────────────────────────────┤               │
│          │  [📎] 输入消息...    [发送] [⏹] │               │
└──────────┴─────────────────────────────────┴───────────────┘
```

### 5.3 设置界面

```
┌─────────────────────────────────────────────────────────────┐
│  AI 助手设置                                                │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  API 配置                                                   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ 提供商:   [OpenAI ▼]                                │   │
│  │ API Key:  [••••••••••••••••••••] 👁                │   │
│  │ 模型:     [gpt-4o-mini                    ]         │   │
│  │ 温度:     [0.7] (0.0-2.0)                          │   │
│  │ Max Token:[2048] (100-8192)                         │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  快速配置                                                   │
│  [OpenAI] [DeepSeek] [Ollama 本地]                         │
│                                                             │
│  高级选项                                                   │
│  ☐ 启用外部词典工具（查询会上传本地词库片段，见隐私说明）  │
│  上下文保留轮数: [10] (4-30)                                │
│  当前系统代理: 已开启 / 已关闭（只读，AI 请求将经过此代理）│
│                                                             │
│  [测试连接]  [保存设置]                                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**测试连接实现**：点击"测试连接"后，发送一条最小成本请求验证配置是否可用：

- 优先调用 `GET {api_url}/models` 列出模型列表（成本为零，不消耗 token）
- 若该端点不支持（如 Ollama 的 `/api/tags` 或自定义提供商），降级为发送一条 `max_tokens=1` 的测试消息（内容："hi"），仅验证连通性和 Key 有效性
- 测试结果显示在按钮旁：✅ 连接成功（显示模型数量或延迟）/ ❌ 失败（显示错误码和简短原因）
- 测试期间按钮禁用并显示 loading 状态，超时阈值 10 秒

### 5.4 主题适配

- 支持浅色/深色主题切换（复用现有主题系统）
- AI 消息气泡颜色与整体风格一致
- 工具调用状态使用 loading 动画

---

## 6. 实现计划

### 6.1 阶段划分

| 阶段 | 内容 | 预估工时 |
|------|------|----------|
| P0 | rig-core 集成 + AI 配置模块 + spike 验证（流式+工具、上下文截断、SearchEngine Send/Sync、API 对齐） | 3-4 天 |
| P1 | 本地工具系统（6 个 Tool） | 2-3 天 |
| P1.5 | 流式响应 + 工具联调 | 2-3 天 |
| P2 | 对话视图 UI | 3-4 天 |
| P3 | 对话管理（多会话、持久化） | 2-3 天 |
| P4 | 测试 + 优化 + 文档 | 2-3 天 |
| **合计** | | **14-20 天** |

### 6.2 里程碑

- **M1 (Week 1)**: rig-core 集成 + 基础对话（无工具调用）
- **M2 (Week 2)**: 工具系统 + 流式响应
- **M3 (Week 3)**: 完整 UI + 对话管理 + 测试

### 6.3 风险与对策

| 风险 | 影响 | 对策 |
|------|------|------|
| rig-core tokio 与 egui 冲突 | 线程阻塞 | 独立线程运行 tokio runtime |
| API Key 安全 | 用户信任 | OS 钥匙串存储（keyring crate），不明文落盘 |
| SearchEngine 不是 Send + Sync | 线程模型崩溃 | P0 spike 验证，若不安全则改为 `Arc<RwLock<SearchEngine<DictEntry>>>` 或通过闭包封装 |
| HelpManager 线程安全 | 后台线程无法引用 | 确认 `HelpManager` 是否 `Send + Sync`（预计是），AI 模块通过 `Arc<HelpManager>` 持有 |
| 网络不稳定 | 请求失败 | 超时重试，离线提示 |
| 429 限流 | 请求被拒 | 遇 `429 Too Many Requests` 显示"请求过于频繁，请稍后"，尊重 `retry-after` 头，避免无限重试 |
| 模型不支持 Function Calling | 工具调用失败 | **降级检测策略**：首次请求返回 400 且提示"没有 Function Calling 支持"时触发降级，标记 `is_downgraded = true`，后续请求不再注册工具。提供"恢复工具调用"按钮供手动重试。降级后 AI 仅基于静态注入的帮助文档摘要回答规则类问题，UI 提示"当前模型不支持工具调用" |
| 流式响应解析复杂 | 开发成本 | rig-core 原生支持，无需手动解析 SSE |
| rig 流式+工具调用能力未验证 | 返工风险 | P0 阶段写最小 spike 验证：(a) 流式响应同时做 Function Calling 循环；(b) 上下文截断方案验证。验证通过后再定裁剪方案 |
| rig-core API 与 PRD 不符 | 开发受阻 | **P0 必做**：开工前对 `docs.rs/rig-core` 核对 0.39 实际 API（宏签名、Agent builder、流式方法名、Tool derive 签名），固化 PRD 里的 API 名字，避免开发时发现接口不匹配 |

### 6.4 实现进度

| 模块 | 内容 | 状态 |
|------|------|------|
| AI 配置 | 配置项定义、独立 ai_config.json 持久化（含 enable_external_dict_tool / privacy_acknowledged / history_rounds） | ⬜ 未开始 |
| API Key 安全 | OS 钥匙串存储（keyring crate），配置文件仅存标识符 | ⬜ 未开始 |
| Provider 接入 | OpenAI / DeepSeek / Ollama / Anthropic / Gemini / Custom 多提供商支持 | ⬜ 未开始 |
| 系统提示词 | AI 行为边界定义，仅回答小鹤音形相关问题 | ⬜ 未开始 |
| 对话视图 UI | 消息气泡、输入框、发送/中断按钮、Markdown 渲染、未配置空状态 | ⬜ 未开始 |
| 设置界面 | AI 配置表单、快速配置按钮、测试连接、高级选项（外部词典开关/轮数/代理状态） | ⬜ 未开始 |
| 隐私提示 | 首次进入对话视图一次性弹窗，记录确认状态 | ⬜ 未开始 |
| 代理复用 | 复用系统代理，设置界面展示代理状态 | ⬜ 未开始 |
| search_text 工具 | 根据汉字/词组查询编码 | ⬜ 未开始 |
| search_code 工具 | 根据编码反查汉字/词组 | ⬜ 未开始 |
| get_help 工具 | 获取内置帮助文档内容 | ⬜ 未开始 |
| list_categories 工具 | 列出所有编码分类 | ⬜ 未开始 |
| get_category_stats 工具 | 获取分类统计信息 | ⬜ 未开始 |
| search_external_dict 工具 | 查询用户本地的 Rime 词典文件（受 enable_external_dict_tool 开关控制） | ⬜ 未开始 |
| 工具降级 | 模型不支持 Function Calling 时移除工具，静态注入帮助文档摘要 | ⬜ 未开始 |
| 流式响应 | 逐 chunk 更新 UI，支持中断生成（abort task，保留部分内容并标注"已中断"） | ⬜ 未开始 |
| 上下文管理 | 按轮数裁剪历史（默认 10 轮），工具结果超 2000 token 时摘要 | ⬜ 未开始 |
| 多会话存储 | 50 个会话，conversations/ 目录，每会话一个 JSON 文件 | ⬜ 未开始 |
| tokio 集成 | 后台线程 runtime，共享状态与 UI 通信 | ⬜ 未开始 |
| 视图切换 | 导航栏新增 AI 对话入口，快捷键 Ctrl/Cmd+J，三视图自由切换 | ⬜ 未开始 |
| 生成中重入控制 | 生成中禁用输入，发送按钮变停止按钮，不支持并发请求 | ⬜ 未开始 |
| 快捷操作 | 复制编码（从 tool_calls 结果提取）、在词典中查看、重新生成 | ⬜ 未开始 |
| 主题适配 | 浅色/深色主题，对话气泡样式 | ⬜ 未开始 |
| spike 验证 | P0 阶段验证 (a) 流式+工具调用 (b) 上下文截断方案 (c) SearchEngine Send/Sync 验证 | ⬜ 未开始 |
| API 对齐 | 开工前核对 rig-core 0.39 实际 API，固化 PRD 中的 API 名 | ⬜ 未开始 |
| 日志调试 | AI 模块关键事件日志输出（log crate 或 eprintln） | ⬜ 未开始 |
| 单会话删除 | 对话历史侧栏右键/悬停删除单个会话 | ⬜ 未开始 |
| 请求频率控制 | 最小请求间隔（默认 500ms），ChatState 记录时间戳 | ⬜ 未开始 |
| URL 校验 | Custom Provider URL 格式校验（http/https 开头、非空、无空格） | ⬜ 未开始 |
| UI 更新节流 | 流式接收时 50ms/5 chunk 间隔触发重绘 | ⬜ 未开始 |

---

## 7. 测试用例

### 7.1 基础对话

| 用例 | 输入 | 期望输出 |
|------|------|----------|
| 打招呼 | "你好" | AI 自我介绍，引导用户提问小鹤音形相关问题 |
| 无关问题 | "今天天气怎么样" | AI 礼貌拒绝，引导回小鹤音形话题 |
| 编码查询 | "赢字怎么编码" | AI 调用 search_text，返回 ylv + 拆解说明 |
| 反查 | "yl 是什么字" | AI 调用 search_code，返回匹配词条列表 |

### 7.2 工具调用

| 用例 | 输入 | 期望工具调用 |
|------|------|--------------|
| 查询编码 | "查一下'赢'的编码" | search_text(query="赢") |
| 反查编码 | "aa 对应什么字" | search_code(code="aa") |
| 获取帮助 | "简码怎么用" | get_help(chapter="简码") |
| 列出分类 | "有哪些编码分类" | list_categories() |

### 7.3 边界情况

| 用例 | 输入 | 期望行为 |
|------|------|----------|
| 空输入 | "" | 提示用户输入 |
| 超长输入 | 1000+ 字符 | 截断或提示缩短 |
| API Key 未配置 | 发送消息 | 提示先配置 API Key |
| API 请求失败 | 网络错误 | 显示错误信息，提供重试按钮 |
| 流式中断 | 网络断开 | 显示已接收内容，标注"已中断"，提供重新生成按钮 |
| 生成中再次操作 | 生成中点击发送 | 发送按钮已变停止按钮，输入框禁用，无法发送新消息 |
| 上下文超限 | 连续 15+ 轮对话 | 自动裁剪至最近 10 轮，系统提示词保留，请求正常完成 |
| 工具结果过大 | 查询返回 5000+ 字符结果 | 工具结果截断为摘要（前 500 + 后 500 字），AI 仍可正常回答 |
| 模型不支持工具 | 使用无 Function Calling 的模型 | 提示降级模式，仅基于帮助文档回答规则类问题 |
| 隐私提示未确认 | 首次进入对话视图 | 弹出一次性隐私提示，确认后方可使用 |
| 外部词典工具关闭 | enable_external_dict_tool=false | Agent 不注册 search_external_dict，AI 无法查询用户本地词库 |

---

## 8. 附录

### 8.1 rig-core 资源

- 官方文档: https://docs.rs/rig-core
- GitHub: https://github.com/0xPlaygrounds/rig
- Agent 指南: https://docs.rig.rs/docs/concepts/agent
- Tool 定义: https://docs.rig.rs/docs/concepts/tools

### 8.2 术语表

| 术语 | 说明 |
|------|------|
| rig-core | Rust LLM 应用框架 |
| Agent | rig-core 的核心抽象，组合模型+提示词+工具 |
| Tool | rig-core 中 AI 可调用的函数，使用 `#[derive(Tool)]` 定义 |
| Preamble | 系统提示词，定义 Agent 行为边界 |
| Function Calling | 大模型调用外部函数的能力 |
| SSE | Server-Sent Events，服务器推送事件流 |

### 8.3 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| 1.0 | 2026-07-09 | 初始草案，明确使用 rig-core |
| 1.1 | 2026-07-09 | 补充：独立 ai_config.json、隐私提示、工具返回值 schema、上下文窗口管理、并发重入策略、代理复用、测试连接实现、工具降级细化、空状态与快捷键调整 |
| 2.0 | 2026-07-10 | 评审修正：删除 full_code 幻觉字段、API Key 改用 OS 钥匙串、中断机制改为 abort task、外部引擎改 RwLock、provider 枚举统一为 6 个、api_url 仅对 Custom 生效、工具返回条数封顶 20、补充 get_help 共享 HelpManager、多会话存储结构、429 限流处理、spike 验证、API 对齐 |
| 2.1 | 2026-07-10 | 合并评审意见：AiConfig 字段澄清（keyring_service/keyring_user）、ConversationMemory 裁剪方案 A/B、search_text 增加 category 参数、请求频率控制、日志与调试、单会话删除、生成中切换策略、空状态设计、降级检测时机细化、URL 校验、UI 更新节流、tokio features 精简、keyring 依赖、tokio runtime 生命周期、SearchEngine/Send Sync 风险项、对话导出格式、快捷键冲突说明 |

### 8.4 对话导出格式

```markdown
# 对话标题
> 时间: 2026-07-10 10:00 | 导出时间: 2026-07-10 11:00

---

**用户** (10:00):
赢字怎么编码

**AI** (10:01) [工具调用: search_text]:
"赢"字的编码是 `ylv`

- 拆解: y(亡) + l(月) + v(贝)
- 分类: 四码全码
- 记忆技巧: 亡+月+贝=赢
```
