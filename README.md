# tldr

A fast Rust CLI tool for AI-assisted codebase navigation. Place a `.tldr` file in any directory to describe what it contains — `tldr` finds them all and prints a terse map to stdout in one shot.

The intent is to give an AI agent a token-efficient answer to "what is this codebase and where does everything live?" before it starts digging through files.

## How it works

Drop a `.tldr` file at the root of any directory you want to annotate:

```toml
description = "Handles JWT authentication and session management"
tags        = ["auth", "security"]
docs        = ["README.md", "https://wiki.example.com/auth"]
```

Run `tldr` and get one line per annotated directory:

```
src/auth:    Handles JWT authentication and session management
src/db:      Database models and connection pooling
src/api:     REST endpoints and request routing
```

Descriptions longer than ~50 tokens are truncated at a word boundary with `...` — keeping output scannable even in large codebases.

## Installation

```sh
cargo install --path .
```

## `.tldr` file format

`.tldr` files are [TOML](https://toml.io). Place one file named exactly `.tldr` at the root of the directory it describes.

| Field | Required | Type | Description |
|-------|----------|------|-------------|
| `description` | ✅ | string | Terse summary of the directory's purpose. Keep it under ~50 tokens. |
| `tags` | ❌ | string array | Topics or domain labels. Used with `--filter` and `taglist`. |
| `docs` | ❌ | string array | Links to relevant documentation — file paths or URLs. |

```toml
description = "Processes and validates incoming webhook payloads"
tags        = ["webhooks", "ingestion"]
docs        = ["docs/webhooks.md", "https://wiki.example.com/webhooks"]
```

## Usage

### List all annotated directories

```sh
tldr              # search from current directory
tldr ./src        # search from a specific path
```

### Incremental exploration

For large codebases, start shallow and go deeper:

```sh
tldr --depth 2    # only look 2 directory levels deep
tldr --depth 4    # go a bit deeper
tldr --limit 20   # stop after 20 entries (shallowest first)
```

### Filter by tag

```sh
tldr taglist              # see all available tags first
tldr --filter auth        # only show entries tagged "auth"
```

### Markdown frontmatter mode

If your repo already has `tldr:` blocks in markdown frontmatter, use `--frontmatter` to pick those up too:

```sh
tldr --frontmatter
```

Expected frontmatter shape:

```markdown
---
tldr:
  description: "Short summary"
  tags: ["auth"]
  docs: ["README.md"]
---
```

### Create a new `.tldr` file

```sh
tldr init            # creates .tldr in the current directory
tldr init ./src/api  # creates .tldr in ./src/api
```

Exits with an error if a `.tldr` already exists.

### Validate `.tldr` files

```sh
tldr validate        # check all .tldr files under cwd
tldr validate ./src  # check a specific subtree
```

Checks for:
- `description` is present and non-empty
- `description` is within the token limit (default: 50 tokens, estimated as chars ÷ 4)
- `tags` and `docs`, if present, are arrays of strings
- No unknown fields

Exits non-zero on failure — suitable for CI.

```sh
tldr validate --max-tokens 30   # stricter token limit
```

## Command reference

| Command | Description |
|---------|-------------|
| `tldr [PATH]` | List all annotated directories |
| `tldr --depth N` | Limit traversal depth |
| `tldr --limit N` | Cap number of results |
| `tldr --filter TAG` | Filter by tag |
| `tldr --frontmatter` | Also scan `*.md` YAML frontmatter |
| `tldr taglist [PATH]` | List all unique tags |
| `tldr init [PATH]` | Create a blank `.tldr` template |
| `tldr validate [PATH]` | Validate `.tldr` files |
| `tldr validate --max-tokens N` | Override token limit |

## Copilot skill

This repo includes a [Copilot CLI agent skill](.github/skills/codebase-orientation/SKILL.md) that teaches Copilot to run `tldr` at the start of any codebase exploration task. Once installed, Copilot will automatically use it when asked to find where something lives or understand a repository's structure.

## License

MIT
