---
description: Execute the implementation plan by processing and executing all tasks defined in tasks.md
scripts:
  sh: scripts/bash/check-prerequisites.sh --json --require-tasks --include-tasks
  ps: scripts/powershell/check-prerequisites.ps1 -Json -RequireTasks -IncludeTasks
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Interactive Clarification (ask when blocked or unsure)

When you cannot resolve a decision confidently from tasks.md, plan.md, the code, or project conventions, ask the user instead of guessing or looping:

- **Ask when** the decision is high-impact or hard to reverse (architecture fit, data-model / schema strategy, breaking API changes, security, deleting or overwriting existing files), a required spec/file/input is missing, or the phase self-fix loop (step 8) has been exhausted on a blocking error.
- **How**: pause and ask ONE specific question at a time. State your recommended option with a one-line reason, then give 2–4 concrete choices plus a short free-form alternative; the user may accept the recommendation by replying "yes" (same style as `/rudis.clarify`).
- **Proceed without asking** when the choice is low-impact and reversible or has a clear conventional default — pick the sensible default, record the assumption in `implementation-log.md`, and continue. Do not interrogate the user over trivia.
- **Bounded (no wait-loop)**: ask at most twice per decision. If still unresolved, apply your recommended default, log it as an assumption for review, and move on — never block indefinitely.
- **Non-interactive sessions** (CI, `rudis run`, headless): never block on a question — apply the documented default, flag the assumption prominently in the final report, and continue.

## Outline

1. Run `{SCRIPT}` from repo root and parse FEATURE_DIR and AVAILABLE_DOCS list. All paths must be absolute. For single quotes in args like "I'm Groot", use escape syntax: e.g 'I'\''m Groot' (or double-quote if possible: "I'm Groot").

   **Path resolution rules (STRICT)**:
   - Resolve the repo root ONCE at the start (`git rev-parse --show-toplevel` or the script output) and derive every file path from it as an absolute path. Never pass relative paths to file tools or scripts.
   - If a file or directory referenced by tasks.md/plan.md cannot be found, do NOT abort the whole run: report the unresolved path with a clear message (expected path, what referenced it, closest existing match), skip only the affected task(s), and continue with the rest.
   - When a task must create a file in a directory that does not exist yet, create the directory first — a missing folder is not an error.

2. **Check checklists status** (if FEATURE_DIR/checklists/ exists):
   - Scan all checklist files in the checklists/ directory
   - For each checklist, count:
     - Total items: All lines matching `- [ ]` or `- [X]` or `- [x]`
     - Completed items: Lines matching `- [X]` or `- [x]`
     - Incomplete items: Lines matching `- [ ]`
   - Create a status table:

     ```text
     | Checklist | Total | Completed | Incomplete | Status |
     |-----------|-------|-----------|------------|--------|
     | ux.md     | 12    | 12        | 0          | ✓ PASS |
     | test.md   | 8     | 5         | 3          | ✗ FAIL |
     | security.md | 6   | 6         | 0          | ✓ PASS |
     ```

   - Calculate overall status:
     - **PASS**: All checklists have 0 incomplete items
     - **FAIL**: One or more checklists have incomplete items

   - **If any checklist is incomplete**:
     - Display the table with incomplete item counts
     - **STOP** and ask: "Some checklists are incomplete. Do you want to proceed with implementation anyway? (yes/no)"
     - Wait for user response before continuing
     - If user says "no" or "wait" or "stop", halt execution
     - If user says "yes" or "proceed" or "continue", proceed to step 3

   - **If all checklists are complete**:
     - Display the table showing all checklists passed
     - Automatically proceed to step 3

3. Load and analyze the implementation context:
   - **REQUIRED**: Read tasks.md for the complete task list and execution plan
   - **REQUIRED**: Read plan.md for tech stack, architecture, and file structure
   - **REQUIRED**: Honor the plan's **Architecture Type** and **Integration Target**. If plan.md lacks these fields, detect them now: read the agent context file (`CLAUDE.md`/`AGENTS.md`), architecture docs (`docs/architecture*`, `ARCHITECTURE.md`, ADRs), and build config (module federation / micro-frontend setup, monorepo layout). Generated code MUST fit the detected architecture — e.g., a micro-frontend remote wired into the host shell, never a standalone app when the project is federated. If architecture is genuinely undeterminable, STOP and ask the user before generating frontend/backend scaffolding.
   - **REQUIRED for any task touching UI**: Honor the plan's **Existing Design System** field. If plan.md lacks it (older plan, or the plan step was skipped), detect it now the same way: sample existing pages/components for the component library actually in use, the theme/token source, and the icon set. Build every new page/component with that existing library/tokens/icons — never introduce a second, unrelated design system alongside what the repo already has. Only fall back to a domain profile's own reference design system when the field says "None found — greenfield frontend" (a genuinely new frontend with nothing to match yet).
   - **IF EXISTS**: Read data-model.md for entities and relationships — the database schema follows data-model.md, which mirrors the source resource model; it deviates from that shape ONLY where data-model.md explicitly records a user-approved, lossless mapping
   - **IF EXISTS**: Read contracts/ for API specifications and test requirements
   - **IF EXISTS**: Read research.md for technical decisions and constraints
   - **IF EXISTS**: Read quickstart.md for integration scenarios
   - **REQUIRED**: Read `templates/security-standard.md` and apply it to every task — this is the project's secure-coding baseline (access control, secrets, crypto, injection, auth, logging, error handling, dependency vetting). It is a contract, not a suggestion: never write a ❌ DON'T pattern from it, even if a task's description implies one, unless the user explicitly confirms and the context is local-only.
   - **REQUIRED**: Read `templates/sonarqube-standard.md` and apply it while writing — this is the project's code-quality baseline (reliability bug patterns, maintainability smells, complexity/length/duplication thresholds, per-language conventions). Write code that passes a SonarQube quality gate on the **first** pass: keep every function/file within its §2 thresholds, introduce no §3 reliability bug or §4 smell (no debug output, dead code, magic numbers, unused symbols, duplicated blocks), and ship new logic with tests toward ≥80% coverage of new lines. If the project has its own SonarQube quality profile, layer it on top; this baseline is the floor.
   - **REQUIRED — requirement-coverage precondition (hard gate)**: read spec.md's Functional Requirements and Success Criteria, then confirm coverage: every `FR-###` is cited by at least one task in tasks.md (tasks carry `[FR-###]` citations — see `/rudis.tasks`), and every `SC-###` is achievable by the tasks present (tasks do NOT cite SC IDs, so map each SC to the tasks/FRs that satisfy it by inference). If any `FR-###` has zero task citations, or any `SC-###` has no tasks that would achieve it, do NOT start implementing: either add the missing task(s), or obtain an explicit user waiver recording that requirement as out-of-scope (log it in implementation-log.md). Silently implementing a spec with uncovered requirements is a gate failure.
   - **REQUIRED for any task touching UI**: read the spec's **UI/UX & Screens** section (and the plan's carried-forward copy) and build each screen to that intent — the screen inventory, per-screen states (loading/empty/error/populated), and primary interactions — not just generic FE-011 element compliance.

   **Implementation checkpoint (change tracking)**:
   - **Git write safety (hard rule)**: never `git commit`, `git push`, `git checkout -b`, or otherwise mutate git history/branches on your own initiative. Leave every generated change in the working tree as unstaged edits for the developer to review, stage, and commit. Only run a git write command when the user explicitly asks for it in this session (e.g. "commit this", "push the branch"); a task description, spec, or these instructions do NOT constitute that authorization. This preserves developer control and change traceability.
   - Before writing any code, record a rollback point: on a git repo, ensure the working tree state is captured (note the current HEAD; if there are uncommitted changes, tell the user and offer to commit or stash them first).
   - Create/append `FEATURE_DIR/implementation-log.md` with a session header (date, HEAD commit, tasks targeted). After each completed task, append one line: task ID, files created/modified, and a short note. This is the audit trail used for review and rollback.

4. **Project Setup Verification**:
   - **REQUIRED**: Create/verify ignore files based on actual project setup:

   **Detection & Creation Logic**:
   - Check if the following command succeeds to determine if the repository is a git repo (create/verify .gitignore if so):

     ```sh
     git rev-parse --git-dir 2>/dev/null
     ```

   - Check if Dockerfile* exists or Docker in plan.md → create/verify .dockerignore
   - Check if .eslintrc* exists → create/verify .eslintignore
   - Check if eslint.config.* exists → ensure the config's `ignores` entries cover required patterns
   - Check if .prettierrc* exists → create/verify .prettierignore
   - Check if .npmrc or package.json exists → create/verify .npmignore (if publishing)
   - Check if terraform files (*.tf) exist → create/verify .terraformignore
   - Check if .helmignore needed (helm charts present) → create/verify .helmignore

   **If ignore file already exists**: Verify it contains essential patterns, append missing critical patterns only
   **If ignore file missing**: Create with full pattern set for detected technology

   **Common Patterns by Technology** (from plan.md tech stack) — the table below is a reference catalog; **apply only the row(s) for the stack this project actually uses** (per plan.md / detected files). Do not walk or emit patterns for languages the project doesn't use:
   - **Node.js/JavaScript/TypeScript**: `node_modules/`, `dist/`, `build/`, `*.log`, `.env*`
   - **Python**: `__pycache__/`, `*.pyc`, `.venv/`, `venv/`, `dist/`, `*.egg-info/`
   - **Java**: `target/`, `*.class`, `*.jar`, `.gradle/`, `build/`
   - **C#/.NET**: `bin/`, `obj/`, `*.user`, `*.suo`, `packages/`
   - **Go**: `*.exe`, `*.test`, `vendor/`, `*.out`
   - **Ruby**: `.bundle/`, `log/`, `tmp/`, `*.gem`, `vendor/bundle/`
   - **PHP**: `vendor/`, `*.log`, `*.cache`, `*.env`
   - **Rust**: `target/`, `debug/`, `release/`, `*.rs.bk`, `*.rlib`, `*.prof*`, `.idea/`, `*.log`, `.env*`
   - **Kotlin**: `build/`, `out/`, `.gradle/`, `.idea/`, `*.class`, `*.jar`, `*.iml`, `*.log`, `.env*`
   - **C++**: `build/`, `bin/`, `obj/`, `out/`, `*.o`, `*.so`, `*.a`, `*.exe`, `*.dll`, `.idea/`, `*.log`, `.env*`
   - **C**: `build/`, `bin/`, `obj/`, `out/`, `*.o`, `*.a`, `*.so`, `*.exe`, `Makefile`, `config.log`, `.idea/`, `*.log`, `.env*`
   - **Swift**: `.build/`, `DerivedData/`, `*.swiftpm/`, `Packages/`
   - **R**: `.Rproj.user/`, `.Rhistory`, `.RData`, `.Ruserdata`, `*.Rproj`, `packrat/`, `renv/`
   - **Universal**: `.DS_Store`, `Thumbs.db`, `*.tmp`, `*.swp`, `.vscode/`, `.idea/`

   **Tool-Specific Patterns**:
   - **Docker**: `node_modules/`, `.git/`, `Dockerfile*`, `.dockerignore`, `*.log*`, `.env*`, `coverage/`
   - **ESLint**: `node_modules/`, `dist/`, `build/`, `coverage/`, `*.min.js`
   - **Prettier**: `node_modules/`, `dist/`, `build/`, `coverage/`, `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`
   - **Terraform**: `.terraform/`, `*.tfstate*`, `*.tfvars`, `.terraform.lock.hcl`
   - **Kubernetes/k8s**: `*.secret.yaml`, `secrets/`, `.kube/`, `kubeconfig*`, `*.key`, `*.crt`

5. Parse tasks.md structure and extract:
   - **Task phases**: Setup, Tests, Core, Integration, Polish
   - **Task dependencies**: Sequential vs parallel execution rules
   - **Task details**: ID, description, file paths, parallel markers [P]
   - **Execution flow**: Order and dependency requirements

6. Execute implementation following the task plan:
   - **Phase-by-phase execution**: Complete each phase before moving to the next
   - **Respect dependencies**: Run sequential tasks in order, parallel tasks [P] can run together  
   - **Follow TDD approach**: Execute test tasks before their corresponding implementation tasks
   - **File-based coordination**: Tasks affecting the same files must run sequentially
   - **Validation checkpoints**: Verify each phase completion before proceeding

7. Implementation execution rules:
   - **Setup first**: Initialize project structure, dependencies, configuration
   - **Tests before code**: If you need to write tests for contracts, entities, and integration scenarios
   - **Core development**: Implement models, services, CLI commands, endpoints
   - **Integration work**: Database connections, middleware, logging, external services
   - **Polish and validation**: Unit tests, performance optimization, documentation
   - **Mimicry Principle (match existing patterns, don't reinvent)**: in a codebase that already has code, do NOT write a task's implementation from a blank-slate description — first find the nearest existing example of the same kind of thing (an existing controller, service, model, migration, test, error handler, logger call, auth guard) and **mimic its structure, naming, imports, and conventions**. New code should look like it was written by whoever wrote the surrounding code: same error-handling shape, same logging style, same validation approach, same file/module layout. This produces code that fits the project (and passes the field-parity/architecture gates below) far more reliably than inventing a fresh style. Deviate from an existing pattern only when the task explicitly requires it, and say so when you do. On a genuinely greenfield project with no example to copy, follow plan.md and the constitution instead.
   - **UI tasks run as a focused design pass (build the screen like a dedicated design prompt, not buried in compliance)**: a plain hand-written "design this screen" prompt reliably beats a pipeline-generated page for one reason — it is *focused*: all of the model's attention goes to one beautiful screen against a reference, with no backend/compliance rules competing for context. Reproduce that here. When a task builds or restyles a user-facing page, treat the visual build as its own step whose working context is ONLY the design inputs: the **visual reference** (the user's image/mockup, the **Existing Design System** detected in Step 3, or the profile's pulled design module), the **screen intent** from the spec's *UI/UX & Screens* section, and **FE-011 as a taste guide**. Deliberately set the backend/compliance concerns aside *while composing the layout* — TMF field parity, outbox, base-entity columns, security/Sonar gates do NOT shape how the page looks; they are verified afterward by the step 8 gates. **Prefer to delegate the page build to a separate focused sub-agent** given just these design inputs (via the Task tool if available), so the large implementation context can't dilute visual attention; then wire data onto what it produced and run the gates. A page generated here must be as considered as one from a standalone "design this screen" prompt — reference-driven and focused, never a formulaic fill-in of the checklist.
     - **Reference-first, never bare-by-default**: a real visual reference is the biggest lever on quality, so establish one before building. Use the spec's **Design source** / sibling pages / the profile's design module. If a UI task has *no* reference and nothing to match (genuinely greenfield), do NOT fall through to a bare framework look — first produce a small **design proposal** (a token set — brand/accent, hue-biased neutrals, semantic status colors, spacing scale, radius, elevation, type scale, one icon set — plus a one-screen layout sketch), state it in the phase report, and build to it. In an interactive session, surface that proposal for a quick confirm before building the rest; in a non-interactive run, proceed on the proposal and record it so it can be reviewed. This is the design-then-wire order: get the screen right against a reference (real or proposed) first, then attach data and run the compliance gates.

8. **Phase verification gate** (run after EVERY phase, before moving on — this prevents error chains):
   - **Build-once, reuse across phases (token discipline for the checks below):** the reference material these gates compare against is stable within a run — build it **once** and reuse it every phase instead of reloading it each time. Specifically: `security-standard.md` and `sonarqube-standard.md` were already loaded in step 3 (do not re-read them per phase); build the per-resource **field inventory** (from `contracts/`), the **operation inventory** (from the backend), and the **source digest / data-model conformance checklist** once, then cache them. **Invalidate and rebuild only the affected slice** when a phase actually changes its source — e.g. a phase that edits a contract rebuilds that resource's field inventory; a phase that adds an endpoint adds it to the operation inventory. Each phase then reconciles only the resources/operations it *touched* against the cached inventory, not the whole project. (Correctness invariant: never let a stale inventory hide a mismatch — if in doubt whether a change invalidated the cache, rebuild that slice.)
   - Run the project's cheapest available checks in this order: type check / compile (e.g., `tsc --noEmit`, `mvn compile`), then lint, then build, then the tests relevant to the phase.
   - **Keep gate output compact** (token thrift): this gate runs after every phase, so do NOT paste full build/lint/test logs into the run. Collapse passing/framework noise to counts; keep verbatim only the failing messages, assertions, and stack traces needed to diagnose and fix (same output-compression rule as `/rudis.test`).
   - **Self-fix loop**: if a check fails, fix the error and re-run — up to 3 attempts per distinct error. Actually fix errors; never just report a recommendation and move on. If an error survives 3 attempts, record it in implementation-log.md as UNRESOLVED with your best diagnosis and continue only if later phases don't depend on it; otherwise stop and report.
   - **Source-conformance check** (the "matches the PDF" gate — run whenever a phase touches an entity, migration, DTO, contract, or FE type): compare the generated artifacts back to the source resource model via the digest under `FEATURE_DIR/inputs/` and `data-model.md`'s conformance checklist. Verify: every resource → table, every sub-resource → child table, every attribute → column (exact name/type/cardinality), every enum/state type present with exactly the source's values kept per resource — **nothing missing, renamed, invented, or simplified**. Any merge/flatten must be a deviation explicitly recorded in `data-model.md` and backed by a lossless mapper; an **unrecorded deviation is a gate failure**. This is cheap (compare against the text digest, not the raw PDF) and catches drift early, before it compounds into a rebuild.
   - **BE/FE/DB field-parity check** (run whenever a phase touches an entity, DTO, mapper, contract, or FE type/form): build ONE field inventory per resource from `contracts/` — the TMF wire contract is the source of truth for names, types, casing, and required/optional — then reconcile all three layers against it:
     - **API DTO ↔ contract**: every field name, type, casing (`camelCase`, `@type`/`@baseType`/`@schemaLocation`), and required/optional flag matches the contract exactly. A single mismatch (e.g. `unitCost` vs `costItemAmount`) causes cascading 400 errors — catch it here, not at runtime.
     - **DB ↔ contract via mapper**: by default the storage model in `data-model.md` mirrors the source, so it maps to the contract directly. Where `data-model.md` records an approved deviation (merged tables, flattened value objects, child tables), verify a mapper reassembles the full wire contract **losslessly** — every contract field produced, every column / child row consumed, nothing dropped, no orphan column. If a deviation merges sub-types with different lifecycles (e.g. `ActualCostItem` vs `ProjectedCostItem` states), verify state transitions are validated **per sub-type**, not against one combined enum.
     - **FE ↔ DTO**: `types.ts`, form fields, and the FormMapper use the same field names/types as the DTO; enumerations (state lifecycles) match, and per-subtype enums are not conflated.
     - On any mismatch, apply the self-fix loop above. A field present in one layer but unaccounted for in another is a **gate failure**, not a warning.
   - **BE→FE operation-coverage check** (run whenever a phase touches a controller, backend service method, or FE service/page — separate from the field-parity check above, which only verifies shared fields match, not that every operation exists on both sides): build ONE operation inventory per resource from the backend implementation (every exposed endpoint/controller method — CRUD plus custom actions like approve/cancel/export) and reconcile the frontend against it:
     - **FE service layer**: every backend operation has a corresponding function in the FE service file (`service/*.ts`), using the same route, method, and parameters as the backend controller.
     - **FE UI trigger**: every FE service function is actually invoked from a page/component (button, menu item, form submit, row action) — a service function nobody calls is as much a gap as a missing one.
     - An operation intentionally not exposed in the UI (e.g. an internal/system-only endpoint) is not a gap **only if** `data-model.md` or the spec records it as out-of-scope for FE; otherwise treat it as missing and implement it.
     - On any mismatch, apply the self-fix loop above. A backend operation with no FE service function, or an FE service function with no UI trigger, is a **gate failure**, not a warning.
   - **Visual composition check** (run whenever a phase adds or edits a page component. A domain profile, if active, defines FE-011 in its frontend constitution and may exceed this; **when no profile is active, this checklist IS the standard** — apply it against the project's own or greenfield design tokens, so a no-profile project still comes out designed, not basic). **Treat FE-011 as a floor and a taste guide, never a ceiling**: ticking every bullet is the minimum to not ship bare — it is NOT the definition of "done". The real bar is whether the page looks as good as the visual reference (and as good as a screen a designer would ship), not whether the checklist is satisfied. A page that passes every bullet but still looks generic/samey has failed the intent — keep raising it toward the reference. This is two passes, not one — the checklist alone is not sufficient:
     - **Pass A — element checklist**: for every list/overview/dashboard-landing page touched in this phase, confirm it is not a bare, flat composition — specifically: a stat/KPI summary row is present above the main table/grid; the empty state is icon/illustration + heading + helper text + primary CTA (never bare "No data" text); the loading state is a shaped skeleton, not a bare spinner, whenever the final layout is known; icons for the same status/entity/nav concept match what earlier pages in this feature already used; and the page has at least one tint/gradient/accent visual anchor instead of being built entirely from flat white/plain surfaces. Also confirm it uses the **Existing Design System** detected in Step 3, not a second/unrelated component library.
       **Beyond "not bare" — the polish bar** (what separates a page that looks *designed* from a merely functional one; enforce it especially when no profile is guiding the look, since that is where output defaults to basic): each KPI/stat tile carries a tinted icon chip + numeral + label + a trend/delta indicator or a small inline sparkline — a plain label+number tile is a fail; status/severity renders as a colored semantic chip (status dot + label) with a leading entity icon per row, never plain text; exactly one area per page spends the brand accent (header, featured stat, or primary CTA) with the rest kept neutral; grouped content sits on cards with deliberate soft elevation and a consistent radius, not flat borderless boxes; spacing follows one scale and columns of figures use tabular numerals. Every color resolves to a design token — never a hardcoded hex.
     - **Pass B — actually look at it rendered**: a checklist can pass while the page still looks bad (cramped spacing, clashing colors, misaligned grid, awkward proportions) — code-only review cannot catch that. **Pass B is MANDATORY, not best-effort, whenever this session has any browser/rendering tooling available** (a preview browser, Playwright/Puppeteer, a Storybook, or a dev server you can screenshot): a UI-touching phase MUST NOT be marked complete without a rendered screenshot of every page it added or changed. You may only take the headless fallback below when there is genuinely no way to render in this session — never to save time or skip a fiddly setup. Start the project's dev server and open each page touched in this phase in a browser (screenshot it), for at least one representative viewport. **Cost cadence (do not re-render everything every phase):** start the dev server **once** and keep it running across phases rather than restarting it each time; render **only the pages this phase actually added or changed**, not every page in the app, and skip a page whose rendered output cannot have changed since it was last screenshotted this run. Keep the localize-and-fix-now benefit (render at the phase that touched the page — do NOT defer all rendering to one end-of-run pass, or a layout defect introduced early gets built on and is hard to attribute), just without redundant re-renders of untouched pages. If the frontend is a Module-Federation remote that can't render standalone, render it through its host shell (or a Storybook/harness that provides `common_remote`) rather than expecting the remote's own dev server to show the page — a remote that 404s or renders blank on its own is a rendering-setup gap to fix, not a reason to skip Pass B or to mark it passed. Judge it holistically: spacing/whitespace rhythm, visual hierarchy, alignment/grid consistency, color harmony and contrast, and overall polish relative to sibling pages already in the app — not just "does it have a stat card". Then run this concrete **"offside" checklist** on each screenshot — any single failure is a gate failure:
       1. **No overflow/clipping**: no content spills outside its container, no unintended horizontal scrollbar, no text truncated or overlapping.
       2. **Alignment**: elements sit on a consistent grid — no control that's off-center or a few pixels out of line with its row/column, no ragged card edges.
       3. **Spacing scale**: gaps/padding follow one consistent scale (design-system spacing tokens) — nothing cramped, nothing with a randomly huge gap.
       4. **Color from tokens**: every color traces to a design token — no clashing/off-palette hue, no stray hardcoded color; status/entity colors match sibling pages.
       5. **Responsive**: re-check at one narrower viewport (e.g. tablet width) — the layout reflows without breaking, clipping, or overlapping.
       If something reads as visually rough even though Pass A passed, that is still a **gate failure**; fix it via the self-fix loop before moving on.
       - **Render → self-critique → fix loop (this is the step that closes the gap with a manual prompt)**: a manual prompt wins largely because a human renders the page, dislikes something, and re-prompts. Do that yourself here — do not accept the first render. After screenshotting, **critique your own screenshot out loud against the visual reference** (the user's mockup, the Existing Design System's sibling pages, or the profile's design module): name the 2–4 biggest gaps between what you rendered and how the reference actually looks and feels (hierarchy, spacing rhythm, color/accent use, component styling, density, polish). Then apply the fixes and **re-render**. Repeat this critique-and-refine cycle until the page holds up next to the reference — at least **2 rounds** for any non-trivial page, more while each round still finds a real gap. "It renders without errors" or "it passes the offside checklist" is NOT the finish line; "it looks as good as the reference / as a screen a designer would ship" is. Record the before/after in the phase report so the improvement is visible.
     - **Non-interactive/headless sessions** (CI, `rudis run`, genuinely no browser/rendering tooling available — confirmed, not assumed): the rendered Pass B cannot run — say so explicitly in the phase report ("visual rendering not verified — no browser available in this session") instead of silently skipping it or implying it was checked, and never claim Pass B passed without having actually rendered the page. Static enforcement is then REQUIRED, not optional, and is stricter to compensate for the missing eyes:
       1. If the active profile ships a static FE-011 linter (e.g. its `validation/fe011-lint.*`), you MUST run it over every page touched this phase and treat any ERROR finding as a gate failure (fix via the self-fix loop). A profile linter that exists but was not run is itself a gate failure.
       2. When no profile linter exists, still statically verify the Pass A element + polish-bar items you can read from source — design tokens used (no hardcoded hex/rgb/px), a stat/KPI summary row present, a non-bare empty state, a shaped skeleton loader, consistent icon-per-concept — and treat a missing item as a gate failure exactly as a rendered check would.
       3. Record every render-unverified page in the phase report and carry it forward as an open **"render-unverified UI"** item. A UI deliverable MUST get a real rendered Pass B before it is handed off as done — do not let a run finish with UI marked complete that was never looked at by anyone. If the whole run is headless, say so at completion and recommend a follow-up render pass (e.g. re-run the visual gate in a session with a browser) rather than implying the UI was visually verified.
     - Treat a page that's functionally correct but visually bare or rough (default library styling, no summary context, plain-text empty state, or a rendered result that just looks off) as a **gate failure**, not a warning — apply the self-fix loop above to add the missing element(s)/fix the composition before marking the phase complete.
   - **Security conformance check** (run whenever a phase touches an endpoint, auth, data access, crypto, file handling, external HTTP calls, or a new/updated dependency — run `templates/security-standard.md` §14's self-review gate against the code just written). **Do not duplicate the real-scan gate below:** where the real SAST/secret/dependency scanners run this phase, they own the *mechanical* categories (secret literals, dependency CVEs/deprecation, and pattern-matchable injection). This self-review then focuses on the **design-level** issues those tools cannot see — missing authorization checks, tenant/owner scoping in the `WHERE` clause, fail-open error handling, business-logic flaws, and DTO/response over-exposure (§2, §7–§9, §12). When a scanner for a category could not run, this self-review is the fallback that covers it. Walk each pattern flagged in the standard's category sections (§2–§12) and confirm none was introduced; walk §14's checklist and confirm every applicable "yes." Map any finding to severity per §1 (BLOCKER — Critical/High severity: e.g. hardcoded secret, missing authz check, unparameterized query, weak crypto, silent catch on a security-relevant path — blocks the phase; MAJOR — Medium severity: gets fixed in the same pass or an explicit user waiver logged in implementation-log.md; MINOR/INFO — Low/informational: logged and may proceed). A new dependency added this phase must pass §4's checks before it's used — treat an unvetted dependency as a gate failure. **Dependency CVE / deprecation / EOL verification is owned by the real-scan gate's Dependencies bullet below** (it runs the live tool query on the first phase across the inherited lockfile and thereafter whenever deps change, and confirms hits against NVD/CVE/OSS Index) — do not run that query a second time here; use its result, and treat a positive hit (Critical/High CVE, or deprecated/yanked/archived/EOL) as a BLOCKER unless a user waiver is logged. If that lookup could not run, the dependency is UNVERIFIED (MAJOR) — do not claim it was checked. Apply the self-fix loop above to any BLOCKER/MAJOR finding before marking the phase complete.
   - **Real static-scan gate — SonarQube + OWASP/SAST (run after any phase that writes or edits code, before the phase is marked complete).** The self-review checks above are the floor, not proof: a real scanner catches what a read-through misses, and passing the client's actual gate is what unblocks delivery ("code fails the scan → project doesn't get paid"). Run the real tools that fit the project's stack — scoped to the files this phase changed where the tool allows — and fix findings through the self-fix loop. Prefer a tool already in the project (its `package.json`/CI); if none is installed, use a zero-install runner (`npx <tool>`, `pipx run <tool>`, or `docker run`) when the ecosystem supports it.
     - **Cadence & cost (do not re-scan what didn't change — this keeps the gate fast and token-thrifty):** probe which scanners are available **once** at the run's start and reuse that result every phase — never re-attempt (and re-report) a tool already known absent. Then match scan frequency to blast radius: **SAST + secrets** are diff-scoped and cheap, so run them each code phase but **only over the files that phase changed**, and skip the category entirely when a phase touched no code/no diff. **Dependency/vuln scan and a full SonarQube scan are expensive and only change when the lockfile or config does** — run the dependency scan on the first phase (the inherited lockfile) and thereafter only when a phase adds/updates a dependency; run the full Sonar scan **once as a final pass before completion**, not after every phase (per-phase quality stays covered by the cheap local linter + the existing compile/lint gate). Run independent scanners **in parallel**, not serially. This turns an O(phases × 4 scanners) tax into roughly one cheap incremental scan per phase plus one full pass at the end.
     - **Quality (SonarQube parity):** each code phase, run the stack's cheap local Sonar-aligned linter over the changed files — JS/TS `eslint` with `eslint-plugin-sonarjs`, Python `ruff`/`pylint`, Java `mvn compile` + SpotBugs, Go `go vet` + `staticcheck` — treating findings against `sonarqube-standard.md` (§2 thresholds, §3 bug patterns, §4 smells) at the shared BLOCKER/CRITICAL/MAJOR/MINOR/INFO scale (`security-standard.md` §1). If the project has SonarQube config (`sonar-project.properties` or a CI job) **and** a scanner, run the full real analysis with the gate wait enabled as the **final pass** (not per phase) — same flow and token rules as `/rudis.test` §5.0 (never hardcode or echo `SONAR_TOKEN`; use the project's env/CI variable).
     - **SAST (OWASP):** run a security static analyzer over the changed code — `semgrep --config p/owasp-top-ten` (any ecosystem) or the language-native one: JS/TS `eslint-plugin-security`, Python `bandit`, Go `gosec`, Java SpotBugs + `find-sec-bugs`. Map each hit to its OWASP category and to §1 severity in `security-standard.md`.
     - **Secrets:** scan the working-tree diff for committed secrets — `gitleaks detect`/`protect` or `trufflehog`. A real hit is a **BLOCKER**: remove the secret, move it to env/vault, and tell the user to **rotate** it (a committed secret is compromised even once removed).
     - **Dependencies / known vulns:** run the ecosystem vuln scan required by `security-standard.md` §4 (`osv-scanner` on the lockfile, Sonatype OSS Index, `npm audit --omit=dev`, `pip-audit`, `govulncheck`). Confirm each flagged package+version against the authoritative sources — **NVD** (nvd.nist.gov) for the CVSS score, **CVE** (cve.org), **Sonatype OSS Index** (guide.sonatype.com) — and take the severity from NVD's CVSS. A known Critical/High CVE is a **BLOCKER**: pin to the advisory's patched version (or the maintained successor) before the phase completes.
     - **Severity handling & thrift:** BLOCKER/MAJOR findings are fixed via the self-fix loop before the phase completes, exactly like the other gates; keep output compact — report counts plus only the failing findings, never the full scanner logs.
     - **Honesty — no false green (hard rule):** if a scanner for a category genuinely cannot run (not installed and no runner available, or offline), say so in the phase report and fall back to that category's §14 self-review, **labeled as an approximation**. Never state the code "passes SonarQube" or "passes OWASP" for a category that no real tool actually scanned this session.
   - Never declare a phase complete while its verification gate fails.

9. Progress tracking and error handling:
   - Report progress after each completed task
   - Halt execution if any non-parallel task fails — but a *path resolution* failure follows the rules in step 1 (skip and continue, don't abort the run)
   - For parallel tasks [P], continue with successful tasks, report failed ones
   - Provide clear error messages with context for debugging
   - Suggest next steps if implementation cannot proceed
   - **IMPORTANT** For completed tasks, make sure to mark the task off as [X] in the tasks file.

10. Completion validation:
    - Verify all required tasks are completed
    - **Verify per requirement, with evidence**: iterate every `FR-###` and `SC-###` from spec.md and confirm the implementation satisfies it, citing the concrete file / endpoint / test that fulfills each — not a blanket "matches the spec". List any FR/SC that is unmet or only partially met as an explicit gap. Re-confirm the spec's Assumptions still hold, and flag any that were invalidated during implementation.
    - Validate that tests pass and coverage meets requirements
    - Confirm the implementation follows the technical plan (including Architecture Type/Integration Target) and the constitution
    - Report final status with summary of completed work, including:
      - Changed-files summary (from implementation-log.md) so the user can see exactly what this run touched
      - **Scan status per category** (SonarQube/quality, SAST/OWASP, secrets, dependencies): for each, state whether a **real tool ran** (name it + PASS/FAIL) or it was a **self-review approximation** — so the reader knows exactly what is proven vs. assumed. Never imply the code "passes the scan" for a category no real tool scanned.
      - Any UNRESOLVED errors with diagnosis
      - Rollback instructions: the checkpoint HEAD recorded in step 3 and the command to revert (e.g., `git diff <checkpoint>..HEAD`, `git checkout <checkpoint> -- <path>` for a single file, or `git reset --hard <checkpoint>` for everything — warn that reset discards all changes)

Note: This command assumes a complete task breakdown exists in tasks.md. If tasks are incomplete or missing, suggest running `/rudis.tasks` first to regenerate the task list.
