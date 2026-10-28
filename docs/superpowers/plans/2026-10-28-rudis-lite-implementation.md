# Rudis Lite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a dependency-light Rust CLI that installs and updates the Claude Code slash-command templates from `../rudis` without modifying the upstream checkout.

**Architecture:** Use a small standard-library Rust binary with a fixed command parser, a manifest-backed embedded template catalog, and an installer module that owns only files carrying the Rudis Lite marker. Templates are copied deliberately from the upstream `rudis/templates/commands/` directory and embedded with `include_str!`; runtime operations never use the network or execute shell commands.

**Tech Stack:** Rust stable, Cargo, Rust standard library, `include_str!`, integration tests with temporary directories. No runtime dependencies unless a later reviewed requirement makes one necessary.

**Spec:** `docs/superpowers/specs/2026-10-28-rudis-lite-design.md`

## Global Constraints

- Support only Claude Code slash commands.
- Do not modify `/home/gn/project/rusdi/rudis/`.
- Do not implement TUI, web, marketplace, profiles, utilities, governance runtime, GitLab access, token handling, or other agents.
- Never overwrite or delete files without the Rudis Lite ownership marker.
- Use Rust standard library first and avoid dependencies.
- Runtime must not execute shell commands, invoke Claude, access the network, or modify the upstream checkout.
- Required checks: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.
- Every upstream template sync records the source path and upstream revision/version.

## Review Focus

- A user-created `.claude/commands/rudis.*.md` without the marker must survive `init` and `update`; test in Task 4.
- A previously managed file with changed user content must not silently lose that content; test the chosen update policy in Task 4.
- A target path that is missing or a file must produce a non-zero result without creating files outside the target; test in Task 2 and Task 4.
- Re-running `init` must be idempotent and must not duplicate markers; test in Task 4.
- Unknown commands, extra arguments, and invalid flag combinations must fail with concise usage output; test in Task 2.

---

### Task 1: Create the Rust project contract and repository guidance

**Files:**
- Create: `Cargo.toml`
- Create: `AGENTS.md`
- Create: `README.md`
- Create: `CHANGELOG.md`
- Create: `.gitignore`
- Create: `src/main.rs`

**Interfaces:**
- Produces the package name `rudis-lite`, binary name `rudis-lite`, and version constant used by later CLI code.
- Documents the supported workflow and prevents future work from expanding into an upstream CLI rewrite.

- [ ] **Step 1: Add the minimal Cargo package metadata**

  Define package `rudis-lite`, version `0.1.0`, Rust 2021 edition, and a binary target named `rudis-lite`. Keep `[dependencies]` empty.

- [ ] **Step 2: Write `AGENTS.md`**

  Document clean-code rules, no-dependency default, required checks, ownership-marker rules, the exact upstream source path, the template-sync review checklist, and explicit out-of-scope features.

- [ ] **Step 3: Write the README installation contract**

  Include release-binary installation, `cargo install --path .`, Claude Code usage, `init`, `update`, `list`, `version`, uninstall instructions, supported command inventory, limitations versus `rudis`, and the upstream sync procedure.

- [ ] **Step 4: Add initial changelog and ignore generated Rust files**

  Record `0.1.0` as the initial slim Claude Code-only release and ignore `target/`.

- [ ] **Step 5: Add a compiling entry point**

  `src/main.rs` should expose `fn main()` and call a temporary `run() -> Result<(), String>` path that exits non-zero on error. Keep the entry point small so CLI parsing can be tested independently in Task 2.

- [ ] **Step 6: Verify the project skeleton**

  Run: `cargo check`

  Expected: the package compiles with no external dependencies.

### Task 2: Implement argument parsing and command dispatch

**Files:**
- Create: `src/cli.rs`
- Modify: `src/main.rs`
- Test: `tests/cli.rs`

**Interfaces:**
- `cli::Command`: `List`, `Init { target: PathBuf }`, `Update { target: PathBuf, dry_run: bool }`, `Version`.
- `cli::parse_args<I, S>(args: I) -> Result<Command, String>` where `I: IntoIterator<Item = S>` and `S: Into<OsString> + Clone`.
- `cli::default_target() -> io::Result<PathBuf>` resolves the current directory.
- `main.rs` dispatches parsed commands to installer/template functions added later.

- [ ] **Step 1: Write parser tests first**

  Cover `list`, `version`, `init` with an explicit directory, `init --here`, default target behavior, `update --dry-run`, unknown commands, unknown flags, missing values, and extra positional arguments.

- [ ] **Step 2: Run the focused tests and verify they fail**

  Run: `cargo test --test cli`

  Expected: FAIL because the parser and command types do not yet exist.

- [ ] **Step 3: Implement the parser with `std::env::args_os`**

  Accept the exact command surface from the spec. Treat `--here` as the current directory and reject conflicting explicit target plus `--here` input. Do not add aliases or subcommands not in the spec.

- [ ] **Step 4: Wire `main.rs` to parser dispatch**

  Convert parser errors to stderr and exit code `2`; leave command execution behind small functions so the code remains testable.

- [ ] **Step 5: Run the focused tests**

  Run: `cargo test --test cli`

  Expected: PASS.

### Task 3: Add the upstream template manifest and embedded catalog

**Files:**
- Create: `templates/commands/` selected upstream command files
- Create: `src/templates.rs`
- Test: `tests/templates.rs`
- Modify: `README.md`
- Modify: `CHANGELOG.md`

**Interfaces:**
- `templates::Template { name: &'static str, source: &'static str, content: &'static str }`.
- `templates::all() -> &'static [Template]` returns deterministic command order.
- `templates::find(name: &str) -> Option<&'static Template>`.
- `templates::OWNERSHIP_MARKER` is the exact marker prefix used by the installer.
- `templates::UPSTREAM_REVISION` records the upstream git revision used for the initial copy.

- [ ] **Step 1: Define the included command set**

  Copy the current Claude-compatible files from `/home/gn/project/rusdi/rudis/templates/commands/` into `rudis-lite/templates/commands/`. Include the complete slash-command template set currently present: `analyze`, `brd`, `checklist`, `clarify`, `constitution`, `converge`, `deploy`, `fan`, `implement`, `operate`, `plan`, `postmortem`, `prototype`, `redesign`, `revamp`, `rudis`, `specify`, `taskstoissues`, `tasks`, `test`, `tracer`, `ultimate`, and `upgrade`.

  Do not copy Python source, scripts, agent-specific variants, or unrelated templates.

- [ ] **Step 2: Add manifest tests before implementation**

  Assert the catalog is non-empty, names are unique and sorted, every expected source path is represented, every embedded content is non-empty, and the recorded revision is not a placeholder.

- [ ] **Step 3: Run the focused tests and verify they fail**

  Run: `cargo test --test templates`

  Expected: FAIL until the manifest module exists.

- [ ] **Step 4: Implement the catalog with `include_str!`**

  Keep the manifest explicit rather than filesystem-discovering at runtime. Prefix generated content with the exact ownership/provenance block while retaining the upstream body unchanged.

- [ ] **Step 5: Record provenance**

  Set `UPSTREAM_REVISION` to the inspected upstream commit `e216888ea226cdb99e27be11c1f963e12eb79c4d` and document that sync in the changelog.

- [ ] **Step 6: Run the focused tests**

  Run: `cargo test --test templates`

  Expected: PASS.

### Task 4: Implement safe installation and update behavior

**Files:**
- Create: `src/installer.rs`
- Modify: `src/main.rs`
- Test: `tests/installer.rs`
- Modify: `src/templates.rs`

**Interfaces:**
- `installer::init(target: &Path) -> Result<Report, Error>`.
- `installer::update(target: &Path, dry_run: bool) -> Result<Report, Error>`.
- `installer::Report { written: Vec<PathBuf>, skipped: Vec<PathBuf>, unchanged: Vec<PathBuf> }`.
- `installer::Error` reports invalid target, directory creation, read, write, and rename failures.
- `installer::is_owned(content: &str) -> bool` checks only the exact ownership marker.

- [ ] **Step 1: Write installer tests first**

  Test first install creates `.claude/commands/` and all managed files; second install is idempotent; unowned existing files are skipped and unchanged; owned files are updated; `--dry-run` writes nothing; missing target and file target fail; temporary-file cleanup leaves no partial output after a write failure where the platform allows simulation.

- [ ] **Step 2: Run the focused tests and verify they fail**

  Run: `cargo test --test installer`

  Expected: FAIL because installer interfaces do not exist.

- [ ] **Step 3: Implement target validation and command directory creation**

  Require an existing directory target for `init`/`update`; create only `<target>/.claude/commands` beneath it. Never canonicalize through a symlink into an unrelated destination without preserving the user-selected target semantics.

- [ ] **Step 4: Implement ownership-aware writes**

  `init` writes missing files and skips unowned files. Owned files may be replaced by the bundled template. Use a unique sibling temporary file, write all bytes, flush/close it, then rename into place; remove the temporary file on failure.

- [ ] **Step 5: Implement update and dry-run reporting**

  `update` considers only existing files whose content begins with the ownership marker. Dry-run returns the same report without writes. Use deterministic report ordering and return a non-zero error for filesystem failures.

- [ ] **Step 6: Wire command dispatch and human-readable reports**

  `main.rs` calls `init`/`update`, prints created/updated/skipped paths, and returns non-zero status for errors. `list` prints template command names; `version` prints package version and upstream revision.

- [ ] **Step 7: Run the focused tests**

  Run: `cargo test --test installer`

  Expected: PASS.

### Task 5: Add end-to-end documentation and quality gates

**Files:**
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `CHANGELOG.md`
- Create: `tests/README.md` only if test execution instructions need a dedicated reference

**Interfaces:**
- User-facing install and update workflow is documented and matches the actual CLI.
- Contributor-facing upstream sync checklist is complete and points to the exact template manifest and revision.

- [ ] **Step 1: Add documentation checks to the review pass**

  Manually verify every command listed in README exists in the manifest and every example uses the real binary syntax. Verify the README explains that Claude Code must be restarted/reloaded as applicable after command changes.

- [ ] **Step 2: Document uninstall precisely**

  State that uninstalling the binary does not delete project command files; users may delete only files carrying the Rudis Lite marker after reviewing them.

- [ ] **Step 3: Document upstream synchronization**

  Explain how to compare `../rudis/templates/commands/`, review command dependencies/placeholders, update the manifest and revision, run tests, and avoid importing unrelated upstream features.

- [ ] **Step 4: Run the complete verification suite**

  Run:

  ```bash
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test
  ```

  Expected: all commands succeed with no warnings treated as errors.

- [ ] **Step 5: Perform a manual smoke test**

  Run `cargo run -- list`, `cargo run -- version`, and `cargo run -- init <temporary-project>`, inspect `.claude/commands/`, add an unowned command, run `update --dry-run`, and confirm the unowned file remains unchanged.

## Final Review Checklist

- [ ] Only `rudis-lite/` changed; upstream `rudis/` is untouched.
- [ ] All bundled files are Claude Code slash-command templates from the selected upstream revision.
- [ ] No network, shell execution, GitLab, TUI, web, marketplace, profile, utility, governance, or other-agent code was added.
- [ ] User-owned files are never overwritten or deleted.
- [ ] README contains a working Claude Code installation path.
- [ ] AGENTS.md defines clean-code and upstream-sync rules.
- [ ] `cargo fmt --check`, Clippy, tests, and manual smoke test pass.
