# Unit 04 — Verifier (read-back + structural contract)

## Objective
Implement `src/verify.rs`: open an emitted `.xlsx` (calamine), and prove it satisfies the
declared contract — the six blocks present with exact header labels, and totals reconciling
to the model. Returns `verified` / `violated` (never `unverifiable` for what can be checked).

## Context
This is the "proven, not exited-0" gate. Structural + deterministic: block headers, block
presence, and that the `total:` footers equal the sums of their columns, plus row-sums equal
the documented totals. It verifies the *layout contract*, not Daniel's private dollar values.

## Acceptance criteria
- [ ] `verify_xlsx(path) -> Result<VerifyReport, Error>` reads the workbook and returns a report of checks `[{name, ok, detail}]`.
- [ ] Asserts: exactly the six block header labels appear (in any position), each with the correct adjacent header cell text.
- [ ] Asserts footer `last met:` cell equals the `total:` row pattern.
- [ ] Asserts the numeric totals reconcile (sum of money column == the model recomputed from the same DB — so verify takes both `path` and the `Snapshot` for cross-check).
- [ ] A tampered/truncated file returns `CONTRACT_VIOLATED` (exit 5), not a panic or a false success.

## Interface contract
`pub fn verify_xlsx(path: &Path, snapshot: &Snapshot) -> Result<VerifyReport, Error>`. The report is JSON-serializable (`{"checks":[…],"verdict":"verified"|"violated"}`).

## Boundaries — do NOT touch
`src/render.rs` (Unit 03), `src/main.rs` (Unit 06).

## Output
`src/verify.rs`

## Verification
`cargo test verify` — including a negative test that mutates a header and asserts `violated`.
