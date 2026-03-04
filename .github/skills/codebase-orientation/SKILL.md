---
name: codebase-orientation
description: >
  Strategy for quickly mapping the structure and purpose of directories in a
  codebase using the `tldr` tool. Use this at the start of any task that
  requires understanding the layout, ownership, or feature areas of an
  unfamiliar repository, or when asked to find where something lives.
---

## Overview

`tldr` is a CLI tool that discovers `.tldr` files — small TOML files that
developers place at the root of a directory to describe its purpose. Running
`tldr` gives a terse, token-efficient map of a codebase that you can read in
full before deciding where to focus.

A `.tldr` file looks like this:

```toml
description = "Handles JWT authentication and session management"
tags        = ["auth", "security"]
docs        = ["README.md", "https://wiki.example.com/auth"]
```

## Orientation Workflow

When starting work in a repository, or when asked to locate something:

1. **Get the full map first** — run `tldr` from the repository root:
   ```
   tldr
   ```
   Read every entry before proceeding. This is fast and cheap.

2. **If the output is large**, do a shallow pass first, then go deeper:
   ```
   tldr --depth 2
   tldr --depth 4
   ```

3. **Narrow by topic** once you know what tags exist:
   ```
   tldr taglist
   tldr --filter auth
   ```

4. **For more detail** on a relevant directory, open its `.tldr` file directly
   — it may contain `docs` links (READMEs, wiki pages) worth reading.

5. **If the repository has annotated markdown files**, add `--frontmatter` to
   also pick up `tldr:` blocks in YAML frontmatter of `.md` files:
   ```
   tldr --frontmatter
   ```

## Reference

| Command | Effect |
|---------|--------|
| `tldr` | List all annotated directories with their descriptions |
| `tldr <path>` | Search from a specific root instead of cwd |
| `tldr --depth N` | Limit traversal to N directory levels deep |
| `tldr --limit N` | Stop after N entries (shallowest paths first) |
| `tldr --filter <tag>` | Only show entries tagged with `<tag>` |
| `tldr --frontmatter` | Also scan `*.md` files for YAML frontmatter |
| `tldr taglist` | Print all unique tags, sorted — use before `--filter` |
| `tldr init [path]` | Create a blank `.tldr` template in a directory |
| `tldr validate [path]` | Check `.tldr` files for correctness and token limits |

## When There Are No `.tldr` Files

If `tldr` returns no results, the repository has not been annotated yet. Fall
back to conventional exploration (README, directory listing, grep). You can
suggest running `tldr init` in key directories to bootstrap annotations.
