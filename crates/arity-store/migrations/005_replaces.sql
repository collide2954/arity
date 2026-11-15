ALTER TABLE run ADD COLUMN replaces INTEGER REFERENCES run (id);
CREATE UNIQUE INDEX run_replaces ON run (replaces) WHERE replaces IS NOT NULL;
