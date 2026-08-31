# Unit 02 — SQLite reader (DB → Snapshot)

## Objective
Implement `src/reader.rs`: open a `finance.db` (v2 schema), run the six block queries, and
build the typed `Snapshot` from Unit 01 — a pure, deterministic read (never writes the DB).

## Context
Ports `snapshot_sheet.py::build_model()` faithfully. The v2 schema (from that file):

- `recurring_items(name, amount, category, subcategory, due_day, cadence, deductible_pct, is_active, flow, notes)`
- `credit_accounts(account_id, balance, credit_limit, minimum_payment, apr, payment_due_day)` JOIN `accounts(id, name, balance, type, is_business)`
- `accounts(id, name, balance, type, is_business)` for debit (checking/savings)
- `recurring_items` flow='inflow' for income
- `transactions(id, account_id, date, payee, amount, txn_type, category, subcategory, deductible_pct)` for "since"
- `data/config.csv` carries `last_met` (and `home_office_pct`) — not in the DB; `last_met` is supplied via CLI flag with a sensible default.

## Acceptance criteria
- [ ] `build_snapshot(conn, last_met, show_all) -> Result<Snapshot>` runs six queries and populates every `Snapshot` field.
- [ ] LIVING = active monthly outflows where category != SUBSCRIPTIONS, ordered by `due_day`; each row's `due` is the ordinal (`17th`) or `end of month` (no day).
- [ ] SUBSCRIPTIONS = active outflows category = SUBSCRIPTIONS (monthly+annual), ordered cadence DESC then name; `duedate` computed next occurrence (next month for Monthly; next year for Annual month-day).
- [ ] CREDIT joins credit_accounts→accounts, computes `utilization`.
- [ ] DEBIT = checking/savings; INCOME = flow='inflow'; SINCE = transactions where `date >= last_met` ordered ASC, with totals `since_total`, `since_expense_total`, `since_counted_total` (expenses + negative transfers categorized CREDIT), `since_gift_total`.
- [ ] Totals block computed: `monthly_recurring`, `subs_total`, `credit_debt`, `cash`.

## Interface contract
Exposes `pub fn build_snapshot(conn: &Connection, last_met: &str, show_all: bool) -> Result<Snapshot, Error>`. The `Snapshot` it returns is exactly Unit 01's struct.

## Boundaries — do NOT touch
`src/render.rs` (Unit 03) and `src/verify.rs` (Unit 04). Do not read `data/config.csv` from this unit — `last_met` comes in as a parameter from the CLI (Unit 06).

## Output
`src/reader.rs`

## Verification
`cargo test reader` plus a fixture-driven test in `tests/` that builds the synthetic DB (Unit 05's fixture) and asserts row counts and totals.
