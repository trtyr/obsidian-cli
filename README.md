# obsidian-cli

[![Crates.io](https://img.shields.io/crates/v/obsidian-cli?style=flat-square&logo=rust)](https://crates.io/crates/obsidian-cli)
[![Rust](https://img.shields.io/badge/rust-2024+-ed8225?style=flat-square&logo=rust&logoColor=white)](https://rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-22C55E?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20·%20Linux-8B5CF6?style=flat-square)]()

English | [简体中文](README_CN.md)

[🐙 GitHub](https://github.com/trtyr/obsidian-cli) · [📦 crates.io](https://crates.io/crates/obsidian-cli) · [⚡ Quick Start](#-quick-start) · [📚 Commands](#-commands) · [🏗️ Architecture](#️-architecture)

**A blazing-fast CLI for Obsidian vaults — works without the desktop app.** Operates directly on vault files with 80+ commands covering notes, links, tags, properties, tasks, daily notes, search, templates, bookmarks, and batch operations. Built in Rust with microkernel architecture for maximum extensibility.

## 🆚 Why obsidian-cli

|  | Obsidian CLI (Official) | obsidian-cli |
|---|---|---|
| **Desktop dependency** | ❌ Requires running app | ✅ Works standalone |
| **Command structure** | ❌ Flat namespace | ✅ Hierarchical `resource action` |
| **Batch operations** | ❌ Limited | ✅ Full batch CRUD |
| **Vault discovery** | ❌ Manual | ✅ Auto-detect + config |
| **Offline operation** | ❌ Needs IPC | ✅ Direct file access |
| **Customization** | ❌ Closed source | ✅ Microkernel, extensible |

> **Key insight**: obsidian-cli treats your vault as a file system, not a running process — enabling automation, CI/CD integration, and headless operation.

## ⚡ Quick Start

```bash
# Install from source
cargo install --path .

# Set your vault (one-time setup)
obsidian-cli vault set "~/Documents/MyVault"

# Now use from anywhere!
obsidian-cli vault info
obsidian-cli note list --recursive
obsidian-cli search "keyword"
obsidian-cli tag list --sort
```

## 📚 Commands

### Note Operations

```bash
obsidian-cli note create <name>           # Create new note
obsidian-cli note read <note>             # Read note content
obsidian-cli note edit <note>             # Open in $EDITOR
obsidian-cli note delete <note>           # Delete note
obsidian-cli note move <src> <dest>       # Move/rename note
obsidian-cli note copy <src> <dest>       # Copy note
obsidian-cli note list [path]             # List files
obsidian-cli note append <note> <text>    # Append to note
obsidian-cli note prepend <note> <text>   # Prepend to note
obsidian-cli note stats [note]            # Show statistics
obsidian-cli note aliases <note>          # Show aliases
obsidian-cli note merge <s1,s2> <dest>    # Merge notes
obsidian-cli note split <note> --level 1  # Split by headings
```

### Link Analysis

```bash
obsidian-cli link outgoing <note>         # Show outgoing links
obsidian-cli link backlinks <note>        # Show backlinks
obsidian-cli link unresolved              # Show unresolved links
obsidian-cli link deadends                # Notes with no incoming links
obsidian-cli link orphans                 # Notes with no links at all
obsidian-cli link rename <old> <new>      # Rename and update all references
```

### Tag Operations

```bash
obsidian-cli tag list                     # List all tags
obsidian-cli tag list --sort              # Sort by count
obsidian-cli tag notes <tag>              # Show notes with tag
obsidian-cli tag add <note> <tag>         # Add tag to note
obsidian-cli tag remove <note> <tag>      # Remove tag from note
obsidian-cli tag rename <old> <new>       # Rename tag across all notes
```

### Frontmatter Properties

```bash
obsidian-cli prop get <note>              # Show all properties
obsidian-cli prop get <note> <key>        # Show specific property
obsidian-cli prop set <note> <key> <val>  # Set property
obsidian-cli prop remove <note> <key>     # Remove property
```

### Task Management

```bash
obsidian-cli task list [note]             # List tasks
obsidian-cli task list --pending          # Show incomplete tasks
obsidian-cli task add <note> <text>       # Add new task
obsidian-cli task done <note> <line>      # Mark task done
obsidian-cli task undone <note> <line>    # Reopen task
obsidian-cli task remove <note> <line>    # Remove task
```

### Daily Notes

```bash
obsidian-cli daily today                  # Show today's note
obsidian-cli daily read [date]            # Read daily note
obsidian-cli daily create [date]          # Create daily note (applies template)
obsidian-cli daily append <text>          # Append to today
obsidian-cli daily prepend <text>         # Prepend to today
obsidian-cli daily path [date]            # Show path
obsidian-cli daily list                   # List all daily notes
```

### Search

```bash
obsidian-cli search <query>               # Full-text search
obsidian-cli search <query> -r            # Regex search
obsidian-cli search <query> --tag <tag>   # Filter by tag
obsidian-cli search <query> --path-only   # Search in paths only
```

### Templates

```bash
obsidian-cli template list                # List templates
obsidian-cli template read <name>         # Read template
obsidian-cli template create <note>       # Create template from note
obsidian-cli template delete <name>       # Delete template
obsidian-cli template apply <tmpl> <note> # Apply template
```

### Bookmarks

```bash
obsidian-cli bookmark list                # List bookmarks
```

### Batch Operations

```bash
# Batch rename notes
obsidian-cli batch rename "old-pattern" "new-replacement" --dry-run
obsidian-cli batch rename "old-pattern" "new-replacement" --force

# Batch move notes to directory
obsidian-cli batch move "*.md" "archive/" --dry-run
obsidian-cli batch move "*.md" "archive/" --force

# Batch delete notes
obsidian-cli batch delete "temp-*.md" --dry-run
obsidian-cli batch delete "temp-*.md" --force

# Batch add tags
obsidian-cli batch tag "**/*.md" --tags "tag1,tag2" --dry-run

# Batch remove tags
obsidian-cli batch untag "**/*.md" --tags "old-tag"

# Batch set property
obsidian-cli batch prop "**/*.md" "status" "draft" --dry-run

# Batch find and replace
obsidian-cli batch replace "**/*.md" "old text" "new text" --dry-run

# Batch add frontmatter properties
obsidian-cli batch frontmatter "**/*.md" --properties "category=notes,status=draft"
```

### Vault Operations

```bash
obsidian-cli vault info                   # Vault info
obsidian-cli vault stats                  # Vault statistics
obsidian-cli vault set [path]             # Set default vault
obsidian-cli vault unset                  # Remove default vault
obsidian-cli vault list                   # List configured vaults
obsidian-cli vault repair --fix-unresolved --dry-run  # Fix broken links
obsidian-cli vault export --pretty        # Export to JSON
```

### Misc

```bash
obsidian-cli outline <note>               # Show headings
obsidian-cli wordcount [note]             # Word count
obsidian-cli recent [count]               # Recent notes
```

## ⚙️ Configuration

The CLI reads Obsidian's configuration files directly:

- `.obsidian/app.json` — app settings
- `.obsidian/daily-notes.json` — daily notes config
- `.obsidian/core-plugins.json` — plugin state

### Default Vault

Set a default vault to use from any directory:

```bash
obsidian-cli vault set "~/Documents/MyVault"
# Now you can run commands from anywhere!
obsidian-cli note list
```

Config is stored at `~/.config/obsidian-cli/config.json`.

### Output Formats

```bash
obsidian-cli -f json <command>   # JSON output
obsidian-cli -f csv <command>    # CSV output
obsidian-cli -f text <command>   # Text output (default)
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
- **No C library required** — obsidian-cli is pure Rust

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
