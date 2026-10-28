---
description: Orchestrate a large, cross-module revamp of an existing application as a controlled, phased sequence of scoped redesigns — understand the old app (brd), plan a prioritized revamp roadmap, then drive /rudis.redesign module by module with per-phase checkpoints, verification, and rollback.
scripts:
  sh: scripts/bash/check-prerequisites.sh --json --paths-only
  ps: scripts/powershell/check-prerequisites.ps1 -Json -PathsOnly
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty). The input names the
revamp scope and intent — e.g., "revamp the whole billing + invoicing area to the new design
system", "modernize the admin app: auth, users, and reporting modules", or "align every
customer-facing page with designs/2025-refresh/". Treat it as a **multi-module scope hint**,
not a single-file target. A single form/page/endpoint is `/rudis.redesign`, not this command.

## Goal

`/rudis.redesign` surgically reworks **one part** of a feature; this command orchestrates a
**revamp that spans many parts across multiple modules** as one governed workflow. It exists
because a large revamp done as a loose pile of ad-hoc edits loses the thread — scope creeps,
modules drift out of sync, and there is no rollback path when phase 4 breaks what phase 2
fixed. This command adds the missing orchestration layer on top of the existing building
blocks (`brd` for understanding, `redesign` for each scoped change, `test`/`converge` for
verification); it does not replace them.

## Operating Constraints (STRICT)

- **Never a big-bang rewrite.** A revamp is a *sequence of scoped redesigns*, each small and
  verifiable. Never regenerate a module wholesale when incremental redesign suffices.
- **Phase isolation.** Each phase is independently checkpointed and verified. A later phase
  never starts until the current phase verifies clean (or the user explicitly accepts a
  known-issue and continues).
- **Scope lock across the whole revamp.** The set of modules/targets is frozen at the plan
  gate. Adding a target mid-revamp requires re-confirming the plan, not silent expansion.
- **Delegate, do not reimplement.** Each scoped change is executed by reading and following
  `/rudis.redesign`'s full instructions — including its own scope-lock, mimicry, confirmation,
  and rollback rules. This orchestrator adds sequencing and cross-module verification; it does
  not weaken or bypass `redesign`'s gates.
- **Absolute paths only**, resolved from the repo root.
- **No silent architecture changes.** Honor the constitution and each module's boundary
  context (plan.md/data-model.md/contracts/ when present, else `docs/brd/modules/<slug>.md`).

## Gate controls (at every gate)

- **continue** (or **yes**) — proceed to the next phase / next revamp phase
- **skip** — skip the current optional step (Understand when a fresh map already exists)
- **stop** — halt; a summary of completed phases + resume instructions is shown

## Execution Steps

### 1. Initialize

Run `{SCRIPT}` from repo root and parse available paths. Detect the platform once (`uname`
on Unix, PowerShell on Windows) and use the matching script variants throughout. Read the
constitution (`/memory/constitution.md`) for constraints the revamp must respect.

### Phase 1 — Understand (brownfield map)

The revamp plan needs an accurate picture of the old app. Check for a current semantic map:

- **If `docs/brd/` exists and is current** for the in-scope modules — reuse it; log
  `[Phase 1] Understand: existing map found, reusing` and offer **skip**.
- **If it is missing or stale** — read and follow `/templates/commands/brd.md`, scoped to the
  revamp's modules (pass the module hint as its `$ARGUMENTS`). This produces/refreshes
  `docs/brd/00-overview.md` and `docs/brd/modules/<slug>.md`.

Do **not** re-read the map later if it is already in context from running `brd` this session.

**Gate 1:**
> **Phase 1 complete** — application understood. Modules in scope: [list].
> Next: Phase 2 — Revamp plan.
> Continue? *(yes / stop)*

### Phase 2 — Revamp plan (scope-freeze gate)

Draft the revamp roadmap and write it to `docs/revamp/<revamp-slug>/plan.md`. It contains:

- **Scope** — the frozen set of modules/targets, each with the file(s) it touches (from the
  brd map's Screens/Pages, API surface, Key Entities). Anything not listed is out of scope.
- **Design source of truth** — the design file/system driving the revamp (Figma export,
  Claude Design HTML, mockups), if any. For UI, this drives *layout*; contracts/entities drive
  field names/types (same rule as `redesign`).
- **Phased sequence** — group targets into phases ordered by dependency and blast radius:
  **data/schema → backend/API → shared components → per-screen UI**, so a downstream phase
  never builds on an un-migrated upstream. Each phase lists its `redesign` targets.
- **Risk & test coverage** — per target, note (from the brd map's Test coverage / `test`'s
  detection) whether a regression safety net exists. Flag **no-coverage** targets: recommend
  `/rudis.test` for them *before* their phase runs.
- **Rollback strategy** — one checkpoint per phase; whole-revamp rollback = revert to the
  Phase-2 baseline HEAD.

Present the plan. **Freeze the scope on confirmation.**

**Gate 2:**
> **Revamp plan ready** — N phases, M targets across K modules. [X targets have no test coverage.]
> Scope will be frozen on continue. Recommended: run `/rudis.test` for no-coverage targets first.
> Options: **continue** (freeze & start Phase 1 of the revamp) · **adjust** (edit scope) · **stop**

### Phase 3 — Execute the revamp phase by phase

Record the **baseline HEAD** as the whole-revamp rollback point; if the tree is dirty, tell
the user and offer to commit/stash first. Append a revamp session entry to
`docs/revamp/<revamp-slug>/revamp-log.md` (date, baseline HEAD, plan path, phases).

**Load shared context once (token discipline).** The boundary artifacts every delegated
`redesign` needs — the constitution, `data-model.md`, `contracts/`, and each in-scope module's
`docs/brd/modules/<slug>.md` — are stable across the whole revamp. Load them **once** here and
run the prerequisite script (`{SCRIPT}`) **once** to resolve paths; then, when you delegate to
`redesign` per target, have it reuse this already-in-context material and **skip its own Step 1
script run and boundary re-reads**. Re-loading unchanged shared context and re-running the
script per target (N targets = N reloads) is the biggest avoidable cost of a large revamp.
Rebuild only the slice a phase actually changed (e.g. a contract a prior phase edited).

For **each phase** in the frozen plan, in order:

1. **Pre-phase safety net.** If any target in this phase was flagged no-coverage and the user
   asked to add tests first, read and follow `/rudis.test` for those files now.
2. **Checkpoint.** Record this phase's checkpoint HEAD in `revamp-log.md`.
3. **Execute each target** by following `/templates/commands/redesign.md` with the target and
   intent as its input. `redesign` enforces its own scope-lock, confirmation gate (Step 3),
   incremental edits, and per-file verification. Honor those — do not short-circuit. **Read
   `redesign.md` once** for the whole revamp and reuse it across every target/phase; do not
   re-read it per target (same cost discipline as `/rudis.ultimate` — re-reading unchanged
   command docs is the biggest avoidable token cost of an orchestrated run). **Parallelize
   within a phase:** targets in the same phase that are independent (touch disjoint file sets —
   e.g. separate per-screen UI redesigns) may run in parallel; keep execution serial only where
   targets share files or one depends on another. Never parallelize *across* phases — the
   dependency ordering (data → BE → shared → UI) is the point.
4. **Phase verification.** After the phase's targets, run type check / compile, lint, build,
   and the tests covering the touched files. **Self-fix loop:** fix and re-run, up to 3
   attempts per distinct error; a surviving error is marked UNRESOLVED with a diagnosis.
5. **Cross-module drift check.** Confirm the phase touched only its planned files
   (`git diff <phase checkpoint>`), and that DTO/contract field names/types still match
   contracts/ (or the module's `technical.md`/brd data model). A revamp must not drift the API.
6. **Log** each modified file and the phase result in `revamp-log.md`.

**Phase gate** (after each phase, except the last):
> **Phase [i/N] complete** — [targets done]. Verification: [pass / N UNRESOLVED].
> Next: Phase [i+1] — [summary].
> Options: **continue** · **stop** (resume later from Phase [i+1])
Never start the next phase while the current phase has an UNRESOLVED failure unless the user
explicitly chooses to continue with the known issue.

### Phase 4 — Gap check & final verify

After the last phase, verify the revamp as a whole rather than phase-locally:

- **Gap check — pick the path that matches the project:**
  - **Spec-driven target** (a `specs/<branch>/` with spec.md + plan.md + tasks.md exists for
    the revamped feature): read and follow `/templates/commands/converge.md` to assess the code
    against those artifacts and append remaining gaps as tasks. `converge` **requires**
    spec/plan/tasks (`--require-tasks`) — do not invoke it without them.
  - **Brownfield target** (revamp seeded from `docs/brd/`, no spec/plan/tasks): do **not** call
    `converge`. Instead assess each revamped module against its `docs/brd/modules/<slug>.md`
    (Capabilities/Screens preserved, API surface intact, Data Flow unbroken) and list any gap
    as a follow-up in `revamp-log.md` — do not silently leave them.
- Run the full build + the broadest test suite that is practical; report pass/fail. **Skip this
  if the last phase already verified the same tree clean with no edits since its checkpoint** —
  re-running an identical full build/test right after the final phase's own verification proves
  nothing new. Run only the broader suite *delta* not already covered phase-locally.
- Update the design artifacts the revamp changed (data-model.md / contracts/ / spec.md, or the
  affected `docs/brd/modules/<slug>.md` sections) so artifacts and code do not drift — same
  discipline as `redesign`'s Rules.

### Phase 5 — Report

- Per-phase summary of what changed (from `revamp-log.md`).
- Diff overview: `git diff <baseline HEAD>` stats.
- Verification results per phase + final (checks run, pass/fail, anything UNRESOLVED).
- **Rollback map:** per phase, `git checkout <phase checkpoint> -- <paths>`; whole revamp,
  `git reset --hard <baseline HEAD>` (warn: discards the entire revamp).
- Any converge gaps queued as follow-ups, and suggested next steps (e.g. `/rudis.test` for
  newly thin areas, `/rudis.redesign` for a target deferred out of scope).

## Rules

1. **Sequence, don't rewrite.** The whole value is controlled, verifiable phases — never a
   single sweeping regeneration.
2. **Freeze scope at Gate 2.** New targets mean a re-confirmed plan, not silent scope creep.
3. **Verify before advancing.** No phase starts on top of an UNRESOLVED failure without an
   explicit user decision to accept the known issue.
4. **Delegate to sibling commands** for the actual work — `brd` (understand), `redesign` (each
   scoped change), `test` (safety net), and `converge` for the final gap check **only on a
   spec-driven target** (brownfield revamps check against the brd map instead — converge needs
   spec/plan/tasks) — and follow their full instructions and gates; this command only
   sequences them.
5. **Respect user choices** — "stop" halts immediately with a resume summary; "skip" skips
   only the optional Understand step. Do not argue or re-ask.
6. **Artifacts must not drift.** Update data-model.md / contracts/ / spec.md, or the brd module
   map, whenever the revamp changes what they describe.
7. **Non-interactive sessions** (CI, `rudis run`, headless): never block on a gate. If the
   scope is unambiguous and every in-scope target is safe, proceed phase by phase with the
   safest in-scope interpretation and flag assumptions in the report; if scope is ambiguous or
   a target would spill outside the frozen set, stop and report rather than guess.
