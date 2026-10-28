---
description: Upgrade a dependency, framework, or language runtime to a target version as a controlled, staged migration — read the real changelog/breaking changes, plan incremental stages, apply each with codemods + hand-fixes, and verify (build/tests/security) with per-stage checkpoints and rollback.
scripts:
  sh: scripts/bash/check-prerequisites.sh --json --paths-only
  ps: scripts/powershell/check-prerequisites.ps1 -Json -PathsOnly
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty). The input names the
upgrade target and (optionally) the versions — e.g. "upgrade React 17 → 18", "bump Spring Boot
to the latest 3.x", "move the project off Node 18 (EOL) to 22", or "update all outdated
dependencies with known CVEs". If no target is given, detect outdated/EOL items (Step 2) and
propose a prioritized list before doing anything.

## Interactive Clarification (ask when target or strategy is unclear)

- **Ask when** the target is ambiguous (which package, which major version), a major bump has
  more than one viable target, or the migration would cross multiple majors at once (e.g. v3 →
  v6) where an intermediate path is safer.
- **How**: pause and ask ONE specific question at a time — recommended option first with a
  one-line reason, then 2–4 concrete choices plus a free-form alternative; "yes" accepts the
  recommendation (same style as `/rudis.clarify`).
- **Bounded (no wait-loop)**: at most twice per decision. If still unresolved, do NOT guess a
  major-version migration blindly — report what is ambiguous and stop.
- **Non-interactive sessions** (CI, `rudis run`, headless): never block — for a safe, single,
  minor/patch bump proceed with the documented default and flag assumptions; for an ambiguous
  or major migration, stop and report rather than risk a half-applied upgrade.

## Goal

`/rudis.redesign` reworks a feature; this command **migrates a version**. It exists because a
tech upgrade done as a blind "bump the number and fix what breaks" wastes time and ships
regressions: the breaking changes must be read from the **real** changelog (not recalled from
memory — a knowledge cutoff misses anything released after it), the migration must be **staged**
so each step is testable, and every stage needs a **regression safety net** and a **rollback
path**. This command adds that discipline on top of the existing building blocks (`test` for the
safety net, `redesign`/`implement`'s mimicry + verification for the edits).

## Operating Constraints (STRICT)

- **Never a blind bump.** Do not raise a version without first reading its real breaking changes
  and finding which ones this codebase actually hits. Do not bump every dependency at once.
- **One major at a time.** Major-version migrations go one major step per stage (v3 → v4 → v5),
  never skipping intermediates when the project's usage depends on removed/changed APIs.
- **Regression net first.** A migration without tests is high-risk — if the touched areas have
  no test coverage, say so and recommend `/rudis.test` (including its UI/API layers) *before*
  upgrading, so a green baseline exists to catch regressions.
- **Real sources, never memory.** Breaking changes, deprecations, EOL dates, and CVEs are live
  facts — read them from the changelog/release notes/migration guide/advisory, don't recall them.
- **Security must not regress.** The target version must not introduce a known Critical/High CVE
  or be itself deprecated/EOL — verify per `security-standard.md` §4 (NVD/CVE/Sonatype OSS Index).
- **Git write safety.** Never `git commit`/`push`/`checkout -b` on your own initiative; leave
  changes as unstaged working-tree edits. Record checkpoints by reading HEAD, not by committing.
- **Absolute paths only**, resolved from the repo root.

## Gate controls (at every gate)

- **continue** / **yes** — proceed to the next stage
- **stop** — halt; a summary of completed stages + resume instructions is shown

## Execution Steps

### 1. Initialize

Run `{SCRIPT}` from repo root and parse available paths. Read the constitution and, if present,
`plan.md` (tech stack) and the agent context file for conventions the migration must respect.
Resolve the repo root once; derive absolute paths from it.

### 2. Resolve the target(s) & current state

- Identify each thing to upgrade and its **current** version from the manifest/lockfile
  (`package.json`+lock, `pyproject.toml`/`requirements.txt`, `go.mod`, `pom.xml`/`build.gradle`,
  `*.csproj`, the runtime in `.nvmrc`/`Dockerfile`/CI).
- Determine the **target** version. If the user named it, use it. Otherwise detect candidates
  with the ecosystem's own tool — `npm outdated`, `pip list --outdated`, `go list -m -u all`,
  `mvn versions:display-dependency-updates` — and cross-check runtime/framework support windows
  against **endoflife.date**. Prioritize: EOL/at-risk runtime > known-CVE dependency (per
  `security-standard.md` §4) > major behind > minor/patch.
- Present the proposed target list and let the user confirm/adjust before any research or edits.

### 3. Research breaking changes (real changelog — not memory)

For each target, read the **actual** upgrade notes between current and target version:

- The package's CHANGELOG / GitHub Releases / migration guide (`UPGRADING.md`, "migrating to
  vN" docs), and for a runtime/major framework its official migration guide + endoflife.date.
- Extract the breaking changes and, crucially, **which ones this codebase actually hits** —
  grep the repo for the removed/renamed/changed APIs, config keys, and options. A breaking
  change the project never uses is noted as N/A, not work.
- Note whether an **official codemod / automated migration** exists (e.g. `npx <pkg>-codemod`,
  a framework `migrate` command) — prefer it over hand-editing where available.
- If the network/tool lookup genuinely can't run, say so and stop rather than guessing a
  migration from memory — an upgrade planned from a stale recollection is how regressions ship.

### 4. Plan the staged migration (confirmation gate)

Write the plan to `docs/upgrades/<target-slug>/plan.md`:

- **Stages** — ordered, each a bounded testable increment: transitive/peer deps that must move
  first, then one major bump per stage; group only same-risk minor/patch bumps together.
- **Per stage** — the version change, the breaking changes it triggers that this code hits, the
  files affected, and whether a codemod covers part of it.
- **Regression net** — the test command(s) that will gate each stage; if coverage is thin for
  the affected areas, flag it and recommend `/rudis.test` first.
- **Rollback** — one checkpoint per stage; whole-upgrade rollback = the Step-5 baseline HEAD.

**Gate:** show the plan; proceed only on explicit confirmation. Freeze the target set here.

### 5. Establish the regression baseline

Record the **baseline HEAD** (rollback point); if the tree is dirty, tell the user and offer to
commit/stash first. Run the project's build + test suite **now** and capture the green baseline
(counts). If it is already red, stop and report — you cannot tell an upgrade regression from a
pre-existing failure otherwise. Append a session entry to `docs/upgrades/<target-slug>/upgrade-log.md`.

### 6. Apply & verify each stage

For **each stage** in the frozen plan, in order:

1. **Checkpoint** — record this stage's checkpoint HEAD in `upgrade-log.md`.
2. **Bump** — update the manifest to the stage's version and refresh the lockfile with the
   ecosystem's installer (do not hand-edit a lockfile).
3. **Migrate** — run the official codemod if one exists, then hand-fix the remaining breaking
   changes this code hits, following the surrounding code's conventions (Mimicry Principle).
   Update deprecated APIs to their replacements, not to another soon-deprecated form.
4. **Verify** — type check / compile, lint, build, then the tests covering the touched areas
   (and E2E/API tests if the upgrade touches runtime/HTTP behavior). **Self-fix loop:** fix and
   re-run, up to 3 attempts per distinct error; a survivor is logged UNRESOLVED with a
   diagnosis. Compare against the Step-5 baseline — a test that was green and is now red is a
   regression to fix before advancing, not to accept.
5. **Security re-check** — confirm the new version has no known Critical/High CVE and is not
   itself deprecated/EOL (`security-standard.md` §4 — NVD/CVE/Sonatype OSS Index). A positive
   hit is a BLOCKER: move to a patched version or stop.
6. **Log** the stage result in `upgrade-log.md`.

**Stage gate** (after each stage except the last): report what changed + verification result;
**continue** / **stop** (resume later from this stage). Never start the next stage on top of an
UNRESOLVED regression unless the user explicitly accepts the known issue.

**Final full regression (after the last stage).** The per-stage runs above are scoped to the
touched areas, but a version migration — especially a framework or runtime bump — ripples beyond
the files you edited. Run the **full** build + test suite once now and compare it to the Step-5
green baseline: any test that was green then and is red now is an upgrade regression to fix before
reporting done. This single full run at the end is the real regression gate; the scoped per-stage
runs are not sufficient proof on their own.

### 7. Report

- Per-stage summary: version changes, breaking changes handled (and codemods run), from
  `upgrade-log.md`.
- Diff overview: `git diff <baseline HEAD>` stats.
- Verification: build/test results per stage + final, anything UNRESOLVED, and the security
  re-check outcome per upgraded package.
- **Rollback map:** per stage `git checkout <stage checkpoint> -- <paths>`; whole upgrade
  `git reset --hard <baseline HEAD>` (warn: discards the entire upgrade).
- Follow-ups: manual steps a codemod couldn't do, deprecations to revisit, and any target
  deferred out of scope.

## Rules

1. **Read, don't recall.** Every breaking change, deprecation, EOL date, and CVE comes from a
   real source read this session — never from training-data memory.
2. **Stage and verify.** One major per stage; no stage advances over a regression or a security
   BLOCKER without an explicit user decision.
3. **Safety net before surgery.** Recommend `/rudis.test` first when the affected areas lack
   coverage — an upgrade with no regression net is a guess.
4. **Delegate the edits' discipline.** Apply changes with the same Mimicry + verification rigor
   as `/rudis.implement`/`/rudis.redesign`; this command sequences the migration, it does not
   invent a new editing style.
5. **Artifacts must not drift.** Update `plan.md`'s tech-stack/version fields (and the brd
   dependency inventory, if present) when the upgrade changes them.
6. **Never auto-commit or auto-branch.** Changes stay as reviewable working-tree edits.
