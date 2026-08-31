//! The typed intermediate document — the single handoff between reader, renderer, and verifier.

use serde::{Deserialize, Serialize};

/// A monthly recurring outflow (LIVING block).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Recurring {
    pub name: String,
    pub amount: f64,
    /// Authored due label, e.g. "17th" or "end of month".
    pub due: String,
}

/// A subscription (SUBSCRIPTIONS block). Monthly + annual.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Subscription {
    pub name: String,
    pub amount: f64,
    /// "Monthly" | "Annual".
    pub cadence: String,
    /// Authored due label: "8th", "May 5th", "".
    pub due: String,
    /// Computed next occurrence, ISO `YYYY-MM-DD`.
    pub due_date: String,
    /// `true` when deductible → rendered as `x` in "Work Expense?".
    pub work_expense: bool,
}

/// A credit account (CREDIT ACCOUNT INFO block).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreditAccount {
    pub name: String,
    pub balance: f64,
    pub minimum_payment: f64,
    /// Authored due label, e.g. "20th" / "end of month".
    pub payment_due: String,
    pub limit: f64,
    /// APR as a percent number (28.24).
    pub apr: f64,
    /// Computed utilization percent (0.0–100.0+).
    pub utilization: f64,
}

/// A debit (checking/savings) account (DEBIT ACCOUNT INFO block).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DebitAccount {
    pub name: String,
    /// Signed; negative = overdrawn.
    pub balance: f64,
    pub notes: String,
}

/// A work-income row (WORK INCOME block).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Income {
    pub name: String,
    pub balance: f64,
    pub notes: String,
}

/// A single transaction in the "since last met" list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Txn {
    pub date: String,
    pub payee: String,
    pub amount: f64,
    /// Raw source `txn_type`: income | expense | transfer | gift.
    pub txn_type: String,
    /// Title-case category: "Living", "Other", "Credit", "Subscriptions".
    pub category: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Meta {
    pub last_met: String,
    pub generated: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Totals {
    pub since_total: f64,
    pub since_expense_total: f64,
    /// Money that actually left the account (expenses + card payments).
    pub since_counted_total: f64,
    pub since_gift_total: f64,
    pub monthly_recurring: f64,
    pub subs_total: f64,
    pub credit_debt: f64,
    pub cash: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    pub meta: Meta,
    pub living: Vec<Recurring>,
    pub subscriptions: Vec<Subscription>,
    pub credit: Vec<CreditAccount>,
    pub debit: Vec<DebitAccount>,
    pub income: Vec<Income>,
    pub since: Vec<Txn>,
    pub totals: Totals,
}
