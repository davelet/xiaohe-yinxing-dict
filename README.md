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

## 构建

```bash
cargo build --release
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
