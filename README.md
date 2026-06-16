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

---

## macOS 签名与公证（可选）

CI 默认生成的 `.dmg` 是**未签名**的。macOS 用户首次打开会被 Gatekeeper 拦截，需要右键 → 打开，极为不便。如果要发"开箱即用"的 Release，请按以下步骤配置 Apple Developer ID 签名 + Notarization。

### 一次性准备

1. 拥有 Apple Developer 账号（[developer.apple.com](https://developer.apple.com)，年费 $99）。
2. 在 Xcode → Settings → Accounts → 你的账号 → Manage Certificates 中，创建一个 **Developer ID Application** 证书。
3. 导出证书为 `.p12` 格式（Keychain Access → Certificates → 右键 → Export → 选 `.p12` 并设密码）。
4. 记录以下信息：
   - `.p12` 导出时设置的密码
   - 你的 **Apple ID** 邮箱
   - 在 [appleid.apple.com](https://appleid.apple.com) → App-Specific Passwords 生成一个用于 `notarytool` 的密码
   - **Team ID**（Developer 账号详情页可以看到，10 位字符串）

### 在 GitHub 仓库配置 Secrets

进入 **Settings → Secrets and variables → Actions → New repository secret**，依次添加：

| Secret 名称 | 内容 |
|---|---|
| `APPLE_CERT_P12_BASE64` | 在终端执行 `base64 -i cert.p12 \| pbcopy` 复制 |
| `APPLE_CERT_PASSWORD` | 导出 `.p12` 时设置的密码 |
| `KEYCHAIN_PASSWORD` | 任意强密码（CI 临时 keychain 用） |
| `APPLE_ID` | 你的 Apple ID 邮箱 |
| `APPLE_APP_SPECIFIC_PASSWORD` | 步骤 4 生成的 app-specific 密码 |
| `APPLE_TEAM_ID` | 你的 10 位 Team ID |

> **注意**：Secret 名称必须**完全一致**（大写 + 下划线）。只要 `APPLE_CERT_P12_BASE64` 未配置，签名/公证步骤自动跳过，DMG 仍会正常生成。

### 验证

- 打一个 tag（如 `v0.1.0`），等待 `Build & Release` workflow 跑完。
- 下载 `.dmg`，挂载后**直接双击** `.app` 能打开 = 配置成功。
- 在终端 `codesign -dv --verbose=4 /Applications/小鹤音形词典.app` 可查看签名信息。
- 在终端 `spctl -a -vv /Applications/小鹤音形词典.app` 应显示 `source=Notarized Developer ID`。

### 证书轮换

- `.p12` 过期（默认 1 年，可续期）或 Team 变更时，重新导出 `.p12`，更新 `APPLE_CERT_P12_BASE64` 即可。
- 公证 (`notarytool`) 不需要更新证书 — 它只验证你的 Apple ID + Team ID + app-specific password。
