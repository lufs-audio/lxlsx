# Unit 03 — XLSX renderer (Snapshot → .xlsx)

## Objective
Implement `src/render.rs`: render a `Snapshot` to a brand-styled `.xlsx` matching the golden
six-block layout, using `rust_xlsxwriter`.

## Context
Ports `snapshot_sheet.py::render_xlsx()` geometry and the LUFS palette (INK `111111`, CREAM
`FBF9E2`, TEAL `78BEBA`, MUTED `8A8A82`, HAIR `D9D7C8`), with Space Mono headers / Public
Sans body / Host Grotesk totals. Golden layout (see SPEC.md): LIVING (A–C), CREDIT (G–L),
DEBIT (G+H), WORK INCOME (G+H), SUBSCRIPTIONS (A–F), ALL EXPENSES SINCE LAST TIME (G–K) with
`total:`/`statements:`/`last met:` footer. Blocks are **dynamic** — start after the prior
block, no hard-coded row numbers.

## Acceptance criteria
- [ ] `render_snapshot(snapshot: &Snapshot, out_path: &Path) -> Result<(), Error>` writes a valid xlsx.
- [ ] Header cells carry the exact golden labels and teal fill; body uses the LUFS fonts.
- [ ] Money columns use number format `"$"#,##0.00`; APR/utilization use percent.
- [ ] SUBSCRIPTIONS split monthly-then-annual; `DUE` = authored ordinal; `DUE DATE` = computed ISO; `Work Expense?` = `x` when deductible.
- [ ] LIVING `due` = ordinal or `end of month`.
- [ ] "since" footer: `total: $<since_counted_total>`, `statements: link`, `last met: <last_met>`.
- [ ] Column widths mirror `snapshot_sheet.py`'s widths table.

## Interface contract
`pub fn render_snapshot(snapshot: &Snapshot, out_path: &Path) -> Result<(), Error>`. Consumes the exact `Snapshot` from Unit 01. Produces the file Unit 04 reads back.

## Boundaries — do NOT touch
`src/reader.rs` (Unit 02), `src/verify.rs` (Unit 04), `src/main.rs` (Unit 06).

## Output
`src/render.rs`

## Verification
`cargo test render` + a fixture test that renders a snapshot and re-opens it with `calamine` (sanity: workbook opens, sheet `Snapshot` present).
