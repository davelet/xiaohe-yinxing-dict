# 小鹤音形词典 (xiaohe-yinxing-dict)

小鹤音形（flypy）输入法离线查询工具。字典数据内置在应用中，无需联网，无需解析外部文件。

## 目标

提供一个桌面 GUI 应用，支持快速查询小鹤音形的各类编码，包括：

- **一级简码** — 单键映射字
- **二重简码** — 双键编码字
- **三码填空** — 三键编码词/短语
- **四码全码** — 字/词/短语的完整四码
- **次选字/词** — 二简次选、四码次选词
- **符号 & 快符** — `;` 引导和 `o` 引导的符号
- **部首部件** — `ob` 引导
- **Emoji & 微信表情** — `oi` / `ow` 引导
- **网站直达** — 自定义网址缩略码
- **随心所欲** — 用户自定义编码

## 功能特性

- **离线查询** — 无需联网，所有数据内置在应用中
- **自动更新检查** — 启动时静默检查 GitHub 是否有新版本，有更新时提示用户
- **快速搜索** — 使用 Trie 前缀树 + 文本匹配的混合搜索引擎
- **完整帮助文档** — 内置小鹤音形完整帮助文档

## 帮助文档

本项目包含完整的小鹤音形帮助文档，由 `src/help.rs` 使用静态 Rust 代码直接渲染，无需联网、无需外部文件。

包含以下章节：

| 章节 | 内容 |
|------|------|
| 导读 | 帮助文档主索引 |
| 1 入门概述 | 小鹤音形整体介绍 |
| 1.1 双拼 | 双拼方案详解 |
| 1.2 双形（鹤形） | 鹤形概述 |
| 1.2.1 规则 | 鹤形拆分规则 |
| 1.2.2 字根 | 鹤形字根表 |
| 2 应用 | 输入法应用概述 |
| 2.1 简码 | 简码字词表 |
| 2.2 符号 | 符号编码表 |
| 2.3 Win版 | Windows版使用指南 |
| 2.4 安卓版 | 安卓版使用指南 |
| 2.5 挂接 | 挂接第三方输入法 |
| 3 文章 | 相关文章 |
| 4 问题 | 常见问题解答 |
| 5 指引 | 学习指引 |
| 6 关于 | 关于小鹤 |

## 构建

```bash
cargo build --release
```

### 提交前检查

GitHub Actions 会在 tag 推送时运行格式、lint 和测试检查，建议提交前本地先跑一遍以避免 CI 失败：

```bash
cargo fmt --all -- --check   # 格式检查
cargo clippy --all-targets   # lint 检查
cargo test                   # 测试

# 格式自动修复
cargo fmt --all
```

## 词典数据

词典数据内联在 `src/dict_data.rs` 中，编译时直接嵌入二进制，无需解析外部文件。

### 数据来源

```
my-mac-rime-data/（Rime 词典）
    → 小鹤音形使用说明.md（Markdown 表格，~15000 条）
    → build.rs（编译期解析）
    → src/dict_data.rs（静态 Rust 常量数组，~14938 条）
```

### 重建 dict_data.rs

当 `my-mac-rime-data` 子模块更新后，如果需要重新生成词典数据：

```bash
# 1. 从 git 历史恢复构建工具和源数据
git show HEAD:build.rs > build.rs
git show HEAD:小鹤音形使用说明.md > 小鹤音形使用说明.md

# 2. 构建以触发解析生成
cargo build

# 3. 将生成结果拷贝为源码文件
cp "$(find target -name generated_dict.rs | head -1)" src/dict_data.rs

# 4. 清理临时文件
rm build.rs 小鹤音形使用说明.md
```

## 应用图标

CI 打包使用 `assets/AppIcon.icns` 静态文件，它与运行时（`cargo run`）窗口图标由**完全相同的像素生成算法**生成，保证视觉一致性。

### 更换图标

1. 修改 `src/icon.rs` 中的 `generate_icon_rgba(size)` 函数（调整颜色、图案等）
2. 本地生成新 `.icns`：
   ```bash
   cargo run --features generate-icon --bin generate_icon -- .
   ```
3. 替换静态文件并提交：
   ```bash
   cp AppIcon.icns assets/
   git add assets/AppIcon.icns
   git commit -m "更新应用图标"
   ```

## 子模块：my-mac-rime-data

本项目的字典数据基于 [davelet/my-mac-rime-data](https://github.com/davelet/my-mac-rime-data) 仓库，已将其添加为 git 子模块（`my-mac-rime-data/`）。该仓库包含了小鹤音形（flypy）输入法的 Rime 配置与词典文件：

| 文件 | 说明 |
|---|---|
| `flypy.dict.yaml` | 小鹤音形主词典 |
| `flypy.schema.yaml` | 小鹤音形主方案 |
| `flypy.custom.yaml` | 小鹤音形用户定制 |
| `flypydz.dict.yaml` | 小鹤音形单字词典 |
| `flypydz.schema.yaml` | 小鹤音形单字方案 |
| `double_pinyin_flypy.custom.yaml` | 小鹤双拼用户定制 |
| `double_pinyin_flypy.schema.yaml` | 小鹤双拼方案 |
| `default.custom.yaml` | Rime 全局默认配置 |
| `squirrel.custom.yaml` | Squirrel（macOS 鼠须管）外观定制 |
| `japanese.dict.yaml` / `japanese.{kana,mozc,jmdict}.dict.yaml` / `japanese.schema.yaml` | 日语输入方案 |
| `README.md` | 子模块仓库说明 |
