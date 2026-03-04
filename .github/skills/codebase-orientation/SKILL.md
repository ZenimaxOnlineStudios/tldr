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

**Write the description in one attempt. Do not revise after writing.**
Descriptions that need revision are usually too long or too vague to begin with.

#### The single-clause rule

A description is one clause that answers: **"what does this directory own?"**

- State the *what*, not the *how* or *why*
- Name the domain or data type, not the implementation detail
- Omit phrases like "contains code for", "handles", "responsible for", "this directory" — they add no information
- Omit motivation ("Foundation for...", "Used by...", "Designed to...") — that's documentation, not a label

#### Good vs bad

| ✅ Good | ❌ Bad — why |
|--------|-------------|
| `JWT authentication and session management` | `This directory contains all the code related to how users authenticate` — verbose, circular |
| `ESO definition data reader from binary .dat files` | `Rust library for reading and serving ESO game definition data from binary .dat files. Foundation for tooling that needs to query or export def data` — two sentences, implementation trivia |
| `REST API route definitions and middleware` | `Handles incoming HTTP requests and routes them to the correct handler functions` — describes mechanism, not ownership |
| `Database schema migrations` | `SQL migration files for updating the database schema over time` — "over time" is implied; "SQL migration files" is redundant |

#### Tone

Descriptions are labels, not reviews. Keep them strictly factual.

- Do not include subjective judgements: "legacy", "messy", "poorly organised", "needs refactoring", "technical debt"
- Do not editorialize: "unfortunately", "oddly", "for historical reasons"
- Do not include warnings or opinions about quality — those belong in comments or issues, not in a navigation label

If a directory is genuinely confusing or poorly bounded, describe what it *actually contains*, not what you think of it.

| ✅ Factual | ❌ Opinion |
|-----------|-----------|
| `Vendor-copied utilities and legacy helpers` | `Messy grab-bag of old utilities that should be cleaned up` |
| `Authentication — mix of OAuth and legacy session code` | `Legacy auth code that nobody wants to touch` |

#### Formula

```
<noun phrase describing what this owns>
```

Examples:
- `Payment processing and Stripe webhook handling`
- `Build scripts and CI pipeline configuration`
- `User-facing React components for the checkout flow`

## Orientation Workflow

When starting work in a repository, or when asked to locate something:

1. **Get the full map first** — run `tldr --plain` from the repository root:
   ```
   tldr --plain
   ```
   Read every entry before proceeding. This is fast and cheap.

2. **If the output is large**, do a shallow pass first, then go deeper:
   ```
   tldr --plain --depth 2
   tldr --plain --depth 4
   ```

3. **Narrow by topic** once you know what tags exist:
   ```
   tldr taglist
   tldr --plain --filter auth
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

## Output Format

By default `tldr` emits coloured output with terminal-width word-wrapping.
**Always pass `--plain` when running `tldr` programmatically** — this disables
ANSI colour codes and produces clean, unambiguous output for parsing.

Each entry is one logical record, formatted as:

```
<path>: <description>
```

With `--plain` the output is always one line per entry, regardless of terminal
width. Without `--plain` long descriptions wrap onto continuation lines indented
to align with the description start — do not mistake these for new entries.

## Reference

| Command | Effect |
|---------|--------|
| `tldr --plain` | **Disable colour and wrapping — use this for programmatic consumption** |
| `tldr` | List all annotated directories with their descriptions |
| `tldr <path>` | Search from a specific root instead of cwd |
| `tldr --depth N` | Limit traversal to N directory levels deep |
| `tldr --limit N` | Stop after N entries (shallowest paths first) |
| `tldr --filter <tag>` | Only show entries tagged with `<tag>` |
| `tldr --frontmatter` | Also scan `*.md` files for YAML frontmatter |
| `tldr taglist` | Print all unique tags, sorted — use before `--filter` |
| `tldr init [path]` | Create a blank `.tldr` template in a directory |
| `tldr validate [path]` | Check `.tldr` files for correctness and token limits |
| `tldr validate --max-tokens N` | Override the token limit (default: 50) |

## When There Are No `.tldr` Files

If `tldr` returns no results, the repository has not been annotated yet. Fall
back to conventional exploration (README, directory listing, grep). You can
suggest running `tldr init` in key directories to bootstrap annotations.
