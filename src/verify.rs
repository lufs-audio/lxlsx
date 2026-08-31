//! Verifier: read the emitted `.xlsx` back and prove it satisfies the layout contract.

use calamine::Reader;
use serde::Serialize;
use std::path::Path;

use crate::error::Error;

const BLOCK_HEADERS: [&str; 14] = [
    "LIVING",
    "AMT",
    "MONTHLY DUE DATE",
    "SUBSCRIPTIONS",
    "FREQUENCY",
    "DUE DATE",
    "Work Expense?",
    "CREDIT ACCOUNT INFO",
    "BALANCE",
    "MINIMUM PAYMENT",
    "PAYMENT DUE",
    "LIMIT",
    "APR",
    "DEBIT ACCOUNT INFO",
];

const SINCE_HEADER: &str = "ALL EXPENSES SINCE LAST TIME";
const WORK_INCOME_HEADER: &str = "WORK INCOME";

#[derive(Debug, Clone, Serialize)]
pub struct Check {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub checks: Vec<Check>,
    pub verdict: String,
}

/// Verify the emitted xlsx against the six-block structural contract.
///
/// # Errors
/// Returns [`Error::Verify`] if the file can't be opened or parsed.
pub fn verify_xlsx(path: &Path) -> Result<VerifyReport, Error> {
    let mut workbook = calamine::open_workbook_auto(path)?;
    let sheet_names = workbook.sheet_names().clone();
    let mut checks: Vec<Check> = Vec::new();

    checks.push(Check {
        name: "workbook_opens".into(),
        ok: !sheet_names.is_empty(),
        detail: format!("sheets: {sheet_names:?}"),
    });

    let mut found: Vec<String> = Vec::new();

    // Collect every cell's string value across all sheets.
    if let Some(sheet_name) = sheet_names.first().cloned() {
        if let Ok(range) = workbook.worksheet_range(&sheet_name) {
            for row in range.rows() {
                for cell in row {
                    if let calamine::Data::String(s) = cell {
                        let t = s.trim().to_string();
                        if !t.is_empty() {
                            found.push(t);
                        }
                    }
                }
            }
        }
    }

    for h in BLOCK_HEADERS {
        let ok = found.iter().any(|s| s == h);
        checks.push(Check {
            name: format!("header:{h}"),
            ok,
            detail: if ok {
                "present".into()
            } else {
                "missing".into()
            },
        });
    }
    let since_ok = found.iter().any(|s| s == SINCE_HEADER);
    checks.push(Check {
        name: format!("header:{SINCE_HEADER}"),
        ok: since_ok,
        detail: if since_ok {
            "present".into()
        } else {
            "missing".into()
        },
    });
    let income_ok = found.iter().any(|s| s == WORK_INCOME_HEADER);
    checks.push(Check {
        name: format!("header:{WORK_INCOME_HEADER}"),
        ok: income_ok,
        detail: if income_ok {
            "present".into()
        } else {
            "missing".into()
        },
    });

    // Footer markers.
    let total_ok = found.iter().any(|s| s == "total:");
    checks.push(Check {
        name: "footer:total".into(),
        ok: total_ok,
        detail: if total_ok {
            "present".into()
        } else {
            "missing".into()
        },
    });
    let last_met_ok = found.iter().any(|s| s == "last met:");
    checks.push(Check {
        name: "footer:last_met".into(),
        ok: last_met_ok,
        detail: if last_met_ok {
            "present".into()
        } else {
            "missing".into()
        },
    });

    let all_ok = checks.iter().all(|c| c.ok);
    Ok(VerifyReport {
        checks,
        verdict: if all_ok {
            "verified".into()
        } else {
            "violated".into()
        },
    })
}
