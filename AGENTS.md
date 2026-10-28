# AGENTS.md

## Purpose

`rudis-lite` is a deliberately small Rust clone of the Claude Code slash-command
workflow from `../rudis`. It is not a second implementation of the full Rudis CLI.

## Scope boundary

Keep only:

- bundled Claude Code slash-command templates;
- `list`, `init`, `update`, and `version`;
- safe installation into `.claude/commands/`;
- documentation and tests for that workflow.

Do not add TUI, web, marketplace/plugin management, profiles, utilities, governance
runtime, GitLab/token handling, network downloads, shell execution, or other agents.

Never modify `/home/gn/project/rusdi/rudis/` from this project.

## Clean-code rules

- Use stable Rust and the standard library first.
- Keep the binary entry point thin and put testable behavior in library modules.
- Prefer small functions with explicit inputs and results over global state.
- Do not add a dependency unless the standard library cannot reasonably solve the need.
- Do not add aliases, configuration, abstraction layers, or future-facing options without
  a current requirement.
- Keep output deterministic so tests and scripts can rely on it.
- Treat filesystem paths and file contents as untrusted input; return useful errors.
- Never overwrite or delete a file unless it carries the exact Rudis Lite ownership marker.
- Every new behavior needs a test written before its implementation.

## Required checks

Run all of these before reporting a change as ready:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Upstream template synchronization

The source of truth is:

```text
../rudis/templates/commands/
```

When `rudis` changes:

1. Inspect the upstream command templates and relevant documentation.
2. Compare them with `templates/commands/` and the explicit manifest in `src/templates.rs`.
3. Port only Claude Code slash-command changes.
4. Review command names, placeholders, generated paths, and cross-command references.
5. Update `UPSTREAM_REVISION` and `CHANGELOG.md`.
6. Run formatting, Clippy, tests, and a temporary-project smoke test.
7. Confirm no unrelated Python, TUI, web, marketplace, or agent feature was copied.

Every bundled file must retain its upstream body except for the generated provenance
header. Do not silently change command semantics while syncing.

## Ownership marker

Generated files begin with the exact marker exposed by `templates::OWNERSHIP_MARKER`.
`init` and `update` may write only missing files or files beginning with that marker.
User-created files without the marker must be preserved.
