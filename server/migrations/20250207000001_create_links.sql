-- Links table: stores encrypted text and access rules.
-- Designed for SQLite; can be adapted for PostgreSQL (e.g. SERIAL, TIMESTAMPTZ).
CREATE TABLE IF NOT EXISTS links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    token TEXT NOT NULL UNIQUE,
    encrypted_text TEXT NOT NULL,
    password_hash TEXT,
    expires_at TEXT,
    one_time_view INTEGER NOT NULL DEFAULT 0,
    one_time_password INTEGER NOT NULL DEFAULT 0,
    view_count INTEGER NOT NULL DEFAULT 0,
    password_used INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_links_token ON links(token);
CREATE INDEX IF NOT EXISTS idx_links_expires_at ON links(expires_at);
