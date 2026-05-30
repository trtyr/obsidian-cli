//! Hierarchical CLI argument parsing with clap.
//!
//! Structure: `obsidian-cli [global opts] <resource> <action> [args]`

use clap::{Parser, Subcommand, ValueEnum};

/// A blazing-fast CLI for Obsidian vaults.
///
/// Works WITHOUT the Obsidian desktop app — operates directly on vault files.
#[derive(Parser, Debug)]
#[command(name = "obsidian-cli", version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Path to the Obsidian vault (auto-detected if inside one).
    #[arg(long, global = true)]
    pub vault: Option<String>,

    /// Output format.
    #[arg(short, long, global = true, default_value = "text")]
    pub format: OutputFormat,

    /// Verbose output.
    #[arg(short, long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// Output format.
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

/// Top-level commands — each resource is a subcommand with its own actions.
#[derive(Subcommand, Debug)]
pub enum Commands {
    // ═══════════════════════════════════════════════════════════════
    // Note operations
    // ═══════════════════════════════════════════════════════════════
    /// Note CRUD operations.
    #[command(alias = "n")]
    Note {
        #[command(subcommand)]
        action: NoteAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Link operations
    // ═══════════════════════════════════════════════════════════════
    /// Link analysis operations.
    #[command(alias = "l")]
    Link {
        #[command(subcommand)]
        action: LinkAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Tag operations
    // ═══════════════════════════════════════════════════════════════
    /// Tag operations.
    #[command(alias = "t")]
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Property operations
    // ═══════════════════════════════════════════════════════════════
    /// Frontmatter property operations.
    #[command(alias = "p")]
    Prop {
        #[command(subcommand)]
        action: PropAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Task operations
    // ═══════════════════════════════════════════════════════════════
    /// Task operations.
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Daily note operations
    // ═══════════════════════════════════════════════════════════════
    /// Daily note operations.
    #[command(alias = "d")]
    Daily {
        #[command(subcommand)]
        action: DailyAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Search
    // ═══════════════════════════════════════════════════════════════
    /// Search notes by content.
    #[command(alias = "s")]
    Search {
        /// Search query.
        query: String,
        /// Use regex.
        #[arg(short, long)]
        regex: bool,
        /// Case sensitive.
        #[arg(short, long)]
        case_sensitive: bool,
        /// Number of context lines.
        #[arg(long, default_value = "2")]
        context: usize,
        /// Search only in file paths.
        #[arg(long)]
        path_only: bool,
        /// Filter by tag.
        #[arg(long)]
        tag: Option<String>,
        /// Max results.
        #[arg(long, default_value = "20")]
        max_results: usize,
    },

    // ═══════════════════════════════════════════════════════════════
    // Template operations
    // ═══════════════════════════════════════════════════════════════
    /// Template operations.
    #[command(alias = "tmpl")]
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Bookmark operations
    // ═══════════════════════════════════════════════════════════════
    /// Bookmark operations.
    Bookmark {
        #[command(subcommand)]
        action: BookmarkAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Batch operations
    // ═══════════════════════════════════════════════════════════════
    /// Batch operations on multiple notes.
    #[command(alias = "b")]
    Batch {
        #[command(subcommand)]
        action: BatchAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Vault operations
    // ═══════════════════════════════════════════════════════════════
    /// Vault-level operations.
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },

    // ═══════════════════════════════════════════════════════════════
    // Standalone commands (no sub-resource)
    // ═══════════════════════════════════════════════════════════════
    /// Show note outline (headings).
    Outline {
        /// Note path or name.
        note: String,
    },

    /// Show word count.
    Wordcount {
        /// Note path or name (all notes if not provided).
        note: Option<String>,
    },

    /// Show recent notes.
    Recent {
        /// Number of notes to show.
        #[arg(short, long, default_value = "10")]
        count: usize,
    },
}

// ═══════════════════════════════════════════════════════════════════
// Note actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum NoteAction {
    /// Create a new note.
    Create {
        /// Note path or name.
        name: String,
        /// Content to write (reads from stdin if not provided).
        #[arg(short, long)]
        content: Option<String>,
        /// Template to apply.
        #[arg(short, long)]
        template: Option<String>,
        /// Tags to add.
        #[arg(long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Open after creation.
        #[arg(short, long)]
        open: bool,
    },

    /// Read a note's content.
    #[command(alias = "cat")]
    Read {
        /// Note path or name.
        note: String,
        /// Show only frontmatter.
        #[arg(long)]
        frontmatter_only: bool,
        /// Show only body.
        #[arg(long)]
        body_only: bool,
    },

    /// Edit a note (opens in $EDITOR).
    Edit {
        /// Note path or name.
        note: String,
    },

    /// Delete a note.
    #[command(alias = "rm")]
    Delete {
        /// Note path or name.
        note: String,
        /// Skip confirmation.
        #[arg(short, long)]
        force: bool,
        /// Permanent delete (skip trash).
        #[arg(long)]
        permanent: bool,
    },

    /// Move/rename a note.
    #[command(alias = "mv")]
    Move {
        /// Source path or name.
        source: String,
        /// Destination path.
        destination: String,
        /// Skip confirmation.
        #[arg(short, long)]
        force: bool,
    },

    /// Copy a note.
    #[command(alias = "cp")]
    Copy {
        /// Source path or name.
        source: String,
        /// Destination path.
        destination: String,
    },

    /// List files in the vault.
    #[command(alias = "ls")]
    List {
        /// Directory path (relative to vault root).
        path: Option<String>,
        /// Recursive listing.
        #[arg(short, long)]
        recursive: bool,
        /// Filter by extension.
        #[arg(short, long, default_value = "md")]
        extension: String,
    },

    /// Append content to a note.
    Append {
        /// Note path or name.
        note: String,
        /// Content to append.
        content: String,
    },

    /// Prepend content to a note.
    Prepend {
        /// Note path or name.
        note: String,
        /// Content to prepend.
        content: String,
    },

    /// Show note stats.
    Stats {
        /// Note path or name (all notes if not provided).
        note: Option<String>,
    },

    /// Show note aliases.
    Aliases {
        /// Note path or name.
        note: String,
    },

    /// Merge multiple notes into one.
    Merge {
        /// Source notes to merge (comma-separated).
        #[arg(required = true, value_delimiter = ',')]
        sources: Vec<String>,
        /// Destination note name.
        destination: String,
        /// Separator between merged notes.
        #[arg(short, long, default_value = "\n\n---\n\n")]
        separator: String,
        /// Delete source notes after merge.
        #[arg(long)]
        delete_sources: bool,
    },

    /// Split a note by headings into separate notes.
    Split {
        /// Note path or name.
        note: String,
        /// Heading level to split at (1-6).
        #[arg(short, long, default_value = "1")]
        level: usize,
        /// Output directory (defaults to same directory as source).
        #[arg(short, long)]
        output: Option<String>,
        /// Delete source note after split.
        #[arg(long)]
        delete_source: bool,
    },
}

// ═══════════════════════════════════════════════════════════════════
// Link actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum LinkAction {
    /// Show outgoing links from a note.
    #[command(alias = "out")]
    Outgoing {
        /// Note path or name.
        note: String,
    },

    /// Show backlinks to a note.
    #[command(alias = "back")]
    Backlinks {
        /// Note path or name.
        note: String,
    },

    /// Show unresolved links.
    Unresolved,

    /// Show dead-end notes (no incoming links).
    Deadends,

    /// Show orphan notes (no links at all).
    Orphans,

    /// Rename a note and update all references to it.
    Rename {
        /// Current note path or name.
        old_name: String,
        /// New note name.
        new_name: String,
        /// Dry run (show what would be changed).
        #[arg(short, long)]
        dry_run: bool,
        /// Skip confirmation.
        #[arg(long)]
        force: bool,
    },
}

// ═══════════════════════════════════════════════════════════════════
// Tag actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum TagAction {
    /// List all tags.
    #[command(alias = "ls")]
    List {
        /// Sort by count.
        #[arg(short, long)]
        sort: bool,
    },

    /// Show notes with a specific tag.
    Notes {
        /// Tag name (without #).
        tag: String,
    },

    /// Add a tag to a note.
    Add {
        /// Note path or name.
        note: String,
        /// Tag name (without #).
        tag: String,
    },

    /// Remove a tag from a note.
    #[command(alias = "rm")]
    Remove {
        /// Note path or name.
        note: String,
        /// Tag name (without #).
        tag: String,
    },

    /// Rename a tag across all notes.
    Rename {
        /// Old tag name (without #).
        old: String,
        /// New tag name (without #).
        new: String,
        /// Dry run.
        #[arg(short, long)]
        dry_run: bool,
    },
}

// ═══════════════════════════════════════════════════════════════════
// Property actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum PropAction {
    /// Get property value(s).
    #[command(alias = "get")]
    Get {
        /// Note path or name.
        note: String,
        /// Property name (all properties if not provided).
        key: Option<String>,
    },

    /// Set a property.
    #[command(alias = "set")]
    Set {
        /// Note path or name.
        note: String,
        /// Property name.
        key: String,
        /// Property value.
        value: String,
    },

    /// Remove a property.
    #[command(alias = "rm")]
    Remove {
        /// Note path or name.
        note: String,
        /// Property name.
        key: String,
    },
}

// ═══════════════════════════════════════════════════════════════════
// Task actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum TaskAction {
    /// List tasks.
    #[command(alias = "ls")]
    List {
        /// Note path or name (all notes if not provided).
        note: Option<String>,
        /// Show only incomplete tasks.
        #[arg(short, long)]
        pending: bool,
        /// Show only completed tasks.
        #[arg(short, long)]
        done: bool,
        /// Filter by tag.
        #[arg(long)]
        tag: Option<String>,
    },

    /// Add a task to a note.
    Add {
        /// Note path or name.
        note: String,
        /// Task text.
        text: String,
    },

    /// Mark a task as done.
    Done {
        /// Note path or name.
        note: String,
        /// Task line number.
        line: usize,
    },

    /// Mark a task as not done.
    Undone {
        /// Note path or name.
        note: String,
        /// Task line number.
        line: usize,
    },

    /// Remove a task.
    #[command(alias = "rm")]
    Remove {
        /// Note path or name.
        note: String,
        /// Task line number.
        line: usize,
    },
}

// ═══════════════════════════════════════════════════════════════════
// Daily note actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum DailyAction {
    /// Show today's daily note (alias for `read today`).
    Today,

    /// Read a daily note.
    #[command(alias = "cat")]
    Read {
        /// Date (YYYY-MM-DD, defaults to today).
        date: Option<String>,
    },

    /// Create a daily note (applies template if configured).
    Create {
        /// Date (YYYY-MM-DD, defaults to today).
        date: Option<String>,
    },

    /// Append to today's daily note.
    Append {
        /// Content to append.
        content: String,
    },

    /// Prepend to today's daily note.
    Prepend {
        /// Content to prepend.
        content: String,
    },

    /// Show the path for a daily note.
    Path {
        /// Date (YYYY-MM-DD, defaults to today).
        date: Option<String>,
    },

    /// List all daily notes.
    #[command(alias = "ls")]
    List,
}

// ═══════════════════════════════════════════════════════════════════
// Template actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum TemplateAction {
    /// List available templates.
    #[command(alias = "ls")]
    List,

    /// Read a template.
    #[command(alias = "cat")]
    Read {
        /// Template name.
        name: String,
    },

    /// Create a new template from a note.
    Create {
        /// Source note path or name.
        note: String,
        /// Template name.
        name: String,
    },

    /// Delete a template.
    #[command(alias = "rm")]
    Delete {
        /// Template name.
        name: String,
        /// Skip confirmation.
        #[arg(short, long)]
        force: bool,
    },

    /// Apply a template to create a new note.
    Apply {
        /// Template name.
        template: String,
        /// Note name.
        note: String,
        /// Variable substitutions (key=value).
        #[arg(short, long, value_delimiter = ',')]
        vars: Vec<String>,
    },
}

// ═══════════════════════════════════════════════════════════════════
// Bookmark actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum BookmarkAction {
    /// List bookmarks.
    #[command(alias = "ls")]
    List,
}

// ═══════════════════════════════════════════════════════════════════
// Batch actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum BatchAction {
    /// Batch rename notes.
    Rename {
        /// Pattern to match (glob or regex).
        pattern: String,
        /// Replacement string.
        replacement: String,
        /// Use regex instead of glob.
        #[arg(short, long)]
        regex: bool,
        /// Dry run.
        #[arg(short, long)]
        dry_run: bool,
        /// Skip confirmation.
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Batch move notes to a directory.
    Move {
        /// Pattern to match.
        pattern: String,
        /// Destination directory.
        destination: String,
        /// Use regex.
        #[arg(short, long)]
        regex: bool,
        /// Dry run.
        #[arg(short, long)]
        dry_run: bool,
        /// Skip confirmation.
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Batch delete notes.
    Delete {
        /// Pattern to match.
        pattern: String,
        /// Use regex.
        #[arg(short, long)]
        regex: bool,
        /// Dry run.
        #[arg(short, long)]
        dry_run: bool,
        /// Skip confirmation.
        #[arg(short = 'f', long)]
        force: bool,
        /// Permanent delete.
        #[arg(long)]
        permanent: bool,
    },

    /// Batch add tags.
    Tag {
        /// Pattern to match notes.
        pattern: String,
        /// Tags to add (comma-separated).
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Use regex.
        #[arg(short, long)]
        regex: bool,
        /// Dry run.
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Batch remove tags.
    Untag {
        /// Pattern to match notes.
        pattern: String,
        /// Tags to remove (comma-separated).
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,
        /// Use regex.
        #[arg(short, long)]
        regex: bool,
        /// Dry run.
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Batch set property.
    Prop {
        /// Pattern to match notes.
        pattern: String,
        /// Property name.
        key: String,
        /// Property value.
        value: String,
        /// Use regex.
        #[arg(short, long)]
        regex: bool,
        /// Dry run.
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Batch find and replace.
    Replace {
        /// Pattern to match notes.
        note_pattern: String,
        /// Text to find.
        find: String,
        /// Replacement text.
        replace: String,
        /// Use regex for find/replace.
        #[arg(short, long)]
        regex: bool,
        /// Case sensitive.
        #[arg(short, long)]
        case_sensitive: bool,
        /// Dry run.
        #[arg(short, long)]
        dry_run: bool,
        /// Skip confirmation.
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Batch add frontmatter properties.
    Frontmatter {
        /// Pattern to match notes.
        pattern: String,
        /// Properties (key=value).
        #[arg(short, long, value_delimiter = ',')]
        properties: Vec<String>,
        /// Use regex.
        #[arg(short, long)]
        regex: bool,
        /// Dry run.
        #[arg(short, long)]
        dry_run: bool,
    },
}

// ═══════════════════════════════════════════════════════════════════
// Vault actions
// ═══════════════════════════════════════════════════════════════════

#[derive(Subcommand, Debug)]
pub enum VaultAction {
    /// Show vault info.
    Info,

    /// Show vault statistics.
    Stats,

    /// Repair vault issues (broken links, orphans, unresolved).
    Repair {
        /// Fix unresolved links by creating stub notes.
        #[arg(long)]
        fix_unresolved: bool,
        /// Remove dead-end notes (no incoming links).
        #[arg(long)]
        remove_deadends: bool,
        /// Dry run (show what would be fixed).
        #[arg(short, long)]
        dry_run: bool,
    },

    /// Export vault data to JSON.
    Export {
        /// Output file path (stdout if not provided).
        #[arg(short, long)]
        output: Option<String>,
        /// Include note content.
        #[arg(long)]
        include_content: bool,
        /// Pretty print JSON.
        #[arg(short, long)]
        pretty: bool,
    },

    /// Set default vault path (saved to config).
    Set {
        /// Vault path (uses current directory if not provided).
        path: Option<String>,
    },

    /// Remove default vault path from config.
    Unset,

    /// List configured vaults.
    List,
}
