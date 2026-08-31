# lxlsx

A contract-verified, agent-first **process registry for Excel workbooks** — the Rust successor
to the spreadsheet half of `budget-danialrami`.

`lxlsx` reads a `finance.db` (SQLite, v2 schema), builds a typed intermediate document, renders
it to a brand-styled `.xlsx`, and **verifies** the result before reporting success — "proven,
not exited 0."

## The shape (House of Process Registries)

| noun | where |
|------|-------|
| kernel | `src/` — typed `Snapshot`, SQLite reader, xlsx renderer, read-back verifier |
| process | `processes/<slug>/manifest.json` + its compiled stage |
| registry | `processes/` folder-per-entry + generated `index.json` |
| verification | `lxlsx verify` — structural contract + totals reconciliation |

This is a **six-noun registry** where the recipe layer is single-hop (`read → render →
verify` per process); cross-process composition is a declared future, not v1.

## Install

```bash
cargo build --release
./target/release/lxlsx --help
```

## Usage

```bash
lxlsx snapshot --db data/finance.db --out data/exports/finances.xlsx   # the review control panel
lxlsx run snapshot --db data/finance.db --json                          # same, JSON envelope
lxlsx list [--json]                                                     # registered processes
lxlsx verify --xlsx data/exports/finances.xlsx                          # read-back + structural verify
lxlsx index [--check]                                                   # regenerate / check the registry index
```

## Exit codes

`0` SUCCESS · `2` USAGE · `3` DB_UNAVAILABLE · `4` PROCESS_NOT_FOUND · `5` CONTRACT_VIOLATED
(`1` is reserved).

## Golden parity

`tests/golden_parity.rs` renders an in-memory fixture and asserts the six-block layout —
LIVING / SUBSCRIPTIONS / CREDIT / DEBIT / WORK INCOME / ALL EXPENSES SINCE LAST TIME — plus
totals reconciliation. The three-verdict model (verified / violated) gates every run; unverified
output is not reported as success.

See `SPEC.md` for the full contract and `docs/units/` for the per-unit contracts.
