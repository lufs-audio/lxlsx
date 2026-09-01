//! Golden-parity integration test: build a fixture DB in-memory, render, verify.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use rusqlite::Connection;
use std::path::PathBuf;

fn seeded_conn() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(include_str!("fixtures/seed.sql"))
        .unwrap();
    conn
}

fn out_path() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/test-output");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(format!("finances-{}.xlsx", std::process::id()))
}

#[test]
fn golden_parity_roundtrip() {
    let conn = seeded_conn();
    let out = out_path();

    let snap = lxlsx::reader::build_snapshot(&conn, "2026-08-15", false).unwrap();
    lxlsx::render::render_snapshot(&snap, &out).unwrap();

    let report = lxlsx::verify::verify_xlsx(&out).unwrap();
    assert_eq!(report.verdict, "verified", "checks: {:?}", report.checks);

    let names: Vec<&str> = report.checks.iter().map(|c| c.name.as_str()).collect();
    for h in [
        "header:LIVING",
        "header:SUBSCRIPTIONS",
        "header:CREDIT ACCOUNT INFO",
        "header:DEBIT ACCOUNT INFO",
        "header:WORK INCOME",
        "header:ALL EXPENSES SINCE LAST TIME",
        "footer:total",
        "footer:last_met",
    ] {
        assert!(names.iter().any(|n| n == &h), "missing {h}");
    }

    assert!(snap.totals.monthly_recurring > 0.0, "living should sum");
    assert!(snap.totals.since_counted_total > 0.0, "since should sum");
    assert!(snap.totals.credit_debt > 0.0, "credit debt should sum");
}

#[test]
fn ordinal_due_labels() {
    let conn = seeded_conn();
    let snap = lxlsx::reader::build_snapshot(&conn, "2026-08-15", false).unwrap();

    let saws = snap.living.iter().find(|r| r.name == "SAWS Water").unwrap();
    assert_eq!(saws.due, "17th");

    let groceries = snap.living.iter().find(|r| r.name == "Groceries").unwrap();
    assert_eq!(groceries.due, "end of month");

    let annual = snap
        .subscriptions
        .iter()
        .find(|s| s.name == "Amazon Prime")
        .unwrap();
    assert_eq!(annual.cadence, "Annual");
    assert_eq!(annual.due, "Apr 3rd");

    // WORK INCOME block is empty in the fixture (matches golden).
    assert!(snap.income.is_empty());
}

#[test]
fn counted_outflow_excludes_gift_and_internal_transfer() {
    let conn = seeded_conn();
    let snap = lxlsx::reader::build_snapshot(&conn, "2026-08-15", false).unwrap();

    let expenses: f64 = snap
        .since
        .iter()
        .filter(|t| t.txn_type == "expense")
        .map(|t| t.amount.abs())
        .sum();
    let credit_xfer: f64 = snap
        .since
        .iter()
        .filter(|t| {
            t.txn_type == "transfer" && t.amount < 0.0 && t.category.eq_ignore_ascii_case("credit")
        })
        .map(|t| t.amount.abs())
        .sum();
    let expected = expenses + credit_xfer;
    assert!(
        (snap.totals.since_counted_total - expected).abs() < 0.01,
        "counted {} != expected {}",
        snap.totals.since_counted_total,
        expected
    );
}

#[test]
fn exe_dev_subscription_and_work_expense() {
    let conn = seeded_conn();
    let snap = lxlsx::reader::build_snapshot(&conn, "2026-08-15", false).unwrap();

    let exe = snap
        .subscriptions
        .iter()
        .find(|s| s.name == "exe.dev")
        .unwrap();
    assert_eq!(exe.cadence, "Monthly");
    assert_eq!(exe.due, "14th");
    assert!(exe.work_expense);
    assert!(!snap.subscriptions.iter().any(|s| s.name == "Apple One"));
}
