# Rudis Lite Distribution and Self-Update Design

## Purpose

Make `rudis-lite` usable by people who do not have Rust installed. Release artifacts
will be built by GitHub Actions for Linux and Windows, published as GitHub Release
assets, and installable through documented scripts or direct downloads.

## Scope

### In scope

- GitHub Actions release workflow triggered by version tags.
- Linux x86_64 and Linux ARM64 release binaries.
- Windows x86_64 release binary.
- SHA-256 checksums for every release asset.
- Linux and Windows installation scripts.
- `rudis-lite self-update` for updating the installed binary.
- Clear separation between binary updates and project slash-command updates.

### Out of scope

- macOS builds in this phase.
- Automatic updates during `rudis-lite update`.
- Authenticated GitHub API access or private repositories.
- Package-manager integrations such as Homebrew, Scoop, Chocolatey, or APT.
- Code signing/notarization beyond what the platform build environment provides.

## User-facing behavior

`rudis-lite update` keeps its existing meaning: update managed Claude Code slash
commands in the target project.

`rudis-lite self-update` updates the installed `rudis-lite` executable. It detects the
current supported platform, checks the public latest GitHub Release, and exits without
changing anything when already current.

Supported platform identities:

| Runtime | Release asset |
|---|---|
| Linux x86_64 | `rudis-lite-linux-x86_64` |
| Linux ARM64 | `rudis-lite-linux-aarch64` |
| Windows x86_64 | `rudis-lite-windows-x86_64.exe` |

Unsupported platforms receive a clear error naming the detected platform and the
supported alternatives.

## Release workflow

GitHub Actions runs on tags matching `v*`, for example:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The workflow:

1. Checks out the tagged source.
2. Builds release binaries for the three supported targets.
3. Renames each binary to the stable asset name above.
4. Generates `SHA256SUMS` covering all binary assets.
5. Creates a GitHub Release for the tag and uploads binaries plus checksums.

The workflow must fail if any target build fails or if an expected asset is missing.
The repository is expected to be public for unauthenticated installation and update.

## Installation

`README.md` documents:

- direct GitHub Release downloads;
- a POSIX `install.sh` that detects Linux architecture, downloads the matching binary,
  verifies its checksum, and installs to a user-writable bin directory;
- a PowerShell `install.ps1` that detects Windows architecture, downloads the matching
  `.exe`, verifies its checksum, and installs to a user-writable directory;
- PATH setup instructions;
- source installation with Cargo for contributors.

Installers must not require administrator privileges by default and must not execute an
unchecked downloaded file. They download to a temporary path, verify SHA-256, then move
the validated file into place.

## Self-update flow

The self-update implementation uses the public GitHub Releases API and release assets:

1. Determine the current executable path.
2. Detect OS and architecture using Rust's compile-time target constants.
3. Fetch `releases/latest` with a descriptive User-Agent.
4. Parse the release tag and asset URLs.
5. Select the matching binary and `SHA256SUMS` asset.
6. Download both to temporary files.
7. Verify the downloaded binary's SHA-256 against `SHA256SUMS`.
8. Preserve executable permissions on Unix where applicable.
9. Replace the current executable only after verification succeeds.
10. Report the old and new versions.

If any network, parsing, checksum, permission, or replacement step fails, the old
executable remains in place. The command must never execute shell commands or invoke a
package manager.

The update operation must handle the current executable being locked on Windows by
using a platform-appropriate replacement strategy or returning an actionable error that
leaves the old executable intact. The first implementation may use a temporary helper
process only if needed and if it is bundled/controlled by the release; it must not use
an arbitrary downloaded script.

## Dependency policy

The existing CLI has no Rust runtime dependencies. Self-update needs HTTP, JSON, and
SHA-256 functionality. Use the smallest well-maintained dependencies necessary, with
default features disabled where practical. Do not add a general update framework.

The chosen HTTP client must support Linux and Windows, HTTPS certificate validation,
timeouts, and bounded response handling. Checksum verification must use a standard
cryptographic SHA-256 implementation, not a handwritten hash.

## Testing

Tests cover:

- platform/asset-name mapping for Linux x86_64, Linux ARM64, and Windows x86_64;
- rejection of unsupported targets;
- parsing a valid latest-release response;
- missing release asset and malformed checksum failures;
- checksum mismatch preserving the existing executable;
- already-current behavior;
- installer command construction and safe temporary-file behavior where practical.

Network tests use local fixtures or an injected HTTP boundary; tests must not depend on
GitHub availability.

Required checks remain:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

## Documentation and maintenance

`AGENTS.md` will add release rules:

- never publish a release without a matching tag and changelog entry;
- keep asset names and target matrix synchronized;
- verify checksums and release assets after a tagged build;
- do not make `update` perform self-update;
- update README install commands when release asset names change;
- test both installer scripts and the self-update path when changing distribution code.

`CHANGELOG.md` records the first release containing distribution and self-update
support.
