CREATE TABLE entity_name_next (
    entity_id INTEGER NOT NULL REFERENCES entity (id),
    name TEXT NOT NULL,
    run_id INTEGER NOT NULL REFERENCES run (id),
    PRIMARY KEY (entity_id, name, run_id)
);
INSERT OR IGNORE INTO entity_name_next (entity_id, name, run_id)
SELECT run_entity.entity_id, json_extract(item.value, '$.label'), run_entity.run_id
FROM run_entity
JOIN run ON run.id = run_entity.run_id
JOIN json_each(run.submission, '$.entities') AS item
    ON json_extract(item.value, '$.id') = run_entity.local_id;
INSERT OR IGNORE INTO entity_name_next (entity_id, name, run_id)
SELECT entity_name.entity_id, entity_name.name, entity.run_id
FROM entity_name
JOIN entity ON entity.id = entity_name.entity_id
WHERE NOT EXISTS (
    SELECT 1 FROM entity_name_next AS next
    WHERE next.entity_id = entity_name.entity_id AND next.name = entity_name.name
);
DROP TABLE entity_name;
ALTER TABLE entity_name_next RENAME TO entity_name;
