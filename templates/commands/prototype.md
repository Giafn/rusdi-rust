---
description: Fast-path prototype builder for lead/PoC work, any size. Design/visual PoCs (screens, mockups, dashboards, decks, demo animations) are routed to the huashu-design skill (auto-installed when missing); everything else is scaffolded as a runnable throwaway prototype. Output lives under prototypes/<name>/ with a single thin PROTO.md — outside the governed SDD cycle, no SDD gates, no git writes.
handoffs:
  - label: Graduate to a real spec
    agent: rudis.specify
    prompt: "Bootstrap a specification from this prototype. Read prototypes/<name>/PROTO.md as the starting point — treat its Assumptions/shortcuts and Skipped governance sections as the gaps to close for production."
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty). The input is a
free-text description of the lead or idea to prototype — e.g. "billing dashboard for the
Acme lead", "a WhatsApp order-intake bot PoC", or "landing page + waitlist for the fintech
prospect". Assume this description is available even if `{ARGS}` appears literally below;
do not ask the user to repeat it unless the command was invoked empty.

## What this command is (and is not)

`/rudis.prototype` exists for the **PoC stage**: inbound leads that need something runnable
in front of a prospect fast, to demonstrate value and get a round of feedback. It is a
deliberately **light** flow that lives **outside** the governed SDD cycle
(`specify → plan → tasks → implement`).

**Output size and process are decoupled.** A prototype may be small (one screen, a mock
API) or large (many entities, several screens, even a faked multi-service shape) — the
process stays flat either way. Size is never a reason to switch to the cycle; **governance
is**. This is the one command you need for a PoC, whatever its size.

- It is **not** a replacement for the SDD cycle and never relaxes or bypasses that cycle's
  gates — it simply does not use them.
- It does **not** produce production-ready code. Every artifact it writes is stamped as a
  throwaway prototype.
- It does **not** perform any git write (no branch, no commit, no push). See Operating
  Constraints.

The cycle enters **only at graduation** — when a converting lead's PoC becomes real,
production, governed work, at which point `/rudis.specify` ingests the `PROTO.md`. Do **not**
push a user toward the cycle merely because the prototype is big; push there only when they
want governance (enforced contracts, field-parity, a codebase the team will build on).

### Two routes

This command picks one of two routes (Step 2):

- **Design route → [huashu-design](https://github.com/alchaincyf/huashu-design).** When the
  PoC is something the prospect *looks at and clicks* — UI screens, web/app mockups, landing
  pages, dashboards, pitch decks, demo animations, infographics — this command does **not**
  design anything itself. It makes sure the `huashu-design` skill is installed, hands the lead
  to it, and wraps the result in `PROTO.md`. Rudis deliberately does not maintain its own design
  logic: huashu-design is used as-is, so its upstream updates flow straight through.
- **Build route → the scaffold flow below.** Bots, APIs, CLIs, integrations, data pipelines,
  or any demo that needs a real running server or real data are built directly by this
  command, in the detected stack.

## Operating Constraints (STRICT)

- **No governance, not "no work".** "One-shot" means no governance ceremony — no approval
  loop, no scope freeze, no implementation lock, no multi-document artifact set
  (`spec.md`/`plan.md`/`tasks.md`/`data-model.md`/`contracts/` are never produced here). It
  does **not** mean one tool call: a larger prototype may be built incrementally across many
  edits and self-verified until it runs. Build as much as the demo needs; just never spin up
  an SDD gate. On the design route, huashu-design's own **design checkpoints** (e.g. showing
  three directions before building) still apply — they are design choices for the user, not
  SDD governance, and this command must not skip or override them.
- **The documentation floor is one file, always.** Regardless of prototype size, the only
  document this command writes is a single thin `PROTO.md` (plus a `README.md`). It never grows
  into the SDD document set. A big prototype gets more *code*, not more *process* and not more
  *docs* — `PROTO.md` stays the one place that records what was faked and what graduation must
  close. (On the design route, huashu-design may leave its own working files inside the
  prototype folder, e.g. `direction-approved.md` or `brand-spec.md`; keep them — they belong to
  huashu, not to the SDD document set.)
- **Isolated output.** All output goes under `prototypes/<short-name>/` at the repo root.
  **Never** write into `specs/`, and never touch an existing governed spec. A PoC is
  disposable; keeping it isolated means it can be deleted with `rm -rf prototypes/<name>`
  and never pollutes the governed surface or its tooling.
- **No git writes (per AGENTS.md).** Leave everything as unstaged working-tree edits for the
  developer to review. Never run `git commit`, `git push`, `git checkout -b`, `git branch`,
  `git reset`, or any tag/force operation, and never pass `--create-branch`. Stay on the
  current branch. This applies to huashu-design while it runs under this command too.
- **Skip constitution, but record the debt.** To stay fast, the prototype skips constitution
  gates. It must record in `PROTO.md` exactly which rules it bypassed, so the technical debt
  is visible at graduation time rather than discovered later.
- **Throwaway framing is mandatory.** `PROTO.md` and the prototype's `README.md` both carry a
  prominent **"⚠️ PROTOTYPE — not production-ready"** banner. Never present prototype code as
  production-ready.
- **Speed over completeness.** Build only what the demo scenario needs. Fake, stub, or
  hardcode anything not essential to the demo (sample data, auth bypass, happy path only) —
  and list every such shortcut in `PROTO.md`.

## Execution Steps

### 1. Understand the lead

Parse the lead/idea description from the user input. Derive:

- a **short slug** (2–4 words, kebab-case) for the directory name, e.g. `acme-billing`;
- the **one-line value proposition** the demo must land;
- the **audience** of the demo (who the prospect is, who will watch);
- the **demo scenario** — the single happy path the prototype must show end to end. When the
  demo has screens, write it as an ordered list of screens (e.g. login → dashboard → invoice
  list → invoice detail → pay).

If the description is empty or too vague to derive a demo scenario, ask ONE concise question
to pin down what the prototype must demonstrate, then proceed. (In non-interactive sessions,
proceed with the safest reasonable interpretation and record it under Assumptions.)

**Scope is not a blocker — it is the user's choice.** If the lead describes a large or
production-shaped system (e.g. "full CRUD across these modules", microservices, a
microfrontend setup), do **not** refuse it and do **not** silently try to stand up real
infrastructure. Offer the user the choice, in one short question:

- **(a) Fast demo here** — build it as a prototype: realize the one or two flows that win the
  demo, fake the rest (hardcoded data, a mock or single API, a faked service shape), stay in
  this command. This is the default for a PoC.
- **(b) Real, governed build** — if they want enforced contracts, real services/data, and a
  codebase the team will build on, this is the SDD cycle; hand off to `/rudis.specify`.

In non-interactive sessions, take **(a)** and record the choice under Assumptions — never hand
off to the governed cycle without the user's say-so.

Pick the demo scenario that best sells the value in option (a) and keep the build focused on
it; the prototype can still be large, but it is scoped to the demo, not to full production
coverage. Record in `PROTO.md` what you realized vs. faked.

### 2. Choose the route

Decide from the demo scenario:

- **Design route** (Step 3) — the value of the demo is in what the prospect **sees and clicks**:
  UI screens, web or mobile app mockups, landing pages, dashboards, pitch decks / slides,
  demo animations, infographics. Data can be faked inside the page. Any number of screens is
  fine — huashu-design builds multi-screen, clickable prototypes.
- **Build route** (Steps 4–5) — the demo needs **logic that really runs**: a bot, an API, a
  CLI, an integration, a data pipeline, real login, a real database, or code the team will
  keep in the project's own stack.
- **Mixed** — if the backend can be faked inside the page without hurting the demo, take the
  design route; if the demo only works with a real running server, take the build route.
- **Unclear** — ask ONE short question ("Is this mainly about how it looks, or about logic that
  must really run?"). In non-interactive sessions take the build route and record the choice
  under Assumptions.

State the chosen route and the one-line reason before continuing.

### 3. Design route — hand off to huashu-design

**3.1 Make sure huashu-design is installed.** Look for `huashu-design/SKILL.md` in the skill
directories your agent loads — e.g. `.claude/skills/` or `~/.claude/skills/` for Claude Code,
`.agents/skills/` or `~/.agents/skills/`, or your agent's own skills folder — or check
`npx skills list -g`. If it is already available as a skill in this session, that counts too.

If it is missing, install it **automatically** — say one line ("huashu-design is not installed
yet; installing it now") and run, installing for the **current agent only**:

```bash
npx --yes skills@latest add alchaincyf/huashu-design -g -y -a <agent-id>
```

`<agent-id>` is your agent's id in the `skills` CLI: `claude-code` (Claude Code), `codex`,
`cursor` (Cursor), `gemini-cli`, `github-copilot`, `opencode`, `windsurf`, `qwen-code`, `kilo`
(Kilo Code), `augment` (Auggie), `roo`, `codebuddy`, `qoder`, `amp`, `bob` (IBM Bob),
`antigravity` (Antigravity). If your agent has no id in that list, leave out `-a <agent-id>`;
the CLI then installs for every agent it detects. Keep `--yes` right after `npx` — it stops npx
from pausing on an install prompt, and the explicit form is not rewritten by command-rewriting
shell hooks.

Then confirm the install is complete: the skill folder must contain `SKILL.md` **and** the
`references/`, `assets/`, and `scripts/` subfolders (huashu depends on them; old `skills` CLI
versions copied only `SKILL.md`). If `npx` is not available, fall back to
`git clone https://github.com/alchaincyf/huashu-design.git <agent skills dir>/huashu-design`.

If installation fails (no Node/npx, no git, no network, permission error), tell the user
plainly what failed and why, then **fall back to the build route** (Step 4) so they still get
a prototype. Record the fallback under Assumptions.

**Keep it current (at most once a week).** If huashu-design was already installed (not installed
just now), read `.rudis-update-check` in the skill folder — one line, a date `YYYY-MM-DD`. If the
file exists and is less than 7 days old, skip this. Otherwise update it:

- **Installed with the `skills` CLI** (no `.git` folder in the skill folder):
  `npx --yes skills@latest update huashu-design -g -y`
- **Installed with `git clone`** (the skill folder has `.git`):
  `git -C <skill folder> pull --ff-only`

Whatever the result, write today's date into `.rudis-update-check` in the skill folder (after the
update, since an update may replace the folder). Say one line only if a new version was
installed ("huashu-design updated to the latest version"). If the update fails (offline, rate
limit, local changes), say so in one line and continue with the installed version — never block
the prototype on an update. (The `git pull` touches only the skill's own folder, never the
project repo, so it is outside the no-git-writes rule.) Why Rudis does this: huashu-design's own update notice only works
for a `git clone` install, so an install made with the `skills` CLI would otherwise never pick up
upstream improvements.

**3.2 Load it.** Invoke `huashu-design` as a skill if your agent supports skills and it is
loaded. Otherwise (e.g. it was installed just now and is not in this session's skill list, or
your agent has no skill mechanism) read its `SKILL.md` directly and follow it; its
`references/`, `assets/`, and `scripts/` paths are relative to the skill folder. Do not copy,
summarize into this repo, or re-implement huashu's rules — always use the installed skill, so
its upstream updates apply.

**3.3 Ask the look & feel once.** Before handing off, ask the user ONE short question with
three parts (skip any part the lead description already answers):

- **Tone** — calm / professional, or bold / eye-catching?
- **Density** — compact (key info only, the rest in detail views), or information-rich?
- **Reference** (optional) — the client's brand, an existing app, or a site it should feel like.

If the user does not answer, or the session is non-interactive, use the default **"clean and
professional, compact, no reference"** — a PoC audience is usually a B2B prospect — and record
it under Assumptions. Why this matters: without a style, huashu-design treats the task as having
no direction and deliberately spreads its three directions wide (one is picked at random), so at
least one usually comes out loud or busy. With a style, its three directions stay inside that
style.

**3.4 Hand off.** Give huashu-design the task with this brief, and let it run its own process
from there (its questions, its three-direction step, its verification):

- the lead description, value proposition, audience, and the demo scenario as an ordered
  list of screens from Step 1;
- the **look & feel from Step 3.3** (tone, density, reference) as the user's style direction —
  huashu still shows three directions, but all within this style;
- **scope: the whole demo flow** — every screen in the list, clickable — not a single screen;
- **output folder: `prototypes/<slug>/`** at the repo root;
- **no git writes** of any kind;
- **split the build when it is big.** If the demo has **more than 4 screens** and your agent
  can spawn subagents: once the user has picked a direction, first write the shared layer
  (design tokens, primitives, shared components, mock data) in the main session, then hand
  groups of screens to parallel subagents that build on that shared layer, and keep only the
  assembly and verification in the main session. huashu-design already allows this
  (multi-file split with one agent per screen group); asking for it keeps a large PoC from
  filling the main context. Agents without subagents build the screens in sequence instead.

**3.5 Wrap up.** When huashu-design has delivered, write `prototypes/<slug>/PROTO.md` (template
in Step 5) and add the "⚠️ PROTOTYPE — not production-ready" banner and run/open instructions
to `prototypes/<slug>/README.md` (create it if huashu did not). In `PROTO.md`, record that it was
built via huashu-design, that it is browser-only HTML/React (not the project's own stack), that
all data is fake, and anything else huashu stubbed. Then go to Step 6.

### 4. Build route — detect the stack (auto-detect)

Choose the stack in this order, stopping at the first that resolves:

1. **Existing repo.** If the working directory already has a recognizable stack (a package
   manifest, framework config, or established project layout), reuse it so the prototype fits
   in and runs with the project's existing tooling.
2. **Active marketplace profile.** If a Rudis profile is active (e.g. a `tm-forum-oda` or
   `pegadaian` profile under `.rudis/`), use the stack that profile defines.
3. **Fallback.** If neither resolves, ask the user **once** which stack to use (offer a
   sensible default for the kind of demo), then proceed. In non-interactive sessions, pick the
   lightest stack that satisfies the demo scenario and record the choice under Assumptions.

State the detected stack and how it was chosen before scaffolding.

### 5. Build route — scaffold `prototypes/<slug>/`

Create the directory and populate it:

- **The runnable app** — source for the detected stack that implements the demo scenario from
  Step 1, and only that. Build exactly as much as the scenario needs — small when the scenario
  is small, larger (more entities/screens, a faked service shape) when it is not — and fake
  everything the demo does not exercise. Scope to the demo, not to full production coverage.
- **`README.md`** — the "⚠️ PROTOTYPE — not production-ready" banner, a one-line description,
  and exact run instructions (install + start commands, URL/port if any).
- **`PROTO.md`** — the single thin brief, using exactly these sections (both routes use this
  template):

```markdown
# PROTO: <Title>

> ⚠️ PROTOTYPE — not production-ready. Disposable PoC built with /rudis.prototype.
> Route: <design (huashu-design) | build (<stack>)>

## What
<one-line value proposition being demonstrated>

## Who
<the lead/prospect, and the audience of the demo>

## Demo scenario
<the happy path the prototype shows, step by step>

## Assumptions / shortcuts
- <what was faked, stubbed, hardcoded, or skipped — e.g. sample data, auth bypass, happy path only>

## Skipped governance
- <which constitution rules were intentionally bypassed, and why it was safe for a PoC>

## Graduation notes
<what would have to change for production; the first step is: run /rudis.specify pointed at this PROTO.md>
```

Keep `PROTO.md` short — it is a demo note plus the bootstrap input for graduation, not a spec.

### 6. Make it run

Where the prototype is previewable (web app, server, CLI, HTML page), actually start or open
it and confirm it runs and shows the demo scenario; share concrete proof (a screenshot for UI,
or the output / served URL). On the design route, huashu-design's own verification counts as
this proof. Where it is not previewable in this environment, state exactly how the user runs
it and what they should see. Never claim it works without having verified it or, failing that,
saying plainly that it was not run here.

### 7. Report and offer graduation

Close with:

- the route taken (design via huashu-design, or build) and why;
- the path to the new `prototypes/<slug>/` and a one-line map of what's inside;
- how to run or open it (echo the README commands);
- the key shortcuts taken (from `PROTO.md`'s Assumptions) so expectations are set;
- the **graduation path**: if the lead converts, run `/rudis.specify` pointed at
  `prototypes/<slug>/PROTO.md` to start the governed cycle — the prototype becomes *input*
  to a real spec, not the final code.

## Iterating on a prototype (no sub-command)

Changes to an existing prototype are made with a **plain free-text prompt** — "make the
stat cards a chart", "add a region filter", "wire it to a real API" — not with a
sub-command. The agent edits the files under `prototypes/<slug>/` directly. For a prototype
built on the design route (its `PROTO.md` says `Route: design`), keep using huashu-design for
visual changes. Do **not** route prototype tweaks through `/rudis.redesign`: that command is for
*governed, implemented* features (scope-lock, checkpoint, confirmation gate, contract checks)
and its ceremony works against the whole point of a throwaway PoC.

Two things to keep straight while iterating:

- **Keep `PROTO.md` honest.** A change that alters an **assumption or the demo scenario**
  (e.g. swapping dummy data for a real API, or adding a new flow) must be reflected in
  `PROTO.md`'s *Assumptions / shortcuts*, *Demo scenario*, or *Skipped governance* sections —
  it is the bootstrap input for graduation and must not drift. Purely cosmetic tweaks (colour,
  copy, layout) need no `PROTO.md` update.
- **Watch for the graduation signal.** If the "change" is really "this is becoming a real
  product the team will build on", that is **graduation**, not iteration — stop free-prompting
  and hand off to `/rudis.specify` (see above). Growing scope alone is fine and stays here;
  wanting governance is the line.

All the Operating Constraints above still hold during iteration — no git writes, no SDD
document set, output stays under `prototypes/<slug>/`, and the throwaway banner stays on.

## Rules

- Never write into `specs/`, never modify a governed spec, and never generate `plan.md` or
  `tasks.md` from this command.
- Never perform a git write of any kind, and never create or switch branches.
- Never present the prototype as production-ready; keep the throwaway banner on every artifact.
- Always record shortcuts and skipped governance in `PROTO.md` — a prototype with no recorded
  assumptions is almost always hiding them.
- Keep the process flat at any size: never spin up approval gates, scope freeze, an
  implementation lock, or the SDD document set — a bigger prototype means more code, not more
  process. Route to `/rudis.specify` only when the user wants **governance** (graduation to
  real/production work), never merely because the prototype is large.
- On the design route, use the installed huashu-design skill as-is: never copy its content into
  this repo, never re-implement its rules here, and only update it through the weekly check in
  Step 3.1.
