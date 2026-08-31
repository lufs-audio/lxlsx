# Unit 01 — Kernel: typed document, exit codes, JSON envelope

## Objective
Stand up the library crate `lxlsx` (lib) with the shareable kernel: the typed `Snapshot`
document, the exit-code taxonomy, the JSON envelope, and the `Error` type everything else maps into.

## Context
See `../../SPEC.md` for the phase-level contract. This is the single source of truth every
other unit builds on — the handoff document (`Snapshot`) and the error/exit surface. Match
`lufs-audio/bplate`'s `profiles/rust-cli/Cargo.toml` clippy lints (`unwrap_used=deny`,
`expect_used=deny`, `panic=deny`), and `lufs-audio/bplate/docs/units/10-exit-code-and-json-envelope-standard.md`.

## Acceptance criteria
- [ ] `src/lib.rs` exposes `mod snapshot; mod error; mod exit; mod json;` with the four modules below.
- [ ] `Error` is an enum with variants for `Db`, `Io`, `Xlsx`, `Verify`, `ProcessNotFound(String)`, `Contract(String)`; a `From<Error> for i32` maps to SPEC.md's exit-code table (0/1/2/3/4/5).
- [ ] `Snapshot` struct (serde Serialize/Deserialize) carries: `meta {last_met, generated, scope}`, `living: Vec<Recurring>`, `subscriptions: Vec<Subscription>`, `credit: Vec<CreditAccount>`, `debit: Vec<DebitAccount>`, `income: Vec<Income>`, `since: Vec<Txn>`, `totals: Totals`.
- [ ] Every amount is `f64`; all dates are `String` (ISO `YYYY-MM-DD`) to avoid cross-crate date-format drift.
- [ ] `json::envelope_ok(data)` → `{"status":"success","data":…}` and `json::envelope_err(code, msg)` → `{"status":"error","code":N,"message":"…"}`.
- [ ] `cargo build` clean under `-D warnings`; no `unwrap`/`expect`/`panic` in non-test code.

## Interface contract
The exact `Snapshot`/`Totals`/`Recurring`/`Subscription`/`CreditAccount`/`DebitAccount`/`Income`/`Txn` field names are the cross-unit handoff. Units 02 (reader) and 03 (renderer) and 04 (verifier) all consume/produce `Snapshot`. Do not rename fields without touching all four at once.

## Boundaries — do NOT touch
Root `Cargo.toml` dependency set beyond adding `serde`, `serde_json`, `thiserror` (already covered). Do not create `src/main.rs` (Unit 06 owns the CLI bin). Do not add any process under `processes/`.

## Output
`Cargo.toml` (workspace root with `[lib]`+`[[bin]]` later), `src/lib.rs`, `src/error.rs`, `src/exit.rs`, `src/json.rs`, `src/snapshot.rs`, `src/snapshot/totals.rs` if you split it.

## Verification
`cargo build` and `cargo clippy --all-targets -D warnings -D clippy::unwrap_used -D clippy::expect_used -D clippy::panic` return green.
