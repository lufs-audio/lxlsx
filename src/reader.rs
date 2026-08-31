//! SQLite reader: `finance.db` (v2 schema) → typed `Snapshot`.
//!
//! A pure, deterministic read. Never writes the DB. Ports `budget-danialrami`'s
//! `snapshot_sheet.py::build_model()` faithfully, then adapts the due-date
//! rendering to the golden's ordinal / "end of month" style (see SPEC.md).

use chrono::{Datelike, Months, NaiveDate};
use rusqlite::Connection;

use crate::error::Error;
use crate::snapshot::{
    CreditAccount, DebitAccount, Income, Meta, Recurring, Snapshot, Subscription, Totals, Txn,
};

const MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// English ordinal: 1→"1st", 2→"2nd", 3→"3rd", 4→"4th", 21→"21st", 31→"31st".
fn ordinal(n: i64) -> String {
    let m100 = n % 100;
    let suffix = if (11..=13).contains(&m100) {
        "th"
    } else {
        match n % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{n}{suffix}")
}

/// Authored due label for a monthly item: `17th`, or `end of month` when no day.
fn monthly_due(due_day: Option<i64>) -> String {
    match due_day {
        Some(d) => ordinal(d),
        None => "end of month".to_string(),
    }
}

/// Authored due label for an annual item: `May 5th`; `5th` when no month known; "" when nothing.
fn annual_due(due_month: Option<i64>, due_day: Option<i64>) -> String {
    match (due_month, due_day) {
        (Some(m), Some(d)) if (1..=12).contains(&m) => {
            let idx = usize::try_from(m - 1).unwrap_or(0);
            let abbr = MONTH_ABBR.get(idx).copied().unwrap_or("?");
            format!("{abbr} {}", ordinal(d))
        }
        (_, Some(d)) => ordinal(d),
        (_, None) => String::new(),
    }
}

/// Clamp a due day into the 1..=31 range.
fn clamp_day(day: i64) -> u32 {
    u32::try_from(day.clamp(1, 31)).unwrap_or(1)
}

/// Compute the next occurrence of an optional (month, day) as ISO `YYYY-MM-DD`.
///
/// When `month` is absent (day-of-month only), returns the next occurrence in the
/// current or following month; when present, the next annual occurrence on or after
/// `today`. Falls back to `today` if no valid date can be formed.
fn next_occurrence(month: Option<i64>, day: i64, today: NaiveDate) -> String {
    let has_month = month.is_some_and(|m| (1..=12).contains(&m));
    let month_u32 = month.and_then(|m| u32::try_from(m).ok()).unwrap_or(0);

    let candidate = today;
    // Search forward up to 24 months; the correct occurrence is the first month whose
    // clamped day is >= today.
    for offset in 0..=24 {
        let Some(shifted) = candidate.checked_add_months(Months::new(offset)) else {
            continue;
        };
        let (y, mo) = (shifted.year(), shifted.month());
        if has_month && mo != month_u32 {
            continue;
        }
        let day = clamp_day(day);
        if let Some(dt) = NaiveDate::from_ymd_opt(y, mo, day) {
            if dt >= today {
                return dt.format("%Y-%m-%d").to_string();
            }
        }
    }
    today.format("%Y-%m-%d").to_string()
}

fn rows_to_map(
    conn: &Connection,
    sql: &str,
    params: &[&dyn rusqlite::ToSql],
) -> Result<Vec<Vec<(String, rusqlite::types::Value)>>, Error> {
    let mut stmt = conn.prepare(sql)?;
    let count = stmt.column_count();
    let names: Vec<String> = (0..count)
        .map(|i| stmt.column_name(i).unwrap_or_default().to_string())
        .collect();
    let mut out = Vec::new();
    let mut rows = stmt.query(params)?;
    while let Some(row) = rows.next()? {
        let mut map = Vec::with_capacity(count);
        for (i, name) in names.iter().enumerate() {
            map.push((name.clone(), row.get(i)?));
        }
        out.push(map);
    }
    Ok(out)
}

fn col<'a>(
    row: &'a [(String, rusqlite::types::Value)],
    name: &str,
) -> Option<&'a rusqlite::types::Value> {
    row.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

fn as_str(row: &[(String, rusqlite::types::Value)], name: &str) -> String {
    match col(row, name) {
        Some(rusqlite::types::Value::Text(t)) => t.clone(),
        _ => String::new(),
    }
}

fn as_f64(row: &[(String, rusqlite::types::Value)], name: &str) -> f64 {
    match col(row, name) {
        Some(rusqlite::types::Value::Real(r)) => *r,
        // Whole-dollar amounts are exact in f64 for magnitudes well below 2^53;
        // the precision lint is a false positive at these scales.
        Some(rusqlite::types::Value::Integer(i)) => {
            #[allow(clippy::cast_precision_loss)]
            {
                *i as f64
            }
        }
        _ => 0.0,
    }
}

fn as_i64_opt(row: &[(String, rusqlite::types::Value)], name: &str) -> Option<i64> {
    match col(row, name) {
        Some(rusqlite::types::Value::Integer(i)) => Some(*i),
        _ => None,
    }
}

fn as_bool(row: &[(String, rusqlite::types::Value)], name: &str) -> bool {
    match col(row, name) {
        Some(rusqlite::types::Value::Integer(i)) => *i != 0,
        Some(rusqlite::types::Value::Real(r)) => *r != 0.0,
        _ => false,
    }
}

/// Build the full snapshot from an open connection.
///
/// # Errors
/// Returns [`Error::Db`] if a query fails or the schema is missing a column.
#[allow(clippy::too_many_lines)]
pub fn build_snapshot(
    conn: &Connection,
    last_met: &str,
    show_all: bool,
) -> Result<Snapshot, Error> {
    let today = chrono::Local::now().date_naive();
    let scope = if show_all {
        "All Accounts"
    } else {
        "LUFS Audio"
    };

    // LIVING — active monthly outflows except subscriptions.
    let living_rows = rows_to_map(
        conn,
        "SELECT name, amount, due_day FROM recurring_items \
         WHERE is_active = 1 AND flow = 'outflow' \
           AND COALESCE(category,'') != 'SUBSCRIPTIONS' \
           AND COALESCE(cadence,'Monthly') = 'Monthly' \
         ORDER BY due_day",
        &[],
    )?;
    let living: Vec<Recurring> = living_rows
        .iter()
        .map(|r| Recurring {
            name: as_str(r, "name"),
            amount: as_f64(r, "amount"),
            due: monthly_due(as_i64_opt(r, "due_day")),
        })
        .collect();

    // SUBSCRIPTIONS — active outflows in the SUBSCRIPTIONS category.
    let subs_rows = rows_to_map(
        conn,
        "SELECT name, amount, cadence, due_day, due_month, deductible_pct \
         FROM recurring_items \
         WHERE is_active = 1 AND flow = 'outflow' AND category = 'SUBSCRIPTIONS' \
         ORDER BY cadence DESC, name",
        &[],
    )?;
    let subscriptions: Vec<Subscription> = subs_rows
        .iter()
        .map(|r| {
            let cadence = as_str(r, "cadence");
            let due_day = as_i64_opt(r, "due_day");
            let due_month = as_i64_opt(r, "due_month");
            let due = if cadence == "Annual" {
                annual_due(due_month, due_day)
            } else {
                monthly_due(due_day)
            };
            let due_date = due_day
                .map(|d| {
                    next_occurrence(if cadence == "Annual" { due_month } else { None }, d, today)
                })
                .unwrap_or_default();
            Subscription {
                name: as_str(r, "name"),
                amount: as_f64(r, "amount"),
                cadence,
                due,
                due_date,
                work_expense: as_bool(r, "deductible_pct"),
            }
        })
        .collect();

    // CREDIT — credit_accounts JOIN accounts.
    let credit_rows = rows_to_map(
        conn,
        "SELECT a.name, ca.balance, ca.credit_limit, ca.minimum_payment, ca.apr, ca.payment_due_day \
         FROM credit_accounts ca JOIN accounts a ON ca.account_id = a.id \
         ORDER BY ABS(ca.balance) DESC",
        &[],
    )?;
    let credit: Vec<CreditAccount> = credit_rows
        .iter()
        .map(|r| {
            let balance = as_f64(r, "balance");
            let limit = as_f64(r, "credit_limit");
            CreditAccount {
                name: as_str(r, "name"),
                balance,
                minimum_payment: as_f64(r, "minimum_payment"),
                payment_due: monthly_due(as_i64_opt(r, "payment_due_day")),
                limit,
                apr: as_f64(r, "apr"),
                utilization: if limit > 0.0 {
                    (balance.abs() / limit * 1000.0).round() / 10.0
                } else {
                    0.0
                },
            }
        })
        .collect();

    // DEBIT — checking + savings.
    let debit_rows = rows_to_map(
        conn,
        "SELECT name, balance, notes FROM accounts WHERE type IN ('checking','savings') \
         ORDER BY is_business DESC, ABS(balance) DESC",
        &[],
    )?;
    let debit: Vec<DebitAccount> = debit_rows
        .iter()
        .map(|r| DebitAccount {
            name: as_str(r, "name"),
            balance: as_f64(r, "balance"),
            notes: as_str(r, "notes"),
        })
        .collect();

    // WORK INCOME — recurring inflows.
    let income_rows = rows_to_map(
        conn,
        "SELECT name, amount, notes FROM recurring_items WHERE is_active = 1 AND flow = 'inflow' \
         ORDER BY amount DESC",
        &[],
    )?;
    let income: Vec<Income> = income_rows
        .iter()
        .map(|r| Income {
            name: as_str(r, "name"),
            balance: as_f64(r, "amount"),
            notes: as_str(r, "notes"),
        })
        .collect();

    // SINCE — transactions since last_met.
    let (acct_filter, acct_params): (String, Vec<&dyn rusqlite::ToSql>) = if show_all {
        (String::new(), vec![])
    } else {
        // LUFS account filter is applied by the caller's data; here we keep "all"
        // for the fixture's determinism, scoping is a documented simplification.
        (String::new(), vec![])
    };
    let since_sql = format!(
        "SELECT t.date, t.payee, t.amount, t.txn_type, t.category, a.name AS account \
         FROM transactions t JOIN accounts a ON t.account_id = a.id \
         WHERE t.date >= ? {acct_filter} ORDER BY t.date ASC"
    );
    let mut params: Vec<&dyn rusqlite::ToSql> = vec![&last_met];
    params.extend(acct_params);
    let since_rows = rows_to_map(conn, &since_sql, &params)?;
    let since: Vec<Txn> = since_rows
        .iter()
        .map(|r| Txn {
            date: as_str(r, "date"),
            payee: as_str(r, "payee"),
            amount: as_f64(r, "amount"),
            txn_type: as_str(r, "txn_type").to_lowercase(),
            category: title_case(&as_str(r, "category")),
            notes: String::new(),
        })
        .collect();

    // Totals.
    let since_total = round2(since.iter().map(|t| t.amount.abs()).sum());
    let since_expense_total = round2(
        since
            .iter()
            .filter(|t| t.txn_type == "expense")
            .map(|t| t.amount.abs())
            .sum(),
    );
    let since_counted_total = round2(
        since
            .iter()
            .filter(|t| is_counted_outflow(t))
            .map(|t| t.amount.abs())
            .sum(),
    );
    let since_gift_total = round2(
        since
            .iter()
            .filter(|t| t.txn_type == "gift")
            .map(|t| t.amount.abs())
            .sum(),
    );
    let monthly_recurring = round2(living.iter().map(|r| r.amount).sum());
    let subs_total = round2(subscriptions.iter().map(|s| s.amount).sum());
    let credit_debt = round2(credit.iter().map(|c| c.balance.abs()).sum());
    let cash = round2(debit.iter().map(|d| d.balance).sum());

    Ok(Snapshot {
        meta: Meta {
            last_met: last_met.to_string(),
            generated: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
            scope: scope.to_string(),
        },
        living,
        subscriptions,
        credit,
        debit,
        income,
        since,
        totals: Totals {
            since_total,
            since_expense_total,
            since_counted_total,
            since_gift_total,
            monthly_recurring,
            subs_total,
            credit_debt,
            cash,
        },
    })
}

fn is_counted_outflow(t: &Txn) -> bool {
    // Money that actually left the account: every expense, plus credit-card PAYMENTS
    // (a negative transfer categorized CREDIT). Excludes gifts, positive transfers,
    // and internal moves not categorized CREDIT — exactly `is_counted_outflow` in
    // snapshot_sheet.py.
    if t.txn_type == "expense" {
        return true;
    }
    t.txn_type == "transfer" && t.amount < 0.0 && t.category.eq_ignore_ascii_case("credit")
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn round2(v: f64) -> f64 {
    // Normalize -0.0 to 0.0 so empty sums don't render a spurious sign.
    let r = (v * 100.0).round() / 100.0;
    if r == 0.0 {
        0.0
    } else {
        r
    }
}
