//! XLSX renderer: typed `Snapshot` → brand-styled `.xlsx`, golden six-block layout.
//!
//! NOTE: `rust_xlsxwriter`'s `write_with_format(row, col, ...)` takes row first.

use chrono::{Datelike, NaiveDate};
use rust_xlsxwriter::{Format, Workbook, Worksheet};
use std::path::Path;

use crate::error::Error;
use crate::snapshot::Snapshot;

const INK: &str = "111111";
const TEAL: &str = "78BEBA";
const BABY_BLUE: &str = "BDD7EE";
const USD: &str = "\"$\"#,##0.00";
const PCT: &str = "0.00%";
const DRIVE_LINK: &str =
    "https://drive.google.com/drive/folders/1ZEcIq0tchuec55Vys0W-EsTZUQDZ1V3I?usp=drive_link";

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

    let due_soon_body_fmt = Format::new()
        .set_font_name("Public Sans")
        .set_font_color(INK)
        .set_font_size(10.0)
        .set_background_color(BABY_BLUE);
    let due_soon_money_fmt = Format::new()
        .set_font_name("Space Mono")
        .set_font_color(INK)
        .set_font_size(10.0)
        .set_num_format(USD)
        .set_align(rust_xlsxwriter::FormatAlign::Right)
        .set_background_color(BABY_BLUE);
    let due_soon_pct_fmt = Format::new()
        .set_font_name("Space Mono")
        .set_font_color(INK)
        .set_font_size(10.0)
        .set_num_format(PCT)
        .set_align(rust_xlsxwriter::FormatAlign::Right)
        .set_background_color(BABY_BLUE);

    let cat_living_fmt = Format::new()
        .set_font_name("Public Sans")
        .set_font_size(10.0)
        .set_background_color("D4EDBC")
        .set_font_color("11734B");
    let cat_other_fmt = Format::new()
        .set_font_name("Public Sans")
        .set_font_size(10.0)
        .set_background_color("BFE1F6")
        .set_font_color("0A53A8");
    let cat_credit_fmt = Format::new()
        .set_font_name("Public Sans")
        .set_font_size(10.0)
        .set_background_color("C6DBE1")
        .set_font_color("215A6C");
    let cat_subs_fmt = Format::new()
        .set_font_name("Public Sans")
        .set_font_size(10.0)
        .set_background_color("FFE5A0")
        .set_font_color("473821");

    let link_fmt = Format::new()
        .set_font_name("Public Sans")
        .set_font_size(10.0)
        .set_font_color("1155CC")
        .set_underline(rust_xlsxwriter::FormatUnderline::Single);

    let anchor = chrono::Local::now().date_naive();

    let ws = wb.add_worksheet().set_name("Snapshot")?;
    ws.set_screen_gridlines(false);

    // ── LEFT PANEL: LIVING (A–C) ──
    let mut row: u32 = 0;
    wh(ws, 0, row, "LIVING", &head_fmt)?;
    wh(ws, 1, row, "AMT", &head_fmt)?;
    wh(ws, 2, row, "MONTHLY DUE DATE", &head_fmt)?;
    row += 1;
    for item in &snapshot.living {
        let soon = is_day_due_soon(&item.due, anchor, 10);
        let bfmt = if soon { &due_soon_body_fmt } else { &body_fmt };
        let mfmt = if soon {
            &due_soon_money_fmt
        } else {
            &money_fmt
        };
        ws.write_with_format(row, 0, &item.name, bfmt)?;
        ws.write_number_with_format(row, 1, item.amount, mfmt)?;
        ws.write_with_format(row, 2, &item.due, bfmt)?;
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
        let soon = is_due_soon(&s.due_date, anchor, 10);
        let bfmt = if soon { &due_soon_body_fmt } else { &body_fmt };
        let mfmt = if soon {
            &due_soon_money_fmt
        } else {
            &money_fmt
        };
        ws.write_with_format(sr, 0, &s.name, bfmt)?;
        ws.write_number_with_format(sr, 1, s.amount, mfmt)?;
        ws.write_with_format(sr, 2, &s.cadence, bfmt)?;
        ws.write_with_format(sr, 3, &s.due, bfmt)?;
        ws.write_with_format(sr, 4, &s.due_date, bfmt)?;
        ws.write_with_format(sr, 5, if s.work_expense { "x" } else { "" }, bfmt)?;
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
        let soon = is_day_due_soon(&c.payment_due, anchor, 10);
        let bfmt = if soon { &due_soon_body_fmt } else { &body_fmt };
        let mfmt = if soon {
            &due_soon_money_fmt
        } else {
            &money_fmt
        };
        let pfmt = if soon { &due_soon_pct_fmt } else { &pct_fmt };
        ws.write_with_format(rr, 6, &c.name, bfmt)?;
        ws.write_number_with_format(rr, 7, c.balance.abs(), mfmt)?;
        ws.write_number_with_format(rr, 8, c.minimum_payment, mfmt)?;
        ws.write_with_format(rr, 9, &c.payment_due, bfmt)?;
        ws.write_number_with_format(rr, 10, c.limit, mfmt)?;
        ws.write_number_with_format(rr, 11, c.apr / 100.0, pfmt)?;
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
        let cfmt = match t.category.to_lowercase().as_str() {
            "living" => &cat_living_fmt,
            "other" => &cat_other_fmt,
            "credit" => &cat_credit_fmt,
            "subscriptions" => &cat_subs_fmt,
            _ => &body_fmt,
        };
        ws.write_with_format(tr, 9, &t.category, cfmt)?;
        ws.write_with_format(tr, 10, &t.notes, &body_fmt)?;
        tr += 1;
    }
    ws.write_with_format(tr, 6, "total:", &label_fmt)?;
    ws.write_number_with_format(tr, 7, snapshot.totals.since_counted_total, &total_fmt)?;

    let stmt_row = tr + 1;
    ws.write_with_format(stmt_row, 6, "statements:", &label_fmt)?;
    let _ = ws.write_url_with_format(stmt_row, 8, DRIVE_LINK, &link_fmt);
    ws.write_with_format(stmt_row, 9, "last met:", &label_fmt)?;
    ws.write_with_format(stmt_row, 10, &snapshot.meta.last_met, &total_fmt)?;

    // ── Screenshot embed (Column M) ──
    if let Some(shot_path) = find_latest_screenshot() {
        if let Ok(img) = rust_xlsxwriter::Image::new(&shot_path) {
            let img = img.set_scale_to_size(300, 900, true);
            wh(ws, 12, 0, "CHASE TRANSACTIONS", &head_fmt)?;
            let _ = ws.insert_image(1, 12, &img);
        }
    }

    let widths = [
        30.0, 12.0, 18.0, 14.0, 14.0, 14.0, 32.0, 14.0, 16.0, 14.0, 12.0, 10.0, 36.0,
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

fn is_due_soon(date_str: &str, anchor: NaiveDate, days: i64) -> bool {
    if let Ok(d) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let diff = (d - anchor).num_days();
        (0..=days).contains(&diff)
    } else {
        false
    }
}

fn is_day_due_soon(day_str: &str, anchor: NaiveDate, days: i64) -> bool {
    let digits: String = day_str.chars().take_while(char::is_ascii_digit).collect();
    let Ok(day) = digits.parse::<u32>() else {
        return false;
    };
    if day == 0 || day > 31 {
        return false;
    }
    let cur_m = anchor.month();
    let cur_y = anchor.year();
    let max_days = days_in_month(cur_y, cur_m);
    let clamped_day = day.min(max_days);
    let cand = if let Some(d) = NaiveDate::from_ymd_opt(cur_y, cur_m, clamped_day) {
        if d < anchor {
            let (ny, nm) = if cur_m == 12 {
                (cur_y + 1, 1)
            } else {
                (cur_y, cur_m + 1)
            };
            let next_max = days_in_month(ny, nm);
            NaiveDate::from_ymd_opt(ny, nm, day.min(next_max))
        } else {
            Some(d)
        }
    } else {
        None
    };

    if let Some(cand_date) = cand {
        let diff = (cand_date - anchor).num_days();
        (0..=days).contains(&diff)
    } else {
        false
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    if month == 12 {
        31
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .and_then(|d| d.pred_opt())
            .map_or(30, |d| d.day())
    }
}

fn find_latest_screenshot() -> Option<std::path::PathBuf> {
    let dirs = [
        std::path::PathBuf::from("data/screenshots"),
        std::path::PathBuf::from("../budget-danialrami/data/screenshots"),
        std::path::PathBuf::from("/Users/danielramirez/repos/budget-danialrami/data/screenshots"),
        std::path::PathBuf::from("reference/finances_2026-08-31/resources"),
    ];
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut images = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ["png", "jpg", "jpeg", "webp"].contains(&ext_lower.as_str()) {
                        if let Ok(meta) = entry.metadata() {
                            if let Ok(mtime) = meta.modified() {
                                images.push((mtime, path));
                            }
                        }
                    }
                }
            }
            if !images.is_empty() {
                images.sort_by_key(|b| std::cmp::Reverse(b.0));
                return Some(images.remove(0).1);
            }
        }
    }
    None
}
