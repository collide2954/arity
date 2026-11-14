CREATE TABLE skip (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL REFERENCES run (id),
    text_id INTEGER NOT NULL REFERENCES text (id),
    path TEXT NOT NULL,
    block_key TEXT NOT NULL,
    quote TEXT NOT NULL,
    start INTEGER NOT NULL,
    "end" INTEGER NOT NULL,
    reason TEXT NOT NULL CHECK (reason IN ('cited', 'other')),
    note TEXT
);
CREATE INDEX skip_text ON skip (text_id, block_key);
