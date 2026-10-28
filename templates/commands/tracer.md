---
description: Trace a reported symptom layer by layer from its entry point down to the database (read-only SELECT), then explain in one document what the current condition is, what it should be, and why it happened — written so the ops team can follow it. Produces no fix recommendation and no work plan.
---

## User Input

```text
$ARGUMENTS
```

You **MUST** consider the user input before proceeding (if not empty).

## Goal

Take a reported symptom with its identifier — "invoice INV-123 is still pending even
though the payment already came in" — and explain it. Follow the real call chain from
the entry point down to the database, read the actual data with read-only `SELECT`,
then write **one document** that answers three things:

1. **What the current condition is** — facts from the code *and* from the data.
2. **What it should be** — the expectation the user stated, locked in Phase 0.
3. **Why this condition could happen** — the root cause, pinned to the exact hop where
   behaviour diverges from that expectation.

The document is written in two layers so the **ops team** can read the first half
without touching code, and an engineer can verify every claim in the second half.

## Operating Constraints

**STRICTLY READ-ONLY.** Do **not** modify application source code, configuration, or
data. The only thing this command writes is its own report under `docs/trace-reports/`.

**Database access is `SELECT` only.** Every guardrail in Phase 3b is mandatory and
must not be relaxed. There is deliberately no confirmation prompt before querying, so
the guardrails are the only protection there is.

**This command explains; it does not prescribe.** It produces **no fix recommendation
and no work plan** — not in the report, not in the chat summary. "Why this could
happen" states the cause; it does not propose the cure. If the user wants a fix,
point them at `/rudis.redesign` (scoped rework) or `/rudis.operate` (incident resolve)
and stop there.

**Never invent a baseline.** The "should be" axis comes from the user only. If they
did not state it, ask — see Phase 0. Writing an assumed expectation into the report is
the one failure this command exists to prevent.

**Language.** Keep the report's section headings exactly as given in the template
below, and write the narrative in the language the user wrote their request in, so the
ops team who reads it is not handed a document in a language they do not work in.

## Execution Steps

### Phase 0 — Lock the symptom and the expectation

Read `$ARGUMENTS`. Pull out two things:

- **The symptom** — what is observed to be wrong.
- **The identifier(s)** — the order number, invoice number, user id, or reference that
  makes the case concrete. These become the `WHERE` filter in Phase 3.

Then run the **expectation gate**. The "should be" axis has exactly one valid source:
the user.

- **Expectation already stated** in `$ARGUMENTS` → lock it verbatim and go to Phase 1.
- **Not stated, interactive session** → ask for it.
  - Use the `AskUserQuestion` tool **if it is available in this agent**.
  - If it is not available, fall back to the house format: a Markdown table of
    options with one marked `**Recommended:** Option [X] - <reasoning>`, one question
    at a time, exactly as `/rudis.clarify` does.
  - Either way, build the options from what you can already see in the repository —
    the status values the code actually uses, the states the schema allows — not from
    a generic list. A grounded option set is what makes the recommendation worth
    anything.
- **Not stated, non-interactive session** → **stop the run.** Do not guess, and do not
  hang waiting for an answer that cannot arrive. Print a message naming what is
  missing and showing how to re-invoke with the expectation inline, for example:

  ```text
  /rudis.tracer invoice INV-123 is still pending — it should have flipped to PAID
  once the payment webhook landed
  ```

  This is not a theoretical branch: `rudis run tracer` and the TUI Run pane invoke
  `claude --print`, where there is no human on the other end.

### Phase 1 — Identify the stack and confirm the entry point

Read the manifests to learn what this project actually is — `package.json`, `go.mod`,
`composer.json`, `pom.xml`, `pyproject.toml` — and skim the house documents for
conventions that are not inferable from any single file: `README`, `docs/`,
`CLAUDE.md`, `AGENTS.md`, and `docs/brd/` when a semantic map already exists.

Then locate the entry point the symptom actually hangs off: an HTTP route, a scheduled
job, a queue consumer, a webhook receiver, or a CLI command. **Confirm it exists by
reading it** — do not assume a route is there because the stack usually has one.

If the entry point does not exist, that is itself a finding, and quite possibly the
root cause. In that case **skip Phase 2** — there is no call chain to walk — but
**still run Phase 3**: the identifier from the symptom usually still exists in the
database even when the application path does not.

### Phase 2 — Trace layer by layer

From the entry point, follow the real call chain **one hop at a time, reading the
actual code at every hop**. Record for each hop:

- the `file:line`,
- what happens there (validation, transformation, branch, write),
- and whether it is consistent with the locked expectation.

Do not guess that a function exists because the framework usually has one. Some
projects go handler → service → repository → ORM; others stop at controller → service
with the service hitting the database directly. Whatever this project does is a fact to
record, not a deviation to correct.

Keep watching for the **turning point** — the single hop where behaviour first departs
from the expectation. That hop is the spine of the "why" axis in Phase 4.

### Phase 3 — Reach the database

#### 3a. Detect the connection

Look for connection configuration where this project would keep it: `.env` and its
variants, the framework's config directory, and `docker-compose.yml`. Prefer a
**read-only credential** if the configuration exposes one.

If the configuration is missing, the database client is not on `PATH`, or the
connection is refused, **do not fail the run.** Drop into **query-only mode**: write
the `SELECT` statements that *would* have been run into the report without executing
them, and mark the report as unverified on the data side. Never print credentials while
doing so — see rule 6.

#### 3b. Read-only guardrail

Validate every statement against all eight rules **before** executing it. These are not
advisory.

1. **One `SELECT` statement per execution.** Reject a statement containing `;` followed
   by any non-whitespace character. A single trailing `;` is fine.
2. **Write keywords are rejected.** After normalising case and ignoring the contents of
   string literals, reject if any of these appears **as a whole word in statement or
   clause position**: `INSERT`, `UPDATE`, `DELETE`, `DROP`, `ALTER`, `CREATE`,
   `TRUNCATE`, `GRANT`, `REVOKE`, `MERGE`, `REPLACE`, `CALL`, `EXEC`, `COPY`, `LOCK`,
   `SET`, `COMMIT`, `ROLLBACK`, `VACUUM`. A naive substring match would reject valid
   queries, and that is a bug in the guardrail rather than a strict guardrail:
   `OFFSET` contains `SET`, and `REPLACE(col, 'a', 'b')` is an ordinary string function
   in a `SELECT`. A name immediately followed by an opening parenthesis is a function
   call and does not count as a write keyword.
3. **A CTE wrapping a write is rejected.** PostgreSQL permits
   `WITH x AS (INSERT ... RETURNING ...)`. Rule 2 already catches it; it is named here
   so it is not quietly allowed.
4. **A row cap is mandatory** on every statement that fetches data. Default 100 rows.
5. **A statement timeout is mandatory**, applied through the client or an optimizer
   hint — **never** through a separate SQL statement, which rule 2 already forbids.
6. **Credentials are never printed**, to chat or to the report. Only these may appear:
   the engine type, the host (partially masked is fine), and the database name.
7. **Use a read-only credential** when the project's configuration provides one.
8. **A statement violating rules 1–5 is not executed**, and the reason is recorded in
   the report's Bukti Database section — never silently dropped.

Per-engine mechanics for rules 4 and 5, verified against each vendor's current
documentation:

| Engine | Client | Row cap | Statement timeout |
|---|---|---|---|
| PostgreSQL | `psql` | `LIMIT n` | `PGOPTIONS='-c statement_timeout=15000'` — milliseconds. `PGOPTIONS` behaves as the `options` connection parameter, which sends `-c name=value` to the server at connection start |
| MySQL | `mysql` | `LIMIT n` | `SELECT /*+ MAX_EXECUTION_TIME(15000) */ ...` — milliseconds, `SELECT`-only, and the hint must come immediately after the first `SELECT` |
| SQL Server | `sqlcmd` | `SELECT TOP n` | `-t 15` — **seconds**, valid range 1–65534. Without this flag sqlcmd never times out, so it is not optional |
| SQLite | `sqlite3` | `LIMIT n` | none available — `.timeout` is a busy/lock timeout, not a statement timeout. The row cap is the only brake; say so in Batasan Pengambilan Data |

For any other engine, apply the same two rules using whatever its client offers, and
record in the report which mechanism was used.

#### 3c. Collect the evidence

Two stages, both through the guardrail, **in this order**.

1. **Read the catalog first.** Table and column shapes from `information_schema.tables`
   and `information_schema.columns`; constraints from
   `information_schema.table_constraints` and
   `information_schema.key_column_usage`; indexes from the engine's index catalog
   (`pg_indexes` on PostgreSQL, `information_schema.statistics` on MySQL, and the
   equivalent elsewhere). All of this is plain `SELECT`, which keeps the read-only
   promise literal and answers the index question below before any data query runs.
2. **Then fetch the data.** Filter the project's own tables — whose identity you only
   learned in stage 1 — using the identifier from the symptom, with the row cap and the
   timeout applied. Fetch multiple identifiers in **one** query with
   `WHERE col IN (...)`. **Never one query per row.**

Also run a `COUNT` to establish blast radius — how many records share the same wrong
condition — because that number is what the ops layer of the report needs.

### Phase 4 — Assemble the three axes

- **Current condition** — the code facts from Phase 2 combined with the data facts from
  Phase 3. Where they disagree, say so plainly: the code claiming one thing while the
  data shows another is frequently the finding itself.
- **Should be** — the expectation locked in Phase 0, restated as given.
- **Why it could happen** — the cause, pinned to the turning point hop. Distinguish a
  **confirmed** cause (you have the code path and the data to prove it) from a
  **suspected** one. If the evidence does not support a conclusion, say what is missing
  rather than reaching for a plausible story.

Useful cause categories, when one fits: a code defect, a data anomaly predating the
current code, a configuration difference, an incomplete migration, a race between
concurrent writes, a manual intervention, or a requirement that was never implemented
in the first place. Do not force a category that the evidence does not support.

### Phase 5 — Write the document and summarize

1. Get the current date and time **from the shell** — do not guess it.
2. Build the output path: `docs/trace-reports/<YYYY-MM-DD-HHmm>--<slug>.md`, relative
   to the repo root, where `<slug>` is a slugified form of the symptom, e.g.
   `docs/trace-reports/2026-09-16-1430--invoice-inv-123-pending.md`.
3. **Check whether that file already exists. If it does, append a sequence suffix**
   (`-2`, `-3`, …) until you find an unused name. The timestamp has minute resolution,
   so two runs on the same symptom within the same minute would otherwise collide and
   the second would overwrite the first — and re-running right after rewording the
   symptom is the normal way this command gets used. The promise never to overwrite is
   kept by this check, not by the timestamp.
4. Create `docs/trace-reports/` if it does not exist, then write the full report there.
5. Make sure the written Markdown passes markdownlint — ATX headings, a language on
   every fenced block, no bare URLs — because `docs/trace-reports/` is not in any
   `.gitignore`, so a committed report is linted by CI like every other `.md`.
6. In the chat, print a **compact** summary only: the three axes in a few lines each,
   the blast radius number, and the report's path. Do **not** reprint the whole
   document — it is already on disk. Print the full body only if the user asks.

## Behaviour at real data volume

Assume the tables you are querying hold tens of thousands to tens of millions of rows,
not the handful in a development database. Three risks follow, and each has a
mandatory answer:

- **Unbounded result sets** — closed by rule 4. The row cap is mandatory, and any
  truncation is recorded in Batasan Pengambilan Data so a partial result is never read
  as a complete one.
- **Filtering on an unindexed column** — a `WHERE` on an unindexed column of a
  multi-million-row table is a sequential scan that can load a production database.
  Closed by the ordering in Phase 3c: the index catalog is read **before** any data
  query runs, and "this column is not indexed" is itself recorded as a finding. The
  statement timeout then cuts the query short — **except on SQLite**, which has no
  statement timeout, so there the row cap is the only brake and the scan runs to
  completion. Record that limitation when the engine is SQLite.
- **The N+1 shape** — closed by the `WHERE col IN (...)` rule in Phase 3c.

## Failure paths

Handle every one of these; none of them is an excuse to abort the run except where
stated.

| Condition | Required behaviour |
|---|---|
| Expectation not stated, interactive session | Ask, via the Phase 0 gate |
| Expectation not stated, non-interactive session | Stop the run with an instruction and an example invocation |
| Connection configuration not found | Drop to query-only mode; mark the report unverified on the data side |
| Database client not on `PATH` | Same as above |
| Connection refused or authentication failed | Same as above, and never print credentials |
| A statement violates the guardrail | Do not execute it; record the reason in Bukti Database |
| A query times out or is cut by the row cap | Record it explicitly in Batasan Pengambilan Data so a partial result is not read as complete |
| The entry point named by the symptom does not exist | Treat it as the finding itself — possibly the root cause. Skip Phase 2, still run Phase 3 |
| The identifier returns no rows at all | A finding, not an error |
| Duplicate or mutually contradictory rows | A finding; report them as they are |
| The output filename collides with an earlier run | Append a sequence suffix; never overwrite |

## Report template

Write the report using exactly these section headings, in this order.

---

# Laporan Trace — [ringkas gejalanya dalam satu frasa]

- **Tanggal**: [YYYY-MM-DD HH:mm]
- **Perintah**: `/rudis.tracer [$ARGUMENTS]`
- **Identifier yang ditelusuri**: [daftar identifier]
- **Sumber data**: [engine] · [host tersamar] · [nama database] — atau `mode query-only (tidak terkoneksi)`

## Lapis 1 — Untuk Tim Ops

Bagian ini ditulis tanpa istilah teknis, tanpa nama berkas, dan tanpa SQL. Siapa pun
di tim ops harus bisa membacanya sampai selesai tanpa membuka kode.

### Ringkasan untuk Ops

[Satu paragraf: apa persoalannya, dalam bahasa sehari-hari.]

### Apa yang Terjadi Sekarang

[Kondisi nyata saat ini — apa yang benar-benar terjadi pada kasus ini, menurut
sistem dan menurut isi datanya.]

### Apa yang Seharusnya Terjadi

[Ekspektasi yang dinyatakan user, ditulis ulang apa adanya.]

### Kenapa Kondisi Ini Bisa Terjadi

[Sebabnya, dijelaskan sebagai cerita perjalanan data. Sebutkan apakah ini sebab yang
sudah terkonfirmasi atau baru dugaan. Tidak ada saran perbaikan di sini.]

### Seberapa Luas

[Jumlah record yang mengalami kondisi sama, dari hasil hitung di database. Kalau
tidak bisa dihitung, katakan begitu dan sebutkan alasannya.]

### Istilah dalam Dokumen Ini

[Glosarium singkat untuk setiap istilah teknis yang terpaksa muncul di Lapis 1.]

## Lapis 2 — Lampiran Teknis

### Rantai Trace per Lapisan

| Lapisan | Lokasi | Apa yang terjadi di sini |
|---|---|---|
| [entry point / service / repository / …] | `path/file.ext:123` | [perilaku di hop ini] |

### Titik Belok

[Hop persis tempat perilaku mulai menyimpang dari ekspektasi, dengan `file:line` dan
alasan kenapa hop itu yang dianggap titik beloknya.]

### Bukti Database

| Tujuan | Query | Ringkasan hasil | Status |
|---|---|---|---|
| [apa yang ingin dibuktikan] | `SELECT ...` | [hasil ringkas] | dijalankan / ditolak pagar — [alasan] / tidak dijalankan (mode query-only) |

### Struktur Tabel dan Index yang Relevan

[Kolom, constraint, dan index dari tabel yang tersentuh. Tandai eksplisit kolom
filter yang ternyata tidak ter-index.]

### Batasan Pengambilan Data

[Apa yang terpotong batas baris atau batas waktu, mekanisme batas waktu yang dipakai
untuk engine ini, dan apa saja yang karena itu belum terverifikasi. Kalau engine-nya
SQLite, sebutkan bahwa tidak ada batas waktu statement sehingga batas baris adalah
satu-satunya rem.]
