CREATE TABLE IF NOT EXISTS spendable_notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    txid TEXT NOT NULL,
    action_index INTEGER NOT NULL,
    value_zat INTEGER NOT NULL,
    rseed BLOB NOT NULL,
    rho BLOB NOT NULL,
    recipient BLOB NOT NULL,
    nullifier BLOB NOT NULL UNIQUE,
    position INTEGER NOT NULL,
    height INTEGER NOT NULL,
    spent_txid TEXT,
    created_at TEXT NOT NULL,
    UNIQUE(txid, action_index)
);

CREATE INDEX IF NOT EXISTS idx_spendable_notes_unspent
    ON spendable_notes(spent_txid) WHERE spent_txid IS NULL;
