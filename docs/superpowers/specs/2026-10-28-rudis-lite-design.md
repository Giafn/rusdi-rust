# Rudis Lite Design

## Purpose

`rudis-lite` is a small Rust CLI that reproduces only the Claude Code slash-command
generation workflow from the upstream `rudis` project. The upstream repository at
`../rudis` remains unchanged and is the reference source for future template updates.

## Scope

### In scope

- Bundle Claude Code command templates in the Rust executable.
- List available commands.
- Install commands into a project's `.claude/commands/` directory.
- Update only files previously managed by `rudis-lite`.
- Support `--dry-run` for update previews.
- Provide version information.
- Document installation, use, uninstall, and upstream synchronization.

### Out of scope

- TUI, web UI, marketplace/plugin management, profiles, utilities, governance runtime,
  GitLab API/token handling, network downloads, and non-Claude agents.
- Reimplementing the Python CLI or its unrelated business logic.
- Overwriting user-owned command files.

## CLI

```text
rudis-lite list
rudis-lite init [--here] [PROJECT_DIR]
rudis-lite update [--here] [PROJECT_DIR] [--dry-run]
rudis-lite version
```

The default project target is the current directory. `init` creates
`.claude/commands/` and writes the bundled command files. `update` changes only files
containing the Rudis Lite ownership marker; unrelated files are preserved.

## Template ownership and provenance

Every generated file starts with a machine-detectable marker containing the upstream
template path and the upstream version/commit recorded by the clone. The marker is the
only authority used to decide whether `update` may replace a file. Files without the
marker are never overwritten or deleted.

Templates are copied from `../rudis/templates/commands/` during a deliberate upstream
sync. Only Claude-compatible slash-command templates are included. The selected list is
kept in a small manifest so additions/removals are reviewable.

## Rust structure

```text
rudis-lite/
├── Cargo.toml
├── AGENTS.md
├── README.md
├── CHANGELOG.md
├── src/
│   ├── main.rs
│   ├── cli.rs
│   ├── installer.rs
│   └── templates.rs
├── templates/commands/
└── tests/
```

Use the Rust standard library first. Dependencies should be avoided unless they reduce
meaningful code or are required for a stable CLI interface. The initial implementation
may use a single small CLI dependency only if it materially improves argument parsing;
otherwise parse the small command surface with `std::env::args`.

## Safety and errors

- Reject a target that is not a directory.
- Create missing `.claude/commands/` directories.
- Refuse to overwrite non-owned files and report each skipped path.
- Write files through a temporary sibling and rename where practical, so an interrupted
  update does not leave a truncated command.
- Return non-zero exit status on invalid arguments, invalid target paths, or write errors.
- Never execute shell commands, invoke Claude, access the network, or modify the
  upstream `rudis` checkout.

## Testing

Tests cover command listing, target resolution, first installation, preservation of
user-owned files, update dry-run behavior, ownership detection, and invalid targets.
The verification baseline is:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Documentation

`README.md` must include installation from a release binary and with `cargo install
--path .`, Claude Code setup, examples, command inventory, update/uninstall behavior,
limitations versus upstream `rudis`, and the upstream sync procedure.

`AGENTS.md` must define clean-code rules, the scope boundary, required checks, template
provenance rules, and the required review steps when upstream `rudis` changes.

## Upstream update workflow

1. Inspect upstream `templates/commands/` and relevant command documentation.
2. Compare the manifest and template contents against the pinned upstream version.
3. Port only Claude Code slash-command changes.
4. Review placeholder, command-name, and cross-command workflow changes.
5. Update provenance and `CHANGELOG.md`.
6. Run formatting, Clippy, tests, and a manual install into a temporary project.
7. Confirm no unrelated upstream feature or dependency was copied.
