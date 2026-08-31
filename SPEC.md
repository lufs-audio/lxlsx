# lxlsx — SPEC

**A LUFS Primitive: an agent-first, contract-verified process registry for Excel workbooks.**

`lxlsx` is the Rust successor to the spreadsheet half of `budget-danialrami` (specifically
`scripts/presentation/snapshot_sheet.py`, PR #10), rebuilt in the House of Process Registries
shape. It reads a `finance.db` (SQLite, v2 schema), builds a typed intermediate **snapshot
document**, renders it to a brand-styled `.xlsx`, and **verifies** the result against a
declared contract — "proven, not exited 0."

The **snapshot** (Daniel's review control panel: LIVING / SUBSCRIPTIONS / CREDIT / DEBIT /
WORK INCOME / ALL EXPENSES SINCE LAST TIME) is process #1. The `scripts/analysis/` reports
(burn rate, category pie, credit payoff, spending by category, …) are processes #2..N — the
plurality that justifies building this as a registry from the jump.

## The six nouns

| Noun | lxlsx instance |
|------|----------------|
| **kernel** | `src/` library: the typed snapshot document, the SQLite reader, the xlsx renderer, the read-back verifier, exit codes, the JSON envelope |
| **process** | one authored unit in `processes/<slug>/` with a `manifest.json` (contract) + a compiled `src/process/<slug>.rs` |
| **registry** | `processes/` folder-per-entry + generated `processes/index.json` with `--check` |
| **recipe** | `lxlsx run <slug>` composes kernel stages (read → model → render → verify) per process; cross-process composition is a declared future, not v1 |
| **handoff** | the typed `Snapshot` document (serde) — the one model read by DB-renderer and verifier alike, so they cannot diverge |
| **verification** | `lxlsx verify`: read the emitted `.xlsx` back (calamine), assert block structure + headers + totals reconcile to the model |

## Golden oracle (load-bearing)

Daniel supplied `finances_2026-08-31.zip` — a Google Sheets HTML export (`Sheet1.html` +
`resources/sheet.css` + the Chase screenshot) as the **proven-correct target**. The port
must match its **structure and content semantics** exactly (six blocks, exact header labels,
ordinal due-date style, Title-case categories, the `total:` / `statements:` / `last met:`
footer). Two documented, deliberate departures from a literal byte-match:

1. **Styling is the LUFS brand layer, not Google's default.** The golden export is unstyled
   (Arial 8pt, black-on-white). `snapshot_sheet.py`'s entire value — and the craft-grounding
   that makes this *ours* rather than a generic xlsx dumper — is the teal-header / Space
   Mono / Public Sans / Host Grotesk palette. We reproduce the golden's *structure* exactly
   and apply the brand *styling* on top. *(Reversible: if Daniel wants pure un-styled parity,
   the palette is a single constant block to strip.)*
2. **Data values come from the input DB, not the golden file.** The golden's specific dollar
   figures are Daniel's live data (private). *Content fidelity* === "whatever the DB says,
   laid out exactly like the golden." Verification is **structural + deterministic + totals-
   reconciling** against a checked-in fixture, not a byte-diff against his private file.
   Live byte-fidelity is achieved by pointing `lxlsx` at the real `finance.db`.

## Golden structure (extracted 2026-08-31)

Six blocks, left panel (A–F) and right panel (G–L):

1. **LIVING** (A1:C): `LIVING | AMT | MONTHLY DUE DATE` — rows of `{name, amount, due}` where
   `due` is an ordinal (`17th`) or `end of month` when no day.
2. **CREDIT ACCOUNT INFO** (G1:L): `CREDIT ACCOUNT INFO | BALANCE | MINIMUM PAYMENT |
   PAYMENT DUE | LIMIT | APR` — utilization/apr rendered as percent.
3. **DEBIT ACCOUNT INFO** (G–): `DEBIT ACCOUNT INFO | BALANCE | NOTES` — descriptive account
   names, signed balances.
4. **WORK INCOME** (G–): `WORK INCOME | BALANCE | NOTES` — may be empty.
5. **SUBSCRIPTIONS** (A–F): `SUBSCRIPTIONS | AMT | FREQUENCY | DUE | DUE DATE | Work
   Expense?` — `FREQUENCY` = Monthly/Annual; `DUE` = authored ordinal/month-day label
   (`8th`, `May 5th`); `DUE DATE` = computed next occurrence (ISO); `Work Expense?` = `x`.
6. **ALL EXPENSES SINCE LAST TIME** (G–K): `ALL EXPENSES SINCE LAST TIME | BALANCE | DATE
   CHARGED | CATEGORY | NOTES` — then footer `total: $X`, `statements: link`, `last met: D`.

Layout is **dynamic** (blocks start after the prior block; columns A–F vs G–L run in
parallel), matching `snapshot_sheet.py` — the golden's specific row numbers (7, 13, 17) are
where the current data lands, not hard-coded geometry.

## Stack

- **read**: `rusqlite` (bundled SQLite) — `finance.db` v2 schema.
- **write**: `rust_xlsxwriter` (by the XlsxWriter author; full formatting, formulas, charts).
- **verify**: `calamine` (read-back the emitted xlsx).
- **CLI**: `clap` (derive); **typed doc**: `serde`/`serde_json`; **dates**: `chrono`.

`polars` is deliberately **deferred**: the v1 analysis surface is SQL + plain-Rust
aggregation, which is simpler, compiles faster, and has no dataframe need yet. Polars becomes
the kernel ingredient the moment a process needs pivots/joins across sources. *(Flagged,
reversible on Daniel's word.)*

## CLI (name `lxlsx`, 5 chars — compliant)

```
lxlsx snapshot --db <path> --out <xlsx> [--json]      # render the control panel
lxlsx run <slug> [--db <path>] [--out <xlsx>] [--json]  # run any registered process
lxlsx list [--json]                                 # list registered processes
lxlsx verify --xlsx <path> [--json]                 # read-back + structural verify
lxlsx index [--check]                               # regenerate / check registry index
```

## Exit codes (fixed floor + domain)

| code | meaning |
|------|---------|
| `0` | SUCCESS |
| `1` | reserved, never assigned |
| `2` | USAGE (bad args/conflict, before work) |
| `3` | DB_UNAVAILABLE (the finance.db input is missing/unreadable) |
| `4` | PROCESS_NOT_FOUND (requested slug not in registry) |
| `5` | CONTRACT_VIOLATED (output failed verification) |

JSON envelope: `{"status":"success","data":{…}}` / `{"status":"error","code":N,"message":"…"}`,
with `code` === the process's numeric exit code.

## Definition of done

A fresh checkout builds (clippy `-D warnings`, no `unwrap`/`expect`/`panic` in non-test
code), tests green, and `lxlsx snapshot` on the checked-in fixture emits an `.xlsx` that
`lxlsx verify` proves structurally identical to the golden's six-block layout, with totals
reconciling to the model — and the process registry `index.json` is generated and `--check`
-consistent.

See `docs/units/` for the per-unit contracts.
