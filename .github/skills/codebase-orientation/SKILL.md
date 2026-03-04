---
name: codebase-orientation
description: >
  Strategy for mapping a codebase using .tldr files and the `tldr` CLI tool.
  Use this whenever .tldr files are present, when asked to understand the
  structure or layout of a repository, when asked to find where something lives,
  or at the start of any task in an unfamiliar codebase.
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

### Description length

The `description` field has a **hard limit of 50 tokens** (estimated as
characters ÷ 4, roughly 200 characters). This is intentional — descriptions
must be terse enough for an AI to scan the entire codebase map in one read.

**Good** — specific, scannable, under the limit:
```
"Handles JWT authentication and session management"
```

**Bad** — too verbose, will fail validation:
```
"This directory contains all of the code related to how users authenticate
themselves using JSON Web Tokens, including the logic for creating, signing,
validating, and refreshing tokens as well as managing user sessions."
```

When writing a description, aim for a single clause that answers:
_"what does this directory own?"_ — not a full sentence explaining how it works.

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

## Creating `.tldr` Files

When asked to create or add `.tldr` files:

1. Use `tldr init` to create the template — never write the file manually:
   ```
   tldr init ./src/auth
   ```

2. Fill in the `description` field. Keep it under 50 tokens (≈200 characters).
   Use the single-clause rule: _"what does this directory own?"_

3. Add relevant `tags` (lowercase, reuse existing tags where possible — run
   `tldr taglist` first to see what's already in use).

4. Add `docs` links if there is a README or wiki page for this directory.

5. **Always validate after creating or editing** `.tldr` files:
   ```
   tldr validate
   ```
   Fix any errors before finishing. A non-zero exit means something is wrong.

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
