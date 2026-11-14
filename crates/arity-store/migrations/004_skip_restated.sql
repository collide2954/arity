CREATE TABLE skip_next (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL REFERENCES run (id),
    text_id INTEGER NOT NULL REFERENCES text (id),
    path TEXT NOT NULL,
    block_key TEXT NOT NULL,
    quote TEXT NOT NULL,
    start INTEGER NOT NULL,
    "end" INTEGER NOT NULL,
    reason TEXT NOT NULL CHECK (reason IN ('cited', 'restated', 'other')),
    note TEXT
);
INSERT INTO skip_next (id, run_id, text_id, path, block_key, quote, start, "end", reason, note)
SELECT id, run_id, text_id, path, block_key, quote, start, "end", reason, note FROM skip;
DROP TABLE skip;
ALTER TABLE skip_next RENAME TO skip;
CREATE INDEX skip_text ON skip (text_id, block_key);
