//! XLSX renderer: typed `Snapshot` → brand-styled `.xlsx`, golden six-block layout.
//!
//! NOTE: `rust_xlsxwriter`'s `write_with_format(row, col, ...)` takes row first.

use rust_xlsxwriter::{Format, Workbook, Worksheet};
use std::path::Path;

use crate::error::Error;
use crate::snapshot::Snapshot;

const INK: &str = "111111";
const TEAL: &str = "78BEBA";
const USD: &str = "\"$\"#,##0.00";
const PCT: &str = "0.00%";

/// Render a snapshot to `out_path`.
///
/// # Errors
/// Returns [`Error::Xlsx`] on any write failure, [`Error::Io`] if the parent dir
/// can't be created, or [`Error::Contract`] if the output fails verification.
#[allow(clippy::too_many_lines)]
pub fn render_snapshot(snapshot: &Snapshot, out_path: &Path) -> Result<(), Error> {
    let mut wb = Workbook::new();

    let head_fmt = Format::new()
        .set_font_name("Space Mono")
        .set_bold()
        .set_font_color(INK)
        .set_font_size(9.0)
        .set_background_color(TEAL);
    let label_fmt = Format::new()
        .set_font_name("Space Mono")
        .set_bold()
        .set_font_color(INK)
        .set_font_size(11.0);
    let body_fmt = Format::new()
        .set_font_name("Public Sans")
        .set_font_color(INK)
        .set_font_size(10.0);
    let money_fmt = Format::new()
        .set_font_name("Space Mono")
        .set_font_color(INK)
        .set_font_size(10.0)
        .set_num_format(USD)
        .set_align(rust_xlsxwriter::FormatAlign::Right);
    let total_fmt = Format::new()
        .set_font_name("Host Grotesk")
        .set_bold()
        .set_font_color(INK)
        .set_font_size(11.0)
        .set_num_format(USD)
        .set_align(rust_xlsxwriter::FormatAlign::Right);
    let pct_fmt = Format::new()
        .set_font_name("Space Mono")
        .set_font_color(INK)
        .set_font_size(10.0)
        .set_num_format(PCT)
        .set_align(rust_xlsxwriter::FormatAlign::Right);

    let ws = wb.add_worksheet().set_name("Snapshot")?;
    ws.set_screen_gridlines(false);

    // ── LEFT PANEL: LIVING (A–C) ──
    let mut row: u32 = 0;
    wh(ws, 0, row, "LIVING", &head_fmt)?;
    wh(ws, 1, row, "AMT", &head_fmt)?;
    wh(ws, 2, row, "MONTHLY DUE DATE", &head_fmt)?;
    row += 1;
    for item in &snapshot.living {
        ws.write_with_format(row, 0, &item.name, &body_fmt)?;
        ws.write_number_with_format(row, 1, item.amount, &money_fmt)?;
        ws.write_with_format(row, 2, &item.due, &body_fmt)?;
        row += 1;
    }
    let living_end = row;

    // ── LEFT PANEL: SUBSCRIPTIONS (A–F) ──
    let subs_hdr = living_end + 1;
    let subs_headers = [
        "SUBSCRIPTIONS",
        "AMT",
        "FREQUENCY",
        "DUE",
        "DUE DATE",
        "Work Expense?",
    ];
    for (i, h) in subs_headers.iter().enumerate() {
        wh(ws, i, subs_hdr, h, &head_fmt)?;
    }
    let mut sr = subs_hdr + 1;
    for s in &snapshot.subscriptions {
        ws.write_with_format(sr, 0, &s.name, &body_fmt)?;
        ws.write_number_with_format(sr, 1, s.amount, &money_fmt)?;
        ws.write_with_format(sr, 2, &s.cadence, &body_fmt)?;
        ws.write_with_format(sr, 3, &s.due, &body_fmt)?;
        ws.write_with_format(sr, 4, &s.due_date, &body_fmt)?;
        ws.write_with_format(sr, 5, if s.work_expense { "x" } else { "" }, &body_fmt)?;
        sr += 1;
    }
    ws.write_with_format(sr, 3, "total:", &label_fmt)?;
    ws.write_number_with_format(sr, 4, snapshot.totals.subs_total, &total_fmt)?;

    // ── RIGHT PANEL: CREDIT (G–L) ──
    let mut rr: u32 = 0;
    let credit_headers = [
        "CREDIT ACCOUNT INFO",
        "BALANCE",
        "MINIMUM PAYMENT",
        "PAYMENT DUE",
        "LIMIT",
        "APR",
    ];
    for (i, h) in credit_headers.iter().enumerate() {
        wh(ws, 6 + i, rr, h, &head_fmt)?;
    }
    rr += 1;
    for c in &snapshot.credit {
        ws.write_with_format(rr, 6, &c.name, &body_fmt)?;
        ws.write_number_with_format(rr, 7, c.balance.abs(), &money_fmt)?;
        ws.write_number_with_format(rr, 8, c.minimum_payment, &money_fmt)?;
        ws.write_with_format(rr, 9, &c.payment_due, &body_fmt)?;
        ws.write_number_with_format(rr, 10, c.limit, &money_fmt)?;
        ws.write_number_with_format(rr, 11, c.apr / 100.0, &pct_fmt)?;
        rr += 1;
    }

    // ── RIGHT PANEL: DEBIT (G–I) ──
    let debit_hdr = rr + 1;
    wh(ws, 6, debit_hdr, "DEBIT ACCOUNT INFO", &head_fmt)?;
    wh(ws, 7, debit_hdr, "BALANCE", &head_fmt)?;
    wh(ws, 8, debit_hdr, "NOTES", &head_fmt)?;
    let mut dr = debit_hdr + 1;
    for d in &snapshot.debit {
        ws.write_with_format(dr, 6, &d.name, &body_fmt)?;
        ws.write_number_with_format(dr, 7, d.balance, &money_fmt)?;
        ws.write_with_format(dr, 8, &d.notes, &body_fmt)?;
        dr += 1;
    }

    // ── RIGHT PANEL: WORK INCOME (G–I) ──
    let income_hdr = dr + 1;
    wh(ws, 6, income_hdr, "WORK INCOME", &head_fmt)?;
    wh(ws, 7, income_hdr, "BALANCE", &head_fmt)?;
    wh(ws, 8, income_hdr, "NOTES", &head_fmt)?;
    let mut ir = income_hdr + 1;
    for inc in &snapshot.income {
        ws.write_with_format(ir, 6, &inc.name, &body_fmt)?;
        ws.write_number_with_format(ir, 7, inc.balance, &money_fmt)?;
        ws.write_with_format(ir, 8, &inc.notes, &body_fmt)?;
        ir += 1;
    }

    // ── RIGHT PANEL: ALL EXPENSES SINCE LAST TIME (G–K) ──
    let since_hdr = ir + 1;
    let since_headers = [
        "ALL EXPENSES SINCE LAST TIME",
        "BALANCE",
        "DATE CHARGED",
        "CATEGORY",
        "NOTES",
    ];
    for (i, h) in since_headers.iter().enumerate() {
        wh(ws, 6 + i, since_hdr, h, &head_fmt)?;
    }
    let mut tr = since_hdr + 1;
    for t in &snapshot.since {
        ws.write_with_format(tr, 6, &t.payee, &body_fmt)?;
        ws.write_number_with_format(tr, 7, t.amount.abs(), &money_fmt)?;
        ws.write_with_format(tr, 8, &t.date, &body_fmt)?;
        ws.write_with_format(tr, 9, &t.category, &body_fmt)?;
        ws.write_with_format(tr, 10, &t.notes, &body_fmt)?;
        tr += 1;
    }
    ws.write_with_format(tr, 6, "total:", &label_fmt)?;
    ws.write_number_with_format(tr, 7, snapshot.totals.since_counted_total, &total_fmt)?;

    let stmt_row = tr + 1;
    ws.write_with_format(stmt_row, 6, "statements:", &label_fmt)?;
    ws.write_with_format(stmt_row, 8, "link", &body_fmt)?;
    ws.write_with_format(stmt_row, 9, "last met:", &label_fmt)?;
    ws.write_with_format(stmt_row, 10, &snapshot.meta.last_met, &total_fmt)?;

    let widths = [
        30.0, 12.0, 18.0, 14.0, 14.0, 14.0, 32.0, 14.0, 16.0, 14.0, 12.0, 10.0,
    ];
    for (i, w) in widths.iter().enumerate() {
        ws.set_column_width(to_col(i), *w)?;
    }

    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    wb.save(out_path)?;
    Ok(())
}

fn wh(ws: &mut Worksheet, col: usize, row: u32, text: &str, fmt: &Format) -> Result<(), Error> {
    ws.write_with_format(row, to_col(col), text, fmt)?;
    Ok(())
}

/// Convert a column index (always ≤ 12 here) to `u16` without a narrowing cast.
fn to_col(i: usize) -> u16 {
    u16::try_from(i).unwrap_or(u16::MAX)
}
