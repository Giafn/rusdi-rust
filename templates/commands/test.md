---
description: Discover existing tests, generate missing tests across layers (unit, UI component/E2E, API/contract), run real + SonarQube-style code quality and security scans, detect/report functional bugs, reproduce VAPT/security findings as failing-then-green regression tests, and persist a timestamped report under docs/test-reports/.
scripts:
  sh: scripts/bash/check-prerequisites.sh --json
  ps: scripts/powershell/check-prerequisites.ps1 -Json
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).
If the user specifies a target (file, folder, module), scope all work to that target only.
If the input names a **security/VAPT finding or a pentest report** (a path to a report, or a
described finding like "IDOR on GET /orders/{id}"), run the VAPT reproduction mode (§5.7):
reproduce each finding as a failing security test, patch it, and leave a green regression test.

**When no target is given (token/time discipline):** on a small project, analyze the whole repository as before. On a **large** one, do not silently read and analyze every source file — default to the highest-value scope (files changed vs the base branch/last tag, plus the CRITICAL-priority files from §3), state clearly in the report that the run was scoped this way and how to widen it, and offer a full-repo pass on request. A deliberate full-repo audit is still available — it just should be an explicit choice on a large codebase, not the accidental default.

## Goal

Ensure the project has adequate test coverage across layers, passes SonarQube-style code quality checks, and surfaces
functional bugs. This command:

1. Discovers existing test infrastructure and frameworks (per layer: unit, UI, API)
2. Identifies coverage gaps across unit, UI (component/E2E), and API/contract layers
3. Generates missing tests following project conventions — unit (§4), UI component + E2E (§4a), API/contract (§4b)
4. Performs SonarQube-style static analysis (code smells, duplication, complexity, maintainability)
5. Scans for functional bugs (logic errors, not just style) and tries to prove each one with a failing test
6. Produces a structured report with findings, a dedicated bug report when bugs exist, and recommendations
7. Persists that report as its own timestamped file under `docs/test-reports/`, so runs can be compared over time

## Operating Constraints

- **Generate tests (and the report file) only** — do NOT modify production/source code without explicit user approval (see Remediation, step 6a). Writing test files and the persisted report under `docs/test-reports/` is not covered by that approval gate — those are this command's normal output, not production code.
- **Follow existing patterns** — match the project's test framework, naming conventions, file structure, and assertion style
- **No fabricated results** — only report issues actually found in the code
- **Minimal mocking** — prefer real implementations; mock only external dependencies (APIs, databases, filesystem)
- **No over-testing** — skip generated code, vendor files, config files, migration files, and type-only files
- **Compress noisy tool output, never lossy content** — when capturing build/test-run output, dedupe repeated pass/skip lines (e.g. "142 passed" instead of listing each) and collapse verbose framework banners. Never summarize, truncate, or dedupe a failing test's assertion message or stack trace, and never skim a source file instead of reading it fully in Step 4 or Step 5 — those are the basis for real decisions and must stay verbatim.

## Execution Steps

### 1. Initialize Context

Run `{SCRIPT}` from repo root and parse JSON for FEATURE_DIR and AVAILABLE_DOCS list. All paths must be absolute.

### 2. Discovery Phase

Detect test infrastructure.

**Test Frameworks** — look for configuration and dependencies:

- JavaScript/TypeScript: Jest, Vitest, Mocha, Playwright, Cypress (check package.json, vitest.config.*, jest.config.*, tsconfig.json)
- Python: pytest, unittest (check pyproject.toml, setup.cfg, conftest.py, tox.ini)
- Java/Kotlin: JUnit, TestNG (check pom.xml, build.gradle)
- Go: built-in testing (check *_test.go)
- Rust: built-in testing (check #[cfg(test)] blocks)
- .NET: xUnit, NUnit, MSTest (check *.csproj)

**Test Locations** — find where tests live:

- `tests/`, `test/`, `__tests__/`, `spec/`
- Co-located: `*.test.*`, `*.spec.*` alongside source
- `src/**/__tests__/`

**Test Run Command** — identify how to run tests:

- npm/yarn/pnpm scripts (test, test:unit, test:coverage)
- pytest / python -m pytest
- go test ./...
- cargo test
- dotnet test

**Test layer infrastructure** — beyond unit, detect what exists for each layer (this decides what §4/§4a/§4b can generate without adding heavy new tooling):

- **UI component**: React/Vue Testing Library, `@testing-library/*`, Vitest/Jest with jsdom, Storybook + test-runner, Enzyme.
- **UI E2E**: Playwright (`playwright.config.*`), Cypress (`cypress.config.*`), WebdriverIO, Selenium.
- **API / contract**: `supertest`/`pactum` (HTTP integration), Pact (`@pact-foundation`), REST Assured (Java), schemathesis / Dredd (OpenAPI-driven), Postman/newman collections; and whether machine-readable contracts exist (`contracts/` — OpenAPI/TMF schemas, `*.proto`, GraphQL SDL).

Record per layer: **present** (framework + version + where its tests live) or **absent**. An absent layer is not a silent skip — §4a/§4b report it and offer to scaffold it (adding a dev dependency needs explicit user approval; never add one silently).

Record:

- Framework name and version (per layer: unit / UI-component / UI-E2E / API-contract)
- Test directory structure
- Naming convention (e.g., `*.test.ts`, `*.spec.ts`, `*.cy.ts`, `*.e2e.ts`, `*_test.go`, `test_*.py`)
- Number of existing test files (per layer)
- Available test scripts/commands (per layer — `test`, `test:unit`, `test:e2e`, `test:component`, `test:contract`)

### 3. Coverage Gap Analysis

Identify source files WITHOUT corresponding test files:

1. List all source files (excluding vendor, node_modules, dist, build, generated, migrations, types-only)
2. For each source file, check if a matching test file exists based on the project's naming convention
3. Classify untested files by priority:
   - **CRITICAL** — business logic, services, controllers, handlers, core domain
   - **HIGH** — utilities, helpers, data transformations, validators
   - **MEDIUM** — components (UI), hooks, middleware
   - **LOW** — constants, config, types, interfaces, models (data-only)

Output a gap summary table:

| Priority | Source File | Expected Test File | Status |
|----------|------------|--------------------|--------|
| CRITICAL | src/services/order.ts | src/services/order.test.ts | MISSING |
| HIGH | src/utils/format.ts | src/utils/format.test.ts | MISSING |

Then identify gaps for the **other two test layers** (only for layers the project actually has, or should have per the feature — do not invent a UI layer for a headless service):

- **UI gap** — for each page/component in the frontend (cross-reference the spec's **UI/UX & Screens** and the routing/page directory), note whether it has a component test and whether the user flows it participates in have an E2E test. Prioritize by the screen's importance (a primary flow like create/checkout/approve is CRITICAL; a static page is LOW).
- **API/contract gap** — for each endpoint the backend exposes (and each operation in `contracts/`), note whether it has an integration test (request → status/response) and, when a machine-readable contract exists, a contract test asserting the response matches the schema. Prioritize by the operation (state-changing/authz-sensitive endpoints CRITICAL).

Extend the gap table with a **Layer** column (`unit` / `ui-component` / `ui-e2e` / `api` / `contract`) so all three layers are visible in one place.

### 4. Unit Test Generation

For each missing test (ordered by priority, CRITICAL first):

1. Read the source file completely
2. Identify all exported functions, classes, methods
3. For each testable unit, generate tests covering:
   - **Happy path** — expected input produces expected output
   - **Edge cases** — empty input, null/undefined, boundary values, zero, negative numbers
   - **Error paths** — invalid input, thrown exceptions, error returns
   - **Type boundaries** — type coercion, optional parameters, default values

**Test file template** (adapt to project framework):

```text
// Follow the project's existing import style
// Group tests by function/method using describe blocks
// Use descriptive test names: "should [expected behavior] when [condition]"
// Arrange-Act-Assert pattern
// Minimal setup — no unnecessary mocking
```

Rules:

- One test file per source file
- Match the project's existing test structure and style exactly
- Use the same assertion library as existing tests
- Import from relative paths matching existing patterns
- Do NOT test private/internal functions unless the project convention does so
- Do NOT add dependencies not already in the project

### 4a. UI Test Generation (component + E2E)

Run this only when the project has a frontend. Use the layer infrastructure recorded in §2.

**Component tests** (for the UI-component layer, when its framework is present):

- One test per component with meaningful logic/states — render it and assert the states the spec requires (loading / empty / error / populated), conditional rendering, and prop/variant behavior.
- Drive interactions the way a user does (Testing Library: query by role/label/text, `userEvent` for clicks/typing) — not by reaching into internals or component instance state.
- Assert accessible output (roles, labels, visible text), not implementation detail (class names, internal state).
- Mock only the network/data boundary (the service layer or `fetch`), never the component under test.

**E2E tests** (for the UI-E2E layer, when Playwright/Cypress is present):

- Generate one E2E spec per **primary user flow** from the spec's UI/UX & Screens (e.g. create → list → detail → approve), not per page — E2E is expensive; cover the critical journeys, not every widget.
- Use stable selectors (role/`data-testid`), wait on state (not fixed sleeps), and keep each spec independent (its own setup/teardown, no cross-test order dependency).
- Prefer the framework's network stubbing for deterministic runs; hit a real test backend only when the flow's value is the integration itself.

### 4b. API / Contract Test Generation

Run this when the project exposes an HTTP/RPC API. Use the layer infrastructure recorded in §2, and treat `contracts/` (OpenAPI/TMF schema, `*.proto`, GraphQL SDL) as the source of truth for shapes when it exists.

**Integration tests** (endpoint behavior, when an HTTP test tool like `supertest`/`pactum`/REST Assured is present):

- One test group per endpoint/operation covering: success (correct status + response body), input validation (400 on bad input), authentication (401 when unauthenticated), authorization (403/404 when out of scope — matching `security-standard.md`'s deny rules), and the notable error paths.
- Field names/types/casing in assertions come from the contract, not guessed — a drift here is exactly what the field-parity gate catches at build time, so tests must encode the contract shape.

**Contract tests** (when a machine-readable contract exists):

- Assert each response validates against the contract schema (status codes, required fields, types, enums) — schema-validate the payload rather than hardcoding a full literal, so the test tracks the contract.
- If a contract-testing tool is present (Pact for consumer-driven, schemathesis/Dredd for OpenAPI-driven), generate in that tool's form; otherwise generate schema-validation assertions in the project's existing API test framework.

**Honesty & no silent heavy deps (applies to §4a and §4b):** generate tests for a layer only when its framework is already present. When a needed layer is **absent**, do NOT silently add Playwright/Cypress/Pact/etc. — report the gap, and offer to scaffold the minimal setup (config + one example spec + a `test:*` script) as a separate, explicitly-approved step. Never claim UI/API coverage exists for a layer that has no runner. Generated UI/API tests are verified in §6 like any other test — a generated test that can't run yet (missing runner) is reported as PENDING SETUP, not as passing.

### 5. SonarQube Compliance Analysis

Analyze all source files for the following quality dimensions. The thresholds in
§5.1–§5.4 are the canonical values from `templates/sonarqube-standard.md` (§2) —
read that file as the source of truth (and to catch any project-specific profile
layered on top); the numbers restated below must stay aligned with it.

#### 5.0 Run the real scanner when the project is wired for SonarQube

The static checks in §5.1–§5.6 approximate the gate; the **only** way to prove
code will pass the project's actual SonarQube gate is to run its scanner against
its own quality profile. Before the static pass, detect whether this project is
already wired for SonarQube and, if so, prefer the real scan:

1. **Detect config** — look for `sonar-project.properties`, a `sonar.*` block in
   `pom.xml`/`build.gradle`, a `sonar` script in `package.json`, or a Sonar stage
   in `.gitlab-ci.yml`/`.github/workflows/*`. Record the `sonar.host.url` and
   project key if present.
2. **Detect a runner** — `sonar-scanner` on PATH, `mvn`/`gradle` with the Sonar
   plugin, or `npx sonar-scanner`.
3. **If both exist**: tell the user you can run the real analysis and ask ONCE
   for confirmation (the scan uploads to their server and needs a token — never
   hardcode or echo it; rely on the environment/CI variable the project already
   uses, e.g. `SONAR_TOKEN`). On yes, run it with the quality-gate wait enabled
   (`-Dsonar.qualitygate.wait=true` for scanner/Maven, or
   `sonar.qualitygate.wait=true`) so the command returns the **actual** gate
   PASS/FAIL, and record that result verbatim in the report's Quality Gate
   section. A real FAIL lists the exact failing conditions — treat each as a
   finding and route production-code fixes through Step 6a.
4. **If config or runner is missing, or the user declines**: state clearly in the
   report that the gate result is a **static approximation, not a verified scan**,
   then proceed with §5.1–§5.6. Do not claim the code "passes SonarQube" on the
   strength of the static pass alone — only a real scan (or the project's CI)
   proves that.

#### 5.1 Code Smells

- Functions/methods exceeding 40 lines
- Files exceeding 300 lines
- Deeply nested conditionals (> 3 levels)
- Functions with > 4 parameters
- Unused variables and imports
- Dead code (unreachable branches)
- Magic numbers and strings (should be constants)
- Console.log / print statements left in production code

#### 5.2 Duplication

- Repeated code blocks (> 6 similar lines across files)
- Copy-paste patterns in adjacent functions
- Duplicated logic that could be extracted to shared utilities

#### 5.3 Complexity

- Cyclomatic complexity > 10 per function
- Cognitive complexity > 15 per function
- Deeply nested callbacks or promise chains
- Complex boolean expressions (> 3 conditions)

#### 5.4 Maintainability

- Missing or misleading function/variable names
- Inconsistent naming conventions within the codebase
- Long parameter lists without destructuring or options objects
- Tight coupling between modules (circular or excessive imports)
- God objects / god functions (doing too many things)

#### 5.5 Security

**Run real scanners first, self-review second.** Like §5.0's SonarQube scan, a real SAST/secret/dependency scan proves what a read-through only guesses — and it is the OWASP side of the external gate a project must pass. When the tools fit the project's stack, run them (prefer a project-installed tool; else a zero-install runner — `npx`, `pipx run`, `docker`), scoped to `$ARGUMENTS` if it narrowed the run, and route findings into the report at `security-standard.md` §1 severities:

- **SAST (OWASP):** `semgrep --config p/owasp-top-ten` (any stack), or the language-native analyzer — JS/TS `eslint-plugin-security`, Python `bandit`, Go `gosec`, Java SpotBugs + `find-sec-bugs`.
- **Secrets:** `gitleaks detect` (or `trufflehog`) — a real hit is a BLOCKER; the secret must be removed AND rotated.
- **Dependencies / vulns:** `osv-scanner` on the lockfile, `npm audit --omit=dev`, `pip-audit`, or `govulncheck` — a known Critical/High CVE is a BLOCKER.

Record in the report, per category, whether a **real tool ran** (name + result) or it was a **self-review approximation** — never claim the code "passes OWASP" for a category no tool scanned. If no scanner can run, fall back to the self-review below and label it as approximation.

Then scan against `templates/security-standard.md` — its ❌ DON'T patterns per category (access control, misconfiguration, supply chain, crypto, injection, insecure design, auth, integrity, logging, exception handling) and its §14 self-review checklist. At minimum, flag:

- Hardcoded credentials, tokens, or secrets
- SQL string concatenation or template literals (injection risk); unparameterized dynamic identifiers (sort/table names) with no allowlist
- Unsanitized user input in templates or `innerHTML`/`dangerouslySetInnerHTML` (XSS risk)
- Use of eval() or equivalent dynamic execution; shell string built from user input (command injection)
- Insecure randomness (`Math.random()`/`rand()`) for tokens, secrets, or salts
- Weak/deprecated crypto (MD5/SHA-1 for passwords, DES/3DES/RC4, ECB mode, bcrypt cost < 10)
- Missing authorization check, or `userId`/`tenantId` taken from client input instead of the verified session
- Silent `catch` that swallows a security-relevant error, or a stack trace leaked to the client
- A dependency added/updated this task that hasn't been checked per the standard's §4 (freshness, known CVEs, typosquatting, license)
- **A deprecated, discontinued, yanked, or archived dependency, or one targeting an EOL runtime.** Detect this by running §4 step 2's tool query against the project's lockfile — `npm view <pkg> deprecated` / `npm outdated`, `pip-audit`, `osv-scanner`, endoflife.date for the runtime — and reading the output; do NOT judge a package "current" from training-data memory (a knowledge cutoff misses anything abandoned after that date). Scan the *whole* lockfile, including inherited scaffold/boilerplate/profile dependencies, not just what this task touched — an obsolete inherited package is the common reason code passes local review yet fails an external gate. A confirmed deprecated/discontinued/EOL dependency is a **BLOCKER** (it will fail an external security/SonarQube gate). If the tool can't run in this session, report the dependencies as UNVERIFIED rather than clean.

#### 5.6 Bug Detection (Functional Correctness)

Distinct from 5.1-5.5 above: those are style/maintainability signals, this is about code that
produces **wrong behavior**. Scan for:

- Off-by-one errors in loops, slices, pagination, or array indexing
- Null/undefined access on a path that can genuinely be null/undefined at runtime
- Inverted or malformed boolean logic (wrong operator, swapped `&&`/`||`, broken De Morgan negation)
- Race conditions / unguarded shared-state mutation across concurrent or async paths
- Unhandled promise rejections, or async errors that are silently dropped
- Resource leaks (unclosed handles, connections, listeners, timers, subscriptions)
- Error swallowing (empty `catch`, or a `catch` that discards the real error/cause)
- Type-coercion bugs (loose equality across mismatched types, unintended truthy/falsy on `0`/`""`/`NaN`)
- Loops/recursion that can run one too many/few times, or never terminate
- Copy-paste bugs (a duplicated block that still references the original's variable/field)

For each candidate, in order:

1. State the concrete failure scenario: what input or state triggers it, and what the wrong
   outcome is (not just "this looks risky").
2. Try to write a minimal reproducing test (Prove-It pattern — it must FAIL against the
   current code).
3. If the test fails as expected -> mark it **CONFIRMED** and hand it to Step 6a's
   remediation flow (Production-code failures bucket), exactly like a bug caught during
   Step 6 verification.
4. If a reproducing test can't be written (missing infra, timing-dependent, needs manual
   verification) -> mark it **SUSPECTED**, state why it couldn't be confirmed, and still
   include it in the Bug Report.

Never label a candidate CONFIRMED without a failing test that proves it — that is the
fabrication line this command must not cross. Static reasoning alone only earns SUSPECTED.

#### 5.7 VAPT / security-finding reproduction (when a finding or pentest report is provided)

Run this mode when `$ARGUMENTS` names a security/VAPT finding or points to a report
(e.g. "reproduce VAPT: broken access control on GET /orders/{id}", or
`docs/vapt/pentest-2026.md`). It turns a vulnerability finding into the same evidence-first
loop §5.6 uses for bugs — **finding → PoC/failing test that reproduces it → patch → green
regression test** — so every security fix is proven, not asserted.

**Scope & safety (STRICT — this is defensive reproduction, not offense):**

- Reproduce the project's **own** reported finding against its **own** code in a **test
  environment** only. Never run an exploit against a live/production system or any third-party
  target, and never generate working offensive tooling aimed at an external system.
- The PoC lives as a **test in the repo suite** (fixtures/payloads in test data), not as a
  standalone attack script. Keep any sample payload minimal — enough to prove the flaw, no more.
- If a finding cannot be reproduced safely in a test (needs a live target, manual steps, or
  infra not present), mark it **SUSPECTED**, explain why, and attach the proposed fix plus
  manual verification steps rather than fabricating a CONFIRMED.

**Per finding, in order:**

1. **Parse & classify** — map the finding to its OWASP category + CWE and to a severity via
   `security-standard.md` §1 (Critical/High → BLOCKER). Locate the affected code (endpoint,
   handler, query, component, dependency).
2. **Reproduce as a failing security test** — write a test (usually at the API/integration
   layer of §4b, or the component layer of §4a) that exercises the vulnerable path and asserts
   the **secure** outcome, so it FAILS against the current vulnerable code. Examples: an
   unauthenticated request asserts 401 but currently gets 200; an IDOR request for another
   tenant's resource asserts 404 but currently returns 200 with data; an injection payload
   asserts rejection/escaping but currently leaks or executes. **CONFIRMED** only when the test
   fails against current code — that failure is the proof the vulnerability is real.
3. **Patch** — apply the `security-standard.md` ✅ DO fix for the category (parameterized query,
   explicit authz check, tenant/owner `WHERE` scoping, output encoding, algorithm/whitelist fix,
   etc.) through Step 6a's remediation gate (production-code changes need the explicit approval
   described there).
4. **Regression green** — re-run: the security test now passes (the vuln is closed). Keep the
   test **permanently** as a regression guard, named for the finding/CWE, so the vulnerability
   can never silently return.
5. **Report** in a dedicated VAPT Reproduction table: finding, OWASP/CWE, severity, reproduced
   (test name), patched (Fixed/Open/Blocked per §6a), regression status.

Severity classification (applies to 5.1-5.7, including bugs and reproduced findings):

- **BLOCKER** — Critical/High-severity security vulnerability (per `templates/security-standard.md` §1), will fail SonarQube quality gate. A Medium/Low-severity security finding is MAJOR/MINOR instead, per that same mapping.
- **CRITICAL** — bug-prone pattern (non-security), high risk of runtime failure
- **MAJOR** — significant maintainability or readability issue
- **MINOR** — style inconsistency or minor improvement opportunity
- **INFO** — suggestion for improvement, not a violation

### 6. Verification

If a test run command was identified in Step 2:

1. Run the existing test suite to establish baseline (capture pass/fail count)
2. After generating new tests, run the full suite again to verify new tests pass
3. If any generated test fails, fix it immediately — never leave broken tests

If no test command is available, skip this step and note it in the report.

### 6a. Remediation (do not leave errors hanging)

Failures fall into two buckets — handle both to completion. This bucket now has two sources:
failures from running the (existing + generated) suite in Step 6, and bugs marked **CONFIRMED**
in Step 5.6 by a reproducing test.

- **Test-side failures** (wrong assertion, bad mock, wrong import in a generated test):
  fix the test yourself immediately and re-run, up to 3 attempts per distinct error.
- **Production-code failures** (the test correctly exposes a real bug): do NOT silently
  skip them. Present a concrete fix per failure (file, line, root cause, exact change)
  and ask ONCE: "Apply these N production fixes? (yes/no)". On "yes", apply them,
  re-run the suite, and iterate (max 3 rounds) until green or genuinely blocked.
  On "no", record them as OPEN BUGS in the report with the proposed fixes attached.

Never end the command with a vague "some errors remain" — every failure is either
fixed, an OPEN BUG with a ready-to-apply fix, or explicitly blocked with a reason. Every
CONFIRMED bug from 5.6 must resolve to Fixed, Open (declined), or Blocked — never left
unmentioned in the report.

### 7. Report

Generate the final report (template below), then persist it and present it:

1. Get the current date/time (e.g. via the shell) — do not guess it.
2. Determine the output path: `docs/test-reports/<YYYY-MM-DD-HHmm>.md`, repo-root relative.
   If `$ARGUMENTS` scoped this run to a specific path, append a slugified version of it, e.g.
   `docs/test-reports/2026-07-20-1430--src-services.md`.
3. Create `docs/test-reports/` if it doesn't exist yet, then write the full report (including
   the header fields below) to that file. Never overwrite a prior report — each run gets its
   own timestamped file, so the directory accumulates a history that can be diffed/compared
   across runs.
4. In the chat response, print a COMPACT summary — the gap/severity table, the bug/finding
   table, and the report's file path — NOT the full report body (it is already persisted to the
   file in step 3). Reprinting the whole report duplicates a large emission; the tables + path
   give full visibility while keeping the run token-thrifty. Print the full body only if the user
   explicitly asks for it.

Report template:

---

# Test Report

- **Scope**: [whole repository | the path from $ARGUMENTS]
- **Date**: [YYYY-MM-DD HH:mm]
- **Command**: `/rudis.test [$ARGUMENTS]`

## Test Infrastructure

Per layer — mark each present (framework + version + location) or absent:

- Unit: [name + version | absent]
- UI component: [name + version | absent]
- UI E2E: [name + version | absent]
- API / contract: [name + version | absent] (contracts/: [present | none])
- Test directory: [path] · Run commands: [unit / e2e / component / contract]
- Existing test files: [count per layer]

## Coverage Gap Analysis

| Layer | Priority | Gaps | Tests Generated |
|-------|----------|------|-----------------|
| unit | CRITICAL/HIGH/MEDIUM/LOW | [n] | [n] |
| ui-component | … | [n] | [n] |
| ui-e2e | … | [n] | [n] |
| api | … | [n] | [n] |
| contract | … | [n] | [n] |
| **Total** | | **[n]** | **[n]** |

For an absent layer, write "absent — not generated" and (if the feature warrants it) note the scaffold offer, rather than a zero that reads like full coverage.

## Generated Tests Summary

List each generated test file with:

- File path and **layer** (unit / ui-component / ui-e2e / api / contract)
- Number of test cases
- What it covers (functions/methods, component states, user flow, or endpoint/operation)
- Run status: passing / PENDING SETUP (runner not yet installed)

## Bug Report

Include this section only if Step 5.6 or Step 6a surfaced at least one CONFIRMED or SUSPECTED
bug. If none were found, write a single line instead: "No bugs found." — do not render an
empty table.

| Status | Severity | File | Description | Evidence | Resolution |
|--------|----------|------|--------------|----------|------------|
| CONFIRMED | CRITICAL | path:line | what goes wrong and when | failing test name | Fixed / Open (declined) / Blocked |
| SUSPECTED | MAJOR | path:line | what goes wrong and when | why it couldn't be reproduced | Needs manual verification |

- **CONFIRMED** — proven by a reproducing test that fails against current code (Step 5.6 or
  Step 6). Resolution must match what actually happened in Step 6a: Fixed, Open (user declined
  the fix, proposed patch attached below), or Blocked (with reason).
- **SUSPECTED** — identified by reasoning/static analysis but not yet proven by a failing
  test. Never presented as fact — flag it for manual review, and include the reproducing test
  you attempted (and why it fell short) so a human can finish the proof.

For every Open or Blocked CONFIRMED bug, attach the proposed fix (file, line, root cause, exact
change) directly under its row so it's ready to apply later.

## VAPT Reproduction

Include this section only when §5.7 ran (a security/VAPT finding was provided). Otherwise omit it.

| Finding | OWASP / CWE | Severity | Reproduced (test) | Patched | Regression |
|---------|-------------|----------|-------------------|---------|------------|
| IDOR on GET /orders/{id} | A01 / CWE-639 | BLOCKER | orders.idor.test.ts (failed → | Fixed | ✅ green |

- **Reproduced** — name the failing security test that proved the vuln against current code (or SUSPECTED + why it couldn't be reproduced safely).
- **Patched** — Fixed / Open (declined, proposed fix attached) / Blocked (reason), per Step 6a.
- **Regression** — ✅ green (security test passes post-fix and stays as a permanent guard) or ✗ (still failing / not yet fixed).

## SonarQube Compliance

| Severity | Count | Category |
|----------|-------|----------|
| BLOCKER | [n] | [breakdown] |
| CRITICAL | [n] | [breakdown] |
| MAJOR | [n] | [breakdown] |
| MINOR | [n] | [breakdown] |
| INFO | [n] | [breakdown] |

### Findings Detail

For each finding (grouped by severity):

- **File**: path:line
- **Rule**: category (e.g., code-smell / duplication / complexity / security)
- **Severity**: BLOCKER / CRITICAL / MAJOR / MINOR / INFO
- **Description**: what was found
- **Recommendation**: how to fix it

## Quality Gate

- **Verification method**: [Real SonarQube scan (`sonar.qualitygate.wait`) | Static approximation — no scan run]
- **Real gate result** (only if §5.0 ran a scan): [PASS | FAIL — list failing conditions]
- [ ] No BLOCKER issues
- [ ] No CRITICAL issues
- [ ] No CONFIRMED bugs left unresolved (each is Fixed, Open with a proposed fix, or Blocked)
- [ ] Unit test coverage: [estimated % based on files with tests / total testable files]
- [ ] All generated tests pass

> If the method above is "Static approximation", this section reflects the
> static checks only — it is NOT proof the project's SonarQube gate passes.

## Recommendations

Prioritized list of next actions to improve quality.

---

## Rules

- Never modify production/source code without the explicit approval described in step 6a — default is test files only
- Never fabricate findings — report only what exists in the code
- Never mark a bug CONFIRMED without a reproducing test that actually fails against current code — unconfirmed suspicions are reported as SUSPECTED, never upgraded to CONFIRMED on reasoning alone
- Never mark a quality gate as passed without evidence
- If test generation cannot proceed (no framework detected, no source files), stop and explain why
- Always verify generated tests compile and pass before reporting success
- Always persist the report to `docs/test-reports/<timestamp>.md` as its own new file — never overwrite a previous run's report, and never skip writing it even if no bugs/findings turned up
