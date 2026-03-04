mod parser;
mod types;
mod walker;

use clap::{Parser, Subcommand};
use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;
use toml::Table;

/// Discover and summarise a codebase for AI-assisted navigation.
///
/// tldr walks a directory tree looking for .tldr files. Each .tldr file is a
/// small TOML file placed at the root of a folder to describe its purpose:
///
///   description = "Handles JWT authentication and session management"
///   tags        = ["auth", "security"]
///   docs        = ["README.md", "https://wiki.example.com/auth"]
///
/// The default output is one line per entry:
///
///   src/auth: Handles JWT authentication and session management
///   src/db:   Database models and connection pooling
///
/// This is intentionally terse — an AI can read it in full and then open
/// individual .tldr files or source files for deeper detail.
#[derive(Parser)]
#[command(name = "tldr", verbatim_doc_comment)]
struct Cli {
    /// Root directory to search. Defaults to the current working directory.
    path: Option<PathBuf>,

    /// Also scan *.md files for YAML frontmatter containing a `tldr:` block.
    ///
    /// Useful when descriptions already live in README frontmatter rather than
    /// dedicated .tldr files. The expected frontmatter shape is:
    ///
    ///   ---
    ///   tldr:
    ///     description: "Short summary"
    ///     tags: ["auth"]
    ///     docs: ["README.md"]
    ///   ---
    #[arg(long)]
    frontmatter: bool,

    /// Only output entries whose tags include TAG.
    ///
    /// Useful for narrowing results to a specific domain when exploring a large
    /// codebase. Use `tldr taglist` first to see what tags are available.
    #[arg(long, value_name = "TAG")]
    filter: Option<String>,

    /// Limit directory traversal to at most N levels deep.
    ///
    /// Depth 1 means only the immediate children of the root are searched.
    /// Omit to traverse the full tree. Combine with --limit to do a broad
    /// shallow sweep before going deeper.
    #[arg(long, value_name = "N")]
    depth: Option<usize>,

    /// Stop after outputting N entries.
    ///
    /// Entries are emitted in depth-first, lexicographic order, so the first N
    /// results are the shallowest/earliest paths. Useful for a quick initial
    /// orientation before requesting more.
    #[arg(long, value_name = "N")]
    limit: Option<usize>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Print all unique tags found across .tldr files, one per line, sorted.
    ///
    /// Use this to discover what domains or themes are annotated in the
    /// codebase before filtering with `tldr --filter <TAG>`.
    Taglist {
        /// Root directory to search. Defaults to the current working directory.
        path: Option<PathBuf>,

        /// Also scan *.md files for YAML frontmatter containing a `tldr:` block.
        #[arg(long)]
        frontmatter: bool,

        /// Limit directory traversal to at most N levels deep.
        #[arg(long, value_name = "N")]
        depth: Option<usize>,
    },

    /// Create a blank .tldr file in a directory.
    ///
    /// Writes a template .tldr to the target directory (default: current
    /// directory) so you can fill in the description, tags, and docs fields.
    /// Exits with an error if a .tldr file already exists.
    ///
    /// Example template created:
    ///
    ///   description = ""
    ///   tags        = []
    ///   docs        = []
    #[command(verbatim_doc_comment)]
    Init {
        /// Directory in which to create the .tldr file. Defaults to the current
        /// working directory.
        path: Option<PathBuf>,
    },

    /// Validate .tldr files found under a directory.
    ///
    /// Checks every .tldr file for:
    ///   - Required `description` field is present and non-empty
    ///   - Description is at most 50 tokens (estimated as chars ÷ 4)
    ///   - `tags` and `docs` fields, if present, are arrays
    ///   - No unknown fields
    ///
    /// Exits with a non-zero status code if any file fails validation, making
    /// it suitable for use in CI pipelines.
    #[command(verbatim_doc_comment)]
    Validate {
        /// Root directory to search. Defaults to the current working directory.
        path: Option<PathBuf>,

        /// Limit directory traversal to at most N levels deep.
        #[arg(long, value_name = "N")]
        depth: Option<usize>,

        /// Override the maximum number of tokens allowed in the description.
        /// Tokens are estimated as characters ÷ 4. Default: 50.
        #[arg(long, value_name = "N", default_value = "50")]
        max_tokens: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Taglist { path, frontmatter, depth }) => {
            let root = resolve_root(path);
            let tags = collect_tags(&root, frontmatter, depth);
            for tag in tags {
                println!("{}", tag);
            }
        }
        Some(Commands::Init { path }) => {
            let dir = resolve_root(path);
            cmd_init(&dir);
        }
        Some(Commands::Validate { path, depth, max_tokens }) => {
            let root = resolve_root(path);
            let ok = cmd_validate(&root, depth, max_tokens);
            if !ok {
                std::process::exit(1);
            }
        }
        None => {
            let root = resolve_root(cli.path);
            let entries = collect_entries(&root, cli.frontmatter, cli.depth);
            let iter = entries.iter().filter(|(_, entry)| {
                cli.filter.as_ref()
                    .map_or(true, |tag| entry.tags.iter().any(|t| t == tag))
            });
            let limited: Box<dyn Iterator<Item = _>> = match cli.limit {
                Some(n) => Box::new(iter.take(n)),
                None => Box::new(iter),
            };
            for (rel_path, entry) in limited {
                println!("{}: {}", rel_path.display(), truncate(&entry.description, 50));
            }
        }
    }
}

/// Truncate `s` to at most `max_tokens * 4` characters, cutting at the last
/// word boundary at or before the limit and appending `...` if truncated.
fn truncate(s: &str, max_tokens: usize) -> std::borrow::Cow<'_, str> {
    let limit = max_tokens * 4;
    if s.len() <= limit {
        return std::borrow::Cow::Borrowed(s);
    }
    // Find the last whitespace at or before the char limit
    let cut = s[..limit]
        .rfind(char::is_whitespace)
        .unwrap_or(limit);
    std::borrow::Cow::Owned(format!("{}...", s[..cut].trim_end()))
}

fn resolve_root(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| env::current_dir().expect("cannot determine current directory"))
}

fn collect_entries(
    root: &PathBuf,
    frontmatter: bool,
    depth: Option<usize>,
) -> Vec<(PathBuf, types::TldrEntry)> {
    let files = walker::find_files(root, frontmatter, depth);
    let mut results = Vec::new();

    for f in files {
        let entry = if f.is_markdown {
            parser::parse_frontmatter(&f.path)
        } else {
            parser::parse_tldr_file(&f.path)
        };

        if let Some(entry) = entry {
            // The path for a .tldr is the containing directory; for .md files it's the file itself
            let dir = if f.is_markdown {
                f.path.clone()
            } else {
                f.path.parent().map(|p| p.to_path_buf()).unwrap_or(f.path)
            };
            let rel = dir.strip_prefix(root).unwrap_or(&dir).to_path_buf();
            results.push((rel, entry));
        }
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

fn collect_tags(root: &PathBuf, frontmatter: bool, depth: Option<usize>) -> BTreeSet<String> {
    collect_entries(root, frontmatter, depth)
        .into_iter()
        .flat_map(|(_, e)| e.tags)
        .collect()
}

const TLDR_TEMPLATE: &str = r#"description = ""
tags        = []
docs        = []
"#;

fn cmd_init(dir: &PathBuf) {
    let target = dir.join(".tldr");
    if target.exists() {
        eprintln!("error: .tldr already exists at {}", target.display());
        std::process::exit(1);
    }
    std::fs::write(&target, TLDR_TEMPLATE).unwrap_or_else(|e| {
        eprintln!("error: could not write {}: {}", target.display(), e);
        std::process::exit(1);
    });
    println!("created {}", target.display());
}

fn cmd_validate(root: &PathBuf, depth: Option<usize>, max_tokens: usize) -> bool {
    let files = walker::find_files(root, false, depth);
    let mut all_ok = true;

    for f in files.iter().filter(|f| !f.is_markdown) {
        let rel = f.path.strip_prefix(root).unwrap_or(&f.path);
        let content = match std::fs::read_to_string(&f.path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}: error reading file: {}", rel.display(), e);
                all_ok = false;
                continue;
            }
        };

        // Parse as raw TOML table to catch unknown fields
        let table: Table = match toml::from_str(&content) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}: invalid TOML: {}", rel.display(), e);
                all_ok = false;
                continue;
            }
        };

        // Check for unknown fields
        for key in table.keys() {
            if !matches!(key.as_str(), "description" | "tags" | "docs") {
                eprintln!("{}: unknown field `{}`", rel.display(), key);
                all_ok = false;
            }
        }

        // Validate description
        match table.get("description") {
            None => {
                eprintln!("{}: missing required field `description`", rel.display());
                all_ok = false;
            }
            Some(toml::Value::String(s)) if s.is_empty() => {
                eprintln!("{}: `description` must not be empty", rel.display());
                all_ok = false;
            }
            Some(toml::Value::String(s)) => {
                let estimated_tokens = s.len() / 4;
                if estimated_tokens > max_tokens {
                    eprintln!(
                        "{}: `description` is ~{} tokens (max {}); keep it terse",
                        rel.display(), estimated_tokens, max_tokens
                    );
                    all_ok = false;
                }
            }
            Some(_) => {
                eprintln!("{}: `description` must be a string", rel.display());
                all_ok = false;
            }
        }

        // Validate tags (must be array of strings if present)
        if let Some(v) = table.get("tags") {
            match v {
                toml::Value::Array(arr) => {
                    if arr.iter().any(|x| !x.is_str()) {
                        eprintln!("{}: `tags` must be an array of strings", rel.display());
                        all_ok = false;
                    }
                }
                _ => {
                    eprintln!("{}: `tags` must be an array", rel.display());
                    all_ok = false;
                }
            }
        }

        // Validate docs (must be array of strings if present)
        if let Some(v) = table.get("docs") {
            match v {
                toml::Value::Array(arr) => {
                    if arr.iter().any(|x| !x.is_str()) {
                        eprintln!("{}: `docs` must be an array of strings", rel.display());
                        all_ok = false;
                    }
                }
                _ => {
                    eprintln!("{}: `docs` must be an array", rel.display());
                    all_ok = false;
                }
            }
        }
    }

    if all_ok {
        let count = files.iter().filter(|f| !f.is_markdown).count();
        println!("validated {} .tldr file{} — all ok", count, if count == 1 { "" } else { "s" });
    }

    all_ok
}

