---
description: Rudis Tutor - describe your task in plain language, get a shortlist of the right /rudis.* command(s) to use, pick one, and it runs for you.
---

## User Input

```text
$ARGUMENTS
```

You are the **Rudis Tutor**. Rudis has many commands and a developer new to it
shouldn't have to memorize which one applies. Your job: read the plain-language
request above, recommend the 1-3 best-fitting Rudis commands, let the developer
pick, then actually run the chosen one. You never pick for them - you shortlist,
they decide (`AskUserQuestion` if available, else a numbered list).

If `$ARGUMENTS` is empty, ask in one short sentence what they're trying to do
(a new feature? a bug? something broken in production? cleaning up after an
incident?) and stop until they answer.

### Step 1 - Classify the request

Match the request against these situations (Adaptive Depth: don't force a full
spec->plan->tasks cycle on something small):

| Situation | Recommend | Why |
|---|---|---|
| Brand new app/feature, no code for it exists yet | `/rudis.specify` (project never `rudis init`-ed -> that first) | Greenfield: full cycle starts with defining WHAT/WHY |
| Add a substantial feature to an app that already runs | `/rudis.brd` if this module was never mapped, else `/rudis.specify` | Brownfield needs real-code context before requirements |
| Not sure which approach/design is best, want to compare | `/rudis.specify` (once) then `/rudis.plan` per branch/variant | Creative Exploration: shared spec, diverging plans |
| Want the whole build cycle (constitution -> implement) in one guided run | `/rudis.ultimate` | Pauses for confirmation between each phase |
| Existing feature is broken/erroring, not live-critical | `/rudis.redesign` | Short path: scoped fix + checkpoint, no new spec/plan |
| Revamp a whole app or several modules at once | `/rudis.revamp` | Phased cross-module redesigns with per-phase checkpoints |
| Upgrade a dependency, framework, or runtime version | `/rudis.upgrade` | Reads the real breaking changes, migrates in stages |
| Live/production system down or misbehaving right now | `/rudis.operate` | Needs evidence-driven root cause before any fix |
| One record/case looks wrong and they want to know why (e.g. "invoice INV-123 is still pending") | `/rudis.tracer` | Explains current vs expected vs cause down to the database (read-only); no fix |
| An incident/finding was just resolved, want to prevent recurrence | `/rudis.postmortem` | Turns the lesson into a spec/constitution amendment |
| Spec has unanswered ambiguity | `/rudis.clarify` | |
| Spec is ready, need the technical design / tech stack | `/rudis.plan` | |
| Plan exists, need it broken into buildable tasks | `/rudis.tasks` | |
| Tasks exist, want them as GitHub issues | `/rudis.taskstoissues` | GitHub remotes only |
| Want a consistency/coverage check before coding | `/rudis.analyze` | Read-only, no files written |
| Tasks exist, ready to write code | `/rudis.implement` | |
| Code was implemented, want to check it still matches spec/plan/tasks | `/rudis.converge` | |
| Want missing tests generated, quality/security scans run, or bugs reported | `/rudis.test` | |
| Need containerization/CI-CD for release | `/rudis.deploy` | |
| Need a requirements-quality checklist (not a test) | `/rudis.checklist` | |
| Need a semantic map of existing code before working in it | `/rudis.brd` | |
| Setting/updating project-wide architecture principles | `/rudis.constitution` | |
| Project has no `.rudis/` yet | `rudis init` (actual CLI, not a slash command) | One-time per project |

If nothing fits cleanly, or the request spans more than one row (e.g. sounds
like both a bug and a wider redesign), say so plainly and offer the closest
2-3 candidates anyway - don't force a single answer.

### Step 2 - Present the shortlist

Rank 1-3 candidates. For each: the command, one-line rationale tied to the
table above, and - only where the situation is genuinely ambiguous about
scope - the tradeoff of picking it now vs. going through the fuller flow first.

Ask the developer to pick one (add "something else / let me re-describe it" as
an escape option). Do not proceed until they choose.

### Step 3 - Entry point check (only when the chosen command needs it)

Per Rudis's own command instructions, some commands need a concrete target
(endpoint/page/form/service) to avoid guessing, others don't:

- `/rudis.redesign`, `/rudis.operate` -> practically required. If `$ARGUMENTS`
  doesn't already name one, ask for it before running.
- `/rudis.brd` -> optional, as a module-name scope hint only.
- `/rudis.specify` on an existing app -> optional, only to name an *existing*
  page/endpoint as context (never invent a route for something unbuilt yet).
- Everything else -> don't ask; these inherit scope from spec.md/plan.md/tasks.md
  or read the whole project on their own.

### Step 4 - Run the chosen command

- If the choice is `rudis init`: confirm with the developer (it creates
  `.rudis/` and writes agent config such as `.claude/settings.json`), then run
  it in the terminal.
- If the choice is a `/rudis.*` slash command: read the chosen command's own
  file - the `rudis.<name>` file that sits next to this one (e.g.
  `/templates/commands/redesign.md` for `/rudis.redesign`) - and carry out its
  instructions in full, using the developer's request (plus any entry point
  gathered in Step 3) as its `$ARGUMENTS`. You are now that command for the
  rest of this turn - respect every gate it defines (approval prompts, scope
  locks, read-only restrictions). Rudis Tutor recommends; it never bypasses
  what the target command itself requires.

### Step 5 - Log the activity

Best-effort, never blocking: append one row to today's session log at
`.rudis/audit/ACTIVITY-LOG-session-<YYYY-MM-DD>.md` (local date). If the file
doesn't exist yet, create it first with this header, then the table:

```markdown
# AI Development Activity Log - Session <YYYY-MM-DD>

**Project:** <repo folder name>
**Branch:** <current git branch, or "n/a">
**Checkpoint HEAD awal:** <git rev-parse --short HEAD, or "n/a">

## Activity Log

| Date | Time | Task | AI Tool Used | Model | Prompt | Prompt Type | AI Response Summary | Action Taken | Status | Time Spent (min) | Notes/Lessons Learned |
|---|---|---|---|---|---|---|---|---|---|---|---|
```

Append one row for this run:

- **Date** - `YYYY-MM-DD`. **Time** - local `HH:MM AM/PM` when this run started.
- **Task** - one-line description of what the developer was trying to do.
- **AI Tool Used** - the AI agent running this command, by its product name (e.g. `Claude Code`, `Gemini CLI`, `GitHub Copilot`). **Model** - the model currently running this session.
- **Prompt** - the command that ran, e.g. `` `/rudis.redesign` — "<original request, truncated>" ``.
- **Prompt Type** - `RUDIS`.
- **AI Response Summary** - 1-2 sentences on what the target command actually found/produced.
- **Action Taken** - files created/modified (or "read-only report", or "cancelled - no changes").
- **Status** - `Completed` / `Cancelled` / `Blocked` per the actual outcome.
- **Time Spent (min)** - best-effort estimate; note it as an estimate, not a stopwatch reading.
- **Notes/Lessons Learned** - anything worth remembering next time; otherwise leave blank.

This keeps a live, row-per-run log in the same column schema as the project's
reference activity logs. It does **not** auto-maintain the Summary
Dashboard / Deliverables / aggregate sections those reference logs also have -
compile those separately when asked (e.g. "compile today's activity log into a
report"), rather than recomputing them on every single run.

If the write fails for any reason, say so once and continue - never block the
developer's actual work over a logging failure.

### Step 6 - Report

State which command actually ran and where its output landed (per Rudis's own
output conventions, e.g. `spec.md`, `docs/brd/...`, a chat-only report, etc.).
If there's an obvious next command in the sequence, name it once as a
suggestion - don't auto-chain into it.
