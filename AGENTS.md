# AGENTS.md — operating contract for lxlsx

Machine-facing rules for agents (human and AI) working in this repo. If this file and a
memory disagree, this file wins.

## What this is

`lxlsx` is a LUFS Primitive: a process registry for Excel workbooks. Its prime directive is
the same one that governs all of LUFS — **"works" means proven correct, not exited 0.** Every
render run re-opens its own output and verifies the structural contract before reporting
success.

## Conventions you must follow

- **Contract over vibes.** A change to the renderer is only "done" when `lxlsx verify` proves
  the six-block layout and totals reconcile. Ship a test in `tests/golden_parity.rs` with any
  renderer change.
- **Honest failure.** If a stage can't run (DB missing, process unknown, output invalid),
  return the matching exit code (`3`/`4`/`5`) and the JSON error envelope — never a false
  success, never a silently-substituted path.
- **Typed document is the handoff.** The `Snapshot` struct (`src/snapshot.rs`) is the single
  source of truth between reader, renderer, and verifier. Rename a field only by touching all
  four at once.
- **Exit-code floor.** `0` success, `2` usage, `5` contract-violated, `1` reserved. Domain
  codes `3`/`4` documented in `SPEC.md`.
- **JSON envelope.** `{"status":"success","data":…}` / `{"status":"error","code":N,"message":"…"}`.
- **Lints.** `cargo clippy --all-targets -- -D warnings` must be clean. No `unwrap`/`expect`/
  `panic` in non-test code (test files carry the trailing allow).

## Registry discipline

`processes/` is the filesystem-as-registry. Every process is `processes/<slug>/manifest.json`.
`processes/index.json` is generated — never hand-edited. Run `lxlsx index` after adding a
process; `lxlsx index --check` (or CI) proves it current.

## Where conventions live

Cross-cutting LUFS Primitive conventions (naming, exit codes, JSON envelope, the Seven LUFS
Tests) live in `lufs-audio/bplate`, not here — see `bplate/docs/units/07`, `08`, `10`, and
`bplate/SPEC.md` §2. This repo implements them; it does not re-author them.

## Boundaries

- `data/` is gitignored and private. Never commit financial data, tokens, or account numbers.
- The fixture (`tests/fixtures/seed.sql`) is synthetic; do not paste real figures into it.
