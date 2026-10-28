# Rudis Lite Distribution Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Publish Linux and Windows binaries, provide verified installers, and add a safe `rudis-lite self-update` command without changing the meaning of `rudis-lite update`.

**Architecture:** Add a small update client behind a testable HTTP boundary, using release metadata and SHA-256 assets from the public GitHub repository. Keep platform mapping, release parsing, checksum verification, and executable replacement separate; use GitHub Actions only for tagged builds and release publication.

**Tech Stack:** Rust stable, `ureq` for HTTPS/HTTP, `serde` + `serde_json` for release JSON, `sha2` for SHA-256, GitHub Actions, POSIX shell, PowerShell.

**Spec:** `docs/superpowers/specs/2026-10-28-rudis-lite-distribution-design.md`

## Global Constraints

- Support Linux x86_64, Linux ARM64, and Windows x86_64.
- Release assets must be `rudis-lite-linux-x86_64`, `rudis-lite-linux-aarch64`, and `rudis-lite-windows-x86_64.exe`.
- Publish `SHA256SUMS` with every release.
- `rudis-lite update` remains project slash-command update; binary update is only `rudis-lite self-update`.
- Never replace the current executable before checksum verification succeeds.
- Never execute shell commands or package managers from the Rust self-update path.
- Runtime downloads use HTTPS, timeouts, bounded responses, and certificate validation.
- Tests must not depend on GitHub availability.
- Required checks: `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test`.

## Review Focus

- A release with one missing asset must fail the workflow and self-update must return a clear error; test in Tasks 3 and 4.
- A malformed or mismatched checksum must leave the installed executable unchanged; test in Task 4.
- An unsupported host/architecture must fail before any network or filesystem replacement; test in Task 2.
- A locked Windows executable must not be truncated or deleted when replacement fails; test the replacement boundary in Task 4.
- Installer scripts must not install an unverified download or require administrator privileges by default; inspect/script-test in Task 5.

---

### Task 1: Add update dependencies and platform/release data types

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Create: `src/platform.rs`
- Create: `src/release.rs`
- Modify: `src/lib.rs`
- Test: `tests/platform_release.rs`

**Interfaces:**
- `platform::Target { asset_name: &'static str, executable_suffix: &'static str }`.
- `platform::current() -> Result<Target, UpdateError>`.
- `platform::for_target(os: &str, arch: &str) -> Result<Target, UpdateError>`.
- `release::Release { tag_name: String, assets: Vec<Asset> }`.
- `release::Asset { name: String, browser_download_url: String }`.
- `release::Release::asset(&self, name: &str) -> Result<&Asset, UpdateError>`.

- [ ] **Step 1: Add failing platform/release tests**

  Assert the three exact mappings, unsupported target rejection, valid release JSON parsing, and missing asset errors.

- [ ] **Step 2: Run the focused tests to verify RED**

  Run: `cargo test --test platform_release`

  Expected: FAIL because the modules and dependency types do not exist.

- [ ] **Step 3: Add minimal dependencies**

  Add `ureq` with JSON support disabled unless needed by the chosen implementation, `serde` with derive, `serde_json`, and `sha2`. Keep default features minimal while retaining system TLS validation on Linux and Windows.

- [ ] **Step 4: Implement platform mapping and release parsing**

  Use compile-time `cfg!(target_os)`/`cfg!(target_arch)` for the running target and serde structs for only the GitHub fields needed.

- [ ] **Step 5: Run focused tests**

  Run: `cargo test --test platform_release`

  Expected: PASS.

### Task 2: Implement checksum and verified download primitives

**Files:**
- Create: `src/download.rs`
- Modify: `src/lib.rs`
- Test: `tests/download.rs`

**Interfaces:**
- `download::parse_checksums(text: &str) -> Result<HashMap<String, String>, UpdateError>`.
- `download::verify_sha256(bytes: &[u8], expected_hex: &str) -> Result<(), UpdateError>`.
- `download::download_verified(client: &impl HttpClient, binary_url: &str, checksum_url: &str, asset_name: &str) -> Result<Vec<u8>, UpdateError>`.
- `download::HttpClient` exposes bounded byte/text GET operations so tests can use an in-memory fake instead of GitHub.

- [ ] **Step 1: Write failing checksum tests**

  Cover valid checksum lines, malformed lines, missing asset checksum, valid bytes, mismatch, and fake HTTP responses.

- [ ] **Step 2: Run focused tests to verify RED**

  Run: `cargo test --test download`

  Expected: FAIL because checksum/download interfaces do not exist.

- [ ] **Step 3: Implement checksum parsing and SHA-256 verification**

  Accept standard `hash  filename` and `hash *filename` forms; normalize hex case; reject malformed or duplicate entries rather than guessing.

- [ ] **Step 4: Implement bounded verified download**

  Download checksum text and binary through the injected boundary, require the named checksum, verify before returning bytes, and never write a target executable in this module.

- [ ] **Step 5: Run focused tests**

  Run: `cargo test --test download`

  Expected: PASS.

### Task 3: Add GitHub latest-release client

**Files:**
- Create: `src/github.rs`
- Modify: `src/lib.rs`
- Test: `tests/github.rs`

**Interfaces:**
- `github::latest_release(client: &impl HttpClient, owner: &str, repo: &str) -> Result<release::Release, UpdateError>`.
- `github::release_assets(release: &Release, target: Target) -> Result<(&Asset, &Asset), UpdateError>` returns binary and `SHA256SUMS` assets.

- [ ] **Step 1: Write failing GitHub fixture tests**

  Use a fake HTTP client to assert the exact public latest-release path, User-Agent requirement, valid response parsing, missing binary asset, and missing checksum asset behavior.

- [ ] **Step 2: Run focused tests to verify RED**

  Run: `cargo test --test github`

  Expected: FAIL because the GitHub client does not exist.

- [ ] **Step 3: Implement GitHub API request**

  Request `https://api.github.com/repos/{owner}/{repo}/releases/latest` with a descriptive User-Agent and a finite timeout; map non-success responses to actionable errors.

- [ ] **Step 4: Implement asset selection**

  Select the platform binary and exact `SHA256SUMS` name. Do not fall back to another architecture.

- [ ] **Step 5: Run focused tests**

  Run: `cargo test --test github`

  Expected: PASS.

### Task 4: Add safe `self-update` command and executable replacement

**Files:**
- Create: `src/self_update.rs`
- Modify: `src/cli.rs`
- Modify: `src/main.rs`
- Modify: `src/lib.rs`
- Test: `tests/self_update.rs`

**Interfaces:**
- `self_update::run(repository: &str) -> Result<UpdateResult, UpdateError>`.
- `self_update::UpdateResult { old_version: String, new_version: String, changed: bool }`.
- `self_update::replace_current(executable: &Path, replacement: &[u8]) -> Result<(), UpdateError>`.

- [ ] **Step 1: Extend CLI tests first**

  Assert `self-update` parses distinctly from `update`, accepts no project target, and rejects extra arguments.

- [ ] **Step 2: Write failing self-update tests**

  Test already-current behavior, successful fixture update, checksum mismatch preserving the old file, and missing assets preserving the old file. Inject release/download boundaries rather than contacting GitHub.

- [ ] **Step 3: Run focused tests to verify RED**

  Run: `cargo test --test self_update`

  Expected: FAIL because command and update implementation do not exist.

- [ ] **Step 4: Implement current-version and release comparison**

  Compare the binary package version with the release tag after stripping a leading `v`; do not downgrade unless a future explicit flag is added.

- [ ] **Step 5: Implement safe replacement**

  Resolve the current executable, write verified bytes to a sibling temporary path, preserve Unix executable mode, rename only after success, and remove temporary files on errors. On Windows, return a clear locked-file error if replacement cannot be performed without risking the old executable.

- [ ] **Step 6: Wire `self-update` into dispatch**

  Keep `update` unchanged. Print old/new versions and whether the binary was already current.

- [ ] **Step 7: Run focused and full tests**

  Run: `cargo test --test self_update` then `cargo test`.

  Expected: all tests PASS.

### Task 5: Add Linux/Windows installers and tagged release workflow

**Files:**
- Create: `install.sh`
- Create: `install.ps1`
- Create: `.github/workflows/release.yml`
- Modify: `README.md`
- Modify: `AGENTS.md`
- Modify: `CHANGELOG.md`
- Test/inspect: `tests/installer_scripts.sh` if shell tooling is available

**Interfaces:**
- Release workflow emits exactly the three binary assets plus `SHA256SUMS`.
- `install.sh` supports Linux x86_64/aarch64 and verifies checksum before installation.
- `install.ps1` supports Windows x86_64 and verifies checksum before installation.

- [ ] **Step 1: Write installer/release assertions**

  Add a shell check that installer scripts contain checksum verification, stable asset names, temporary downloads, and no unchecked execution. Add a YAML/static check for the three target triples, tag trigger, checksum generation, and release upload.

- [ ] **Step 2: Implement `install.sh`**

  Detect `uname -s`/`uname -m`, map only supported Linux targets, download release assets and `SHA256SUMS`, verify with `sha256sum` or `shasum`, install to `${XDG_BIN_HOME:-$HOME/.local/bin}`, and preserve the old binary if validation fails.

- [ ] **Step 3: Implement `install.ps1`**

  Require Windows x64, download the `.exe` and checksums, use `Get-FileHash`, install to a user-writable directory, and avoid replacing an existing binary until verification succeeds.

- [ ] **Step 4: Implement GitHub Actions release workflow**

  Build `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`; use a maintained Rust toolchain action, package stable asset names, compute SHA-256, and create a release on `v*` tags. Fail if expected artifacts are absent.

- [ ] **Step 5: Update README/AGENTS/changelog**

  Document release downloads, both installers, PATH setup, source installation, `self-update`, the distinction from `update`, and release maintenance rules.

- [ ] **Step 6: Run script/static checks and full Rust checks**

  Run the installer script check plus:

  ```bash
  cargo fmt --check
  cargo clippy --all-targets --all-features -- -D warnings
  cargo test
  ```

  Expected: all available checks PASS. Cross-platform builds are verified by GitHub Actions after pushing the workflow.

## Final Review Checklist

- [ ] `rudis-lite update` still only updates project slash commands.
- [ ] `rudis-lite self-update` uses verified release assets and preserves the old binary on failure.
- [ ] Linux x86_64, Linux ARM64, and Windows x86_64 assets are built and published.
- [ ] `SHA256SUMS` is published and used by both installers and self-update.
- [ ] README installation instructions require no Rust for end users.
- [ ] No GitHub network is required by tests.
- [ ] Rust quality gates pass.
