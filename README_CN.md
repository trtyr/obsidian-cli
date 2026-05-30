# obscli

[![Crates.io](https://img.shields.io/crates/v/obscli?style=flat-square&logo=rust)](https://crates.io/crates/obscli)
[![Rust](https://img.shields.io/badge/rust-2024+-ed8225?style=flat-square&logo=rust&logoColor=white)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-22C55E?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20·%20Linux-8B5CF6?style=flat-square)]()

English | [简体中文](README_CN.md)

[🐙 GitHub](https://github.com/trtyr/obsidian-cli) · [📦 crates.io](https://crates.io/crates/obscli) · [⚡ 快速开始](#-快速开始) · [📚 命令](#-命令) · [🏗️ 架构](#️-架构)

**极速 Obsidian 仓库 CLI 工具 — 无需桌面应用即可运行。** 直接操作仓库文件，提供 80+ 命令，覆盖笔记、链接、标签、属性、任务、日记、搜索、模板、书签和批量操作。基于 Rust 构建，采用微内核架构，高度可扩展。

## 🆚 为什么选择 obscli

|  | Obsidian CLI (官方) | obscli |
|---|---|---|
| **桌面依赖** | ❌ 需要运行中的应用 | ✅ 独立运行 |
| **命令结构** | ❌ 扁平命名空间 | ✅ 层次化 `资源 动作` |
| **批量操作** | ❌ 有限 | ✅ 完整批量 CRUD |
| **仓库发现** | ❌ 手动指定 | ✅ 自动检测 + 配置 |
| **离线操作** | ❌ 需要 IPC | ✅ 直接文件访问 |
| **可定制性** | ❌ 闭源 | ✅ 微内核，可扩展 |

> **核心优势**: obscli 将你的仓库视为文件系统，而非运行中的进程 — 实现自动化、CI/CD 集成和无头操作。

## ⚡ 快速开始

```bash
# 从源码安装
cargo install --path .

# 设置仓库（一次性配置）
obscli vault set "~/Documents/MyVault"

# 现在可以在任何目录使用！
obscli vault info
obscli note list --recursive
obscli search "关键词"
obscli tag list --sort
```

## 📚 命令

### 笔记操作

```bash
obscli note create <名称>           # 创建新笔记
obscli note read <笔记>             # 读取笔记内容
obscli note edit <笔记>             # 在 $EDITOR 中打开
obscli note delete <笔记>           # 删除笔记
obscli note move <源> <目标>        # 移动/重命名笔记
obscli note copy <源> <目标>        # 复制笔记
obscli note list [路径]             # 列出文件
obscli note append <笔记> <文本>    # 追加内容
obscli note prepend <笔记> <文本>   # 预置内容
obscli note stats [笔记]            # 显示统计
obscli note aliases <笔记>          # 显示别名
obscli note merge <s1,s2> <目标>    # 合并笔记
obscli note split <笔记> --level 1  # 按标题拆分
```

### 链接分析

```bash
obscli link outgoing <笔记>         # 显示出链
obscli link backlinks <笔记>        # 显示反向链接
obscli link unresolved              # 显示未解析链接
obscli link deadends                # 无入链的笔记
obscli link orphans                 # 完全无链接的笔记
obscli link rename <旧> <新>        # 重命名并更新所有引用
```

### 标签操作

```bash
obscli tag list                     # 列出所有标签
obscli tag list --sort              # 按数量排序
obscli tag notes <标签>             # 显示含此标签的笔记
obscli tag add <笔记> <标签>        # 添加标签
obscli tag remove <笔记> <标签>     # 移除标签
obscli tag rename <旧> <新>         # 跨笔记重命名标签
```

### Frontmatter 属性

```bash
obscli prop get <笔记>              # 显示所有属性
obscli prop get <笔记> <键>         # 显示特定属性
obscli prop set <笔记> <键> <值>    # 设置属性
obscli prop remove <笔记> <键>      # 移除属性
```

### 任务管理

```bash
obscli task list [笔记]             # 列出任务
obscli task list --pending          # 显示未完成任务
obscli task add <笔记> <文本>       # 添加新任务
obscli task done <笔记> <行号>      # 标记完成
obscli task undone <笔记> <行号>    # 重新打开
obscli task remove <笔记> <行号>    # 移除任务
```

### 日记

```bash
obscli daily today                  # 显示今天的日记
obscli daily read [日期]            # 读取日记
obscli daily create [日期]          # 创建日记（应用模板）
obscli daily append <文本>          # 追加到今天
obscli daily prepend <文本>         # 预置到今天
obscli daily path [日期]            # 显示路径
obscli daily list                   # 列出所有日记
```

### 搜索

```bash
obscli search <查询>                # 全文搜索
obscli search <查询> -r             # 正则搜索
obscli search <查询> --tag <标签>   # 按标签过滤
obscli search <查询> --path-only    # 仅搜索路径
```

### 模板

```bash
obscli template list                # 列出模板
obscli template read <名称>         # 读取模板
obscli template create <笔记>       # 从笔记创建模板
obscli template delete <名称>       # 删除模板
obscli template apply <模板> <笔记> # 应用模板
```

### 书签

```bash
obscli bookmark list                # 列出书签
```

### 批量操作

```bash
# 批量重命名笔记
obscli batch rename "旧模式" "新替换" --dry-run
obscli batch rename "旧模式" "新替换" --force

# 批量移动笔记到目录
obscli batch move "*.md" "archive/" --dry-run
obscli batch move "*.md" "archive/" --force

# 批量删除笔记
obscli batch delete "temp-*.md" --dry-run
obscli batch delete "temp-*.md" --force

# 批量添加标签
obscli batch tag "**/*.md" --tags "tag1,tag2" --dry-run

# 批量移除标签
obscli batch untag "**/*.md" --tags "旧标签"

# 批量设置属性
obscli batch prop "**/*.md" "status" "draft" --dry-run

# 批量查找替换
obscli batch replace "**/*.md" "旧文本" "新文本" --dry-run

# 批量添加 frontmatter 属性
obscli batch frontmatter "**/*.md" --properties "category=notes,status=draft"
```

### 仓库操作

```bash
obscli vault info                   # 仓库信息
obscli vault stats                  # 仓库统计
obscli vault set [路径]             # 设置默认仓库
obscli vault unset                  # 移除默认仓库
obscli vault list                   # 列出已配置仓库
obscli vault repair --fix-unresolved --dry-run  # 修复断链
obscli vault export --pretty        # 导出为 JSON
```

### 其他

```bash
obscli outline <笔记>               # 显示标题大纲
obscli wordcount [笔记]             # 字数统计
obscli recent [数量]                # 最近的笔记
```

## ⚙️ 配置

CLI 直接读取 Obsidian 的配置文件：

- `.obsidian/app.json` — 应用设置
- `.obsidian/daily-notes.json` — 日记配置
- `.obsidian/core-plugins.json` — 插件状态

### 默认仓库

设置默认仓库，即可在任何目录使用：

```bash
obscli vault set "~/Documents/MyVault"
# 现在可以在任何地方运行命令！
obscli note list
```

配置文件存储在 `~/.config/obscli/config.json`。

### 输出格式

```bash
obscli -f json <命令>   # JSON 输出
obscli -f csv <命令>    # CSV 输出
obscli -f text <命令>   # 文本输出（默认）
```

## 🏗️ 架构

```
src/
├── main.rs           # 入口，仓库发现 + 命令分发
├── cli.rs            # 层次化 clap 子命令定义
├── kernel/           # 核心模块（微内核）
│   ├── mod.rs        # 模块导出
│   ├── vault.rs      # 仓库配置解析
│   ├── note.rs       # 笔记数据模型（frontmatter, wikilinks, 标签, 任务）
│   ├── index.rs      # 仓库索引（by_name, tag_index, backlinks）
│   ├── search.rs     # 搜索引擎（模糊, 正则, 标签过滤）
│   ├── output.rs     # 输出格式化
│   ├── fs.rs         # 文件系统工具
│   └── config.rs     # 配置管理
└── plugins/          # 功能模块（可插拔）
    ├── mod.rs        # 命令分发到插件
    ├── files.rs      # 笔记 CRUD 操作
    ├── links.rs      # 链接分析 + 重命名
    ├── tags.rs       # 标签 list/notes/add/remove/rename
    ├── properties.rs # 属性 get/set/remove
    ├── tasks.rs      # 任务 list/add/done/undone/remove
    ├── daily.rs      # 日记操作
    ├── search.rs     # 搜索命令
    ├── templates.rs  # 模板操作
    ├── bookmarks.rs  # 书签操作
    ├── batch.rs      # 批量操作
    └── misc.rs       # 仓库 info/stats/repair/export/set/unset/list, 大纲, 字数
```

## 🔧 构建

- **Rust** ≥ 1.85 (edition 2024)
- **无需 C 库** — obscli 是纯 Rust 项目

```bash
# 调试构建
cargo build

# 发布构建
cargo build --release

# 本地安装
cargo install --path .
```

## 📄 许可证

MIT

---

⭐ 觉得有用？在 [GitHub](https://github.com/trtyr/obsidian-cli) 上给我们一个星标！
