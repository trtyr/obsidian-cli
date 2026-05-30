# obscli

[![Crates.io](https://img.shields.io/crates/v/obscli?style=flat-square&logo=rust)](https://crates.io/crates/obscli)
[![Rust](https://img.shields.io/badge/rust-2024+-ed8225?style=flat-square&logo=rust&logoColor=white)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-22C55E?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20·%20Linux-8B5CF6?style=flat-square)]()

English | [简体中文](README_CN.md)

[🐙 GitHub](https://github.com/trtyr/obsidian-cli) · [📦 crates.io](https://crates.io/crates/obscli) · [⚡ Quick Start](#-quick-start) · [📚 Commands](#-commands) · [🏗️ Architecture](#️-architecture)

**A blazing-fast CLI for Obsidian vaults — works without the desktop app.** Operates directly on vault files with 80+ commands covering notes, links, tags, properties, tasks, daily notes, search, templates, bookmarks, and batch operations. Built in Rust with microkernel architecture for maximum extensibility.

## 🆚 Why obscli

|  | Obsidian CLI (Official) | obscli |
|---|---|---|
| **Desktop dependency** | ❌ Requires running app | ✅ Works standalone |
| **Command structure** | ❌ Flat namespace | ✅ Hierarchical `resource action` |
| **Batch operations** | ❌ Limited | ✅ Full batch CRUD |
| **Vault discovery** | ❌ Manual | ✅ Auto-detect + config |
| **Offline operation** | ❌ Needs IPC | ✅ Direct file access |
| **Customization** | ❌ Closed source | ✅ Microkernel, extensible |

> **Key insight**: obscli treats your vault as a file system, not a running process — enabling automation, CI/CD integration, and headless operation.

## 🚀 Installation

```bash
# From crates.io (recommended)
cargo install obscli

# From source
git clone https://github.com/trtyr/obsidian-cli.git
cd obsidian-cli
cargo install --path .
```

## ⚡ Quick Start

```bash
# Set your vault (one-time setup)
obscli vault set "~/Documents/MyVault"

# Now use from anywhere!
obscli vault info
obscli note list --recursive
obscli search "keyword"
obscli tag list --sort
```

## 📚 Commands

### Note Operations

```bash
obscli note create <name>           # Create new note
obscli note read <note>             # Read note content
obscli note edit <note>             # Open in $EDITOR
obscli note delete <note>           # Delete note
obscli note move <src> <dest>       # Move/rename note
obscli note copy <src> <dest>       # Copy note
obscli note list [path]             # List files
obscli note append <note> <text>    # Append to note
obscli note prepend <note> <text>   # Prepend to note
obscli note stats [note]            # Show statistics
obscli note aliases <note>          # Show aliases
obscli note merge <s1,s2> <dest>    # Merge notes
obscli note split <note> --level 1  # Split by headings
```

### Link Analysis

```bash
obscli link outgoing <note>         # Show outgoing links
obscli link backlinks <note>        # Show backlinks
obscli link unresolved              # Show unresolved links
obscli link deadends                # Notes with no incoming links
obscli link orphans                 # Notes with no links at all
obscli link rename <old> <new>      # Rename and update all references
```

### Tag Operations

```bash
obscli tag list                     # List all tags
obscli tag list --sort              # Sort by count
obscli tag notes <tag>              # Show notes with tag
obscli tag add <note> <tag>         # Add tag to note
obscli tag remove <note> <tag>      # Remove tag from note
obscli tag rename <old> <new>       # Rename tag across all notes
```

### Frontmatter Properties

```bash
obscli prop get <note>              # Show all properties
obscli prop get <note> <key>        # Show specific property
obscli prop set <note> <key> <val>  # Set property
obscli prop remove <note> <key>     # Remove property
```

### Task Management

```bash
obscli task list [note]             # List tasks
obscli task list --pending          # Show incomplete tasks
obscli task add <note> <text>       # Add new task
obscli task done <note> <line>      # Mark task done
obscli task undone <note> <line>    # Reopen task
obscli task remove <note> <line>    # Remove task
```

### Daily Notes

```bash
obscli daily today                  # Show today's note
obscli daily read [date]            # Read daily note
obscli daily create [date]          # Create daily note (applies template)
obscli daily append <text>          # Append to today
obscli daily prepend <text>         # Prepend to today
obscli daily path [date]            # Show path
obscli daily list                   # List all daily notes
```

### Search

```bash
obscli search <query>               # Full-text search
obscli search <query> -r            # Regex search
obscli search <query> --tag <tag>   # Filter by tag
obscli search <query> --path-only   # Search in paths only
```

### Templates

```bash
obscli template list                # List templates
obscli template read <name>         # Read template
obscli template create <note>       # Create template from note
obscli template delete <name>       # Delete template
obscli template apply <tmpl> <note> # Apply template
```

### Bookmarks

```bash
obscli bookmark list                # List bookmarks
```

### Batch Operations

```bash
# Batch rename notes
obscli batch rename "old-pattern" "new-replacement" --dry-run
obscli batch rename "old-pattern" "new-replacement" --force

# Batch move notes to directory
obscli batch move "*.md" "archive/" --dry-run
obscli batch move "*.md" "archive/" --force

# Batch delete notes
obscli batch delete "temp-*.md" --dry-run
obscli batch delete "temp-*.md" --force

# Batch add tags
obscli batch tag "**/*.md" --tags "tag1,tag2" --dry-run

# Batch remove tags
obscli batch untag "**/*.md" --tags "old-tag"

# Batch set property
obscli batch prop "**/*.md" "status" "draft" --dry-run

# Batch find and replace
obscli batch replace "**/*.md" "old text" "new text" --dry-run

# Batch add frontmatter properties
obscli batch frontmatter "**/*.md" --properties "category=notes,status=draft"
```

### Vault Operations

```bash
obscli vault info                   # Vault info
obscli vault stats                  # Vault statistics
obscli vault set [path]             # Set default vault
obscli vault unset                  # Remove default vault
obscli vault list                   # List configured vaults
obscli vault repair --fix-unresolved --dry-run  # Fix broken links
obscli vault export --pretty        # Export to JSON
```

### Misc

```bash
obscli outline <note>               # Show headings
obscli wordcount [note]             # Word count
obscli recent [count]               # Recent notes
```

## ⚙️ Configuration

The CLI reads Obsidian's configuration files directly:

- `.obsidian/app.json` — app settings
- `.obsidian/daily-notes.json` — daily notes config
- `.obsidian/core-plugins.json` — plugin state

### Default Vault

Set a default vault to use from any directory:

```bash
obscli vault set "~/Documents/MyVault"
# Now you can run commands from anywhere!
obscli note list
```

Config is stored at `~/.config/obscli/config.json`.

### Output Formats

```bash
obscli -f json <command>   # JSON output
obscli -f csv <command>    # CSV output
obscli -f text <command>   # Text output (default)
```

## 🏗️ Architecture

```
src/
├── main.rs           # Entry point, vault discovery + command dispatch
├── cli.rs            # Hierarchical clap subcommand definitions
├── kernel/           # Core modules (microkernel)
│   ├── mod.rs        # Module exports
│   ├── vault.rs      # Vault configuration parsing
│   ├── note.rs       # Note data model (frontmatter, wikilinks, tags, tasks)
│   ├── index.rs      # Vault indexing (by_name, tag_index, backlinks)
│   ├── search.rs     # Search engine (fuzzy, regex, tag filter)
│   ├── output.rs     # Output formatting
│   ├── fs.rs         # Filesystem helpers
│   └── config.rs     # Configuration management
└── plugins/          # Feature modules (pluggable)
    ├── mod.rs        # Command dispatch to plugins
    ├── files.rs      # note CRUD operations
    ├── links.rs      # link analysis + rename
    ├── tags.rs       # tag list/notes/add/remove/rename
    ├── properties.rs # prop get/set/remove
    ├── tasks.rs      # task list/add/done/undone/remove
    ├── daily.rs      # daily note operations
    ├── search.rs     # search command
    ├── templates.rs  # template operations
    ├── bookmarks.rs  # bookmark operations
    ├── batch.rs      # batch operations
    └── misc.rs       # vault info/stats/repair/export/set/unset/list, outline, wordcount
```

## 🔧 Building

- **Rust** ≥ 1.85 (edition 2024)
- **No C library required** — obscli is pure Rust

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Install locally
cargo install --path .
```

## 📄 License

MIT

---

⭐ Found this useful? Give it a star on [GitHub](https://github.com/trtyr/obsidian-cli).
