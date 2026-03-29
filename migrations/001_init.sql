CREATE TABLE IF NOT EXISTS invoices (
    id TEXT PRIMARY KEY,
    diversifier_index INTEGER NOT NULL UNIQUE,
    address TEXT NOT NULL,
    amount_zat INTEGER NOT NULL,
    memo TEXT,
    invoice_type TEXT NOT NULL DEFAULT 'program',
    wallet_hash TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    received_zat INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    expires_at TEXT,
    paid_at TEXT,
    paid_txid TEXT,
    paid_height INTEGER
);

CREATE TABLE IF NOT EXISTS scan_state (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    last_scanned_height INTEGER NOT NULL,
    next_diversifier_index INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS miner_assignments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_hash TEXT NOT NULL,
    wallet_address TEXT NOT NULL,
    serial_number TEXT NOT NULL,
    foreman_miner_id INTEGER,
    assigned_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS payment_records (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    invoice_id TEXT NOT NULL,
    txid TEXT NOT NULL,
    value_zat INTEGER NOT NULL,
    height INTEGER,
    source TEXT NOT NULL DEFAULT 'block',
    created_at TEXT NOT NULL,
    UNIQUE(invoice_id, txid)
);

CREATE INDEX IF NOT EXISTS idx_invoices_status ON invoices(status);
CREATE INDEX IF NOT EXISTS idx_invoices_wallet ON invoices(wallet_hash);
CREATE INDEX IF NOT EXISTS idx_payment_records_invoice ON payment_records(invoice_id);

CREATE TABLE IF NOT EXISTS merkle_leaves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    leaf_hash TEXT NOT NULL UNIQUE,
    event_type INTEGER NOT NULL,
    wallet_hash TEXT NOT NULL,
    serial_number TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS merkle_roots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    root_hash TEXT NOT NULL,
    leaf_count INTEGER NOT NULL,
    anchor_txid TEXT,
    anchor_height INTEGER,
    created_at TEXT NOT NULL
);
