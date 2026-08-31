-- Synthetic v2-schema finance.db — deterministic, no real financial data.
-- Shapes the six golden blocks; values are round-number placeholders.

PRAGMA foreign_keys = ON;

CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    balance REAL NOT NULL DEFAULT 0,
    type TEXT NOT NULL,           -- checking | savings | credit
    is_business INTEGER NOT NULL DEFAULT 0,
    notes TEXT DEFAULT ''
);

CREATE TABLE credit_accounts (
    account_id TEXT PRIMARY KEY REFERENCES accounts(id),
    balance REAL NOT NULL DEFAULT 0,
    credit_limit REAL,
    minimum_payment REAL,
    apr REAL,
    payment_due_day INTEGER
);

CREATE TABLE recurring_items (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    amount REAL NOT NULL,
    category TEXT,
    subcategory TEXT,
    due_day INTEGER,
    due_month INTEGER,
    cadence TEXT,                 -- Monthly | Annual
    deductible_pct INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    flow TEXT NOT NULL DEFAULT 'outflow',  -- outflow | inflow
    notes TEXT DEFAULT ''
);

CREATE TABLE transactions (
    id INTEGER PRIMARY KEY,
    account_id TEXT REFERENCES accounts(id),
    date TEXT NOT NULL,
    payee TEXT NOT NULL,
    amount REAL NOT NULL,
    txn_type TEXT NOT NULL,       -- income | expense | transfer | gift
    category TEXT,
    subcategory TEXT,
    deductible_pct INTEGER NOT NULL DEFAULT 0
);

-- ── Accounts ──
INSERT INTO accounts (id, name, balance, type, is_business, notes) VALUES
  ('acc-bus',    'LUFS Business Checking (6204)', 3.60,  'checking', 1, ''),
  ('acc-pers1',  'Personal Checking 1 (1657)',     1.00,  'checking', 0, ''),
  ('acc-pers2',  'Personal Checking 2 (8212)',     1.00,  'checking', 0, ''),
  ('acc-save',   'Savings',                        1.00,  'savings',  0, ''),
  ('acc-freedom','Chase Freedom',                  0.0,  'credit',   0, ''),
  ('acc-apple',  'Apple Card',                     0.0,  'credit',   0, ''),
  ('acc-vivint', 'Citizen Pay (Vivint)',           0.0,  'credit',   0, '');

-- ── Credit accounts ──
INSERT INTO credit_accounts (account_id, balance, credit_limit, minimum_payment, apr, payment_due_day) VALUES
  ('acc-freedom', -8292.00, 8500.00, 320.00, 28.24, 20),
  ('acc-apple',   -564.98,  14000.00, 25.00, 27.24, NULL),
  ('acc-vivint',  0.0,      5000.00,  0.00,  0.00,  NULL);

-- ── LIVING (monthly outflows, category != SUBSCRIPTIONS) ──
INSERT INTO recurring_items (name, amount, category, subcategory, due_day, cadence, flow) VALUES
  ('Groceries',            200.00, 'LIVING', 'Groceries',  NULL, 'Monthly', 'outflow'),
  ('Rent + Insurance',     710.95, 'LIVING', 'Rent',       NULL, 'Monthly', 'outflow'),
  ('SAWS Water',            48.07, 'LIVING', 'Utilities',   17, 'Monthly', 'outflow'),
  ('CPS Energy',           353.72, 'LIVING', 'Utilities',   27, 'Monthly', 'outflow'),
  ('Internet',             128.87, 'LIVING', 'Utilities',   13, 'Monthly', 'outflow'),
  ('Gas (Car)',             60.00, 'LIVING', 'Transport', NULL, 'Monthly', 'outflow'),
  ('Payroll',               55.48, 'LIVING', 'Payroll',     24, 'Monthly', 'outflow'),
  ('BCBS Healthcare',      291.24, 'LIVING', 'Healthcare',  31, 'Monthly', 'outflow'),
  ('Vivint Monitoring',     71.84, 'LIVING', 'Security',     5, 'Monthly', 'outflow'),
  ('ADP Payments',          83.95, 'LIVING', 'Payroll',      3, 'Monthly', 'outflow'),
  ('ADP Payroll Fee',      125.25, 'LIVING', 'Payroll',      1, 'Monthly', 'outflow'),
  ('Psychiatrist',          50.00, 'LIVING', 'Healthcare',   8, 'Monthly', 'outflow'),
  ('Therapist',             50.00, 'LIVING', 'Healthcare',  21, 'Monthly', 'outflow');

-- ── SUBSCRIPTIONS (monthly then annual) ──
INSERT INTO recurring_items (name, amount, category, subcategory, due_day, due_month, cadence, deductible_pct, flow) VALUES
  ('Monthly Service Fee 1792',   1.00, 'SUBSCRIPTIONS', 'Banking',   8,  NULL, 'Monthly', 0,   'outflow'),
  ('Monthly Service Fee 1657',  15.00, 'SUBSCRIPTIONS', 'Banking',  12,  NULL, 'Monthly', 0,   'outflow'),
  ('Monthly Service Fee 8212',  15.00, 'SUBSCRIPTIONS', 'Banking',  23,  NULL, 'Monthly', 0,   'outflow'),
  ('Monthly Service Fee 6204',  15.00, 'SUBSCRIPTIONS', 'Banking',  31,  NULL, 'Monthly', 0,   'outflow'),
  ('exe.dev',                   40.63, 'SUBSCRIPTIONS', 'Dev',      14,  NULL, 'Monthly', 100, 'outflow'),
  ('Google Drive',             100.00, 'SUBSCRIPTIONS', 'Storage',   5,  5,    'Annual',  100, 'outflow'),
  ('Amazon Prime',             139.00, 'SUBSCRIPTIONS', 'Retail',    3,  4,    'Annual',  0,   'outflow'),
  ('Costco Membership',         60.00, 'SUBSCRIPTIONS', 'Retail',    1,  5,    'Annual',  0,   'outflow'),
  ('danialrami.com domain',     20.00, 'SUBSCRIPTIONS', 'Domains',  11,  3,    'Annual',  100, 'outflow'),
  ('lufs.audio domain',        250.00, 'SUBSCRIPTIONS', 'Domains',  29,  3,    'Annual',  100, 'outflow'),
  ('lufsaud.io domain',         70.00, 'SUBSCRIPTIONS', 'Domains',   3,  4,    'Annual',  100, 'outflow'),
  ('Openrouter',               220.00, 'SUBSCRIPTIONS', 'AI',       NULL, NULL, 'Annual',  100, 'outflow');

-- ── WORK INCOME (recurring inflows) — deliberately empty in the golden ──
-- (no rows; the block renders as header-only)

-- ── Transactions since last_met (2026-08-15) ──
INSERT INTO transactions (account_id, date, payee, amount, txn_type, category) VALUES
  ('acc-bus', '2026-07-17', 'PALENQUE GROUP', -16.66, 'expense', 'OTHER'),
  ('acc-bus', '2026-07-17', 'COFFEE CRUSH',  -33.65, 'expense', 'OTHER'),
  ('acc-bus', '2026-07-17', 'HEB',           -14.47, 'expense', 'LIVING'),
  ('acc-bus', '2026-07-17', 'CPS',          -356.07, 'expense', 'LIVING'),
  ('acc-bus', '2026-07-20', 'UBER',          -33.94, 'expense', 'OTHER'),
  ('acc-bus', '2026-07-24', 'Payment to Chase', -320.00, 'transfer', 'CREDIT'),
  ('acc-bus', '2026-08-15', 'ADP WAGE PAY',  -46.17, 'expense', 'LIVING');
