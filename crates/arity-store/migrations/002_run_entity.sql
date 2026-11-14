CREATE TABLE run_entity (
    run_id INTEGER NOT NULL REFERENCES run (id),
    local_id TEXT NOT NULL,
    entity_id INTEGER NOT NULL REFERENCES entity (id),
    PRIMARY KEY (run_id, local_id)
);
