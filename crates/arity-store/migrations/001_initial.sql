CREATE TABLE agent (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('person', 'software')),
    name TEXT NOT NULL,
    version TEXT,
    orcid TEXT
);
CREATE UNIQUE INDEX agent_identity
    ON agent (kind, name, COALESCE(version, ''), COALESCE(orcid, ''));

CREATE TABLE work (
    id INTEGER PRIMARY KEY,
    doi TEXT UNIQUE,
    pmid TEXT UNIQUE,
    pmcid TEXT UNIQUE,
    title TEXT NOT NULL,
    year INTEGER,
    license TEXT,
    is_preprint INTEGER NOT NULL DEFAULT 0 CHECK (is_preprint IN (0, 1)),
    metadata TEXT NOT NULL,
    added_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE queue (
    work_id INTEGER PRIMARY KEY REFERENCES work (id),
    state TEXT NOT NULL CHECK (state IN ('queued', 'fetched', 'read', 'failed', 'skipped')),
    query TEXT,
    note TEXT,
    added_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX queue_state ON queue (state, added_at);

CREATE TABLE retrieval (
    id INTEGER PRIMARY KEY,
    work_id INTEGER NOT NULL REFERENCES work (id),
    role TEXT NOT NULL CHECK (role IN ('metadata', 'full_text', 'pdf', 'supplement')),
    supplement_key TEXT,
    url TEXT NOT NULL,
    media_type TEXT,
    sha256 TEXT NOT NULL,
    bytes INTEGER NOT NULL,
    software TEXT NOT NULL,
    retrieved_at TEXT NOT NULL
);
CREATE INDEX retrieval_work ON retrieval (work_id, role);

CREATE TABLE text (
    id INTEGER PRIMARY KEY,
    work_id INTEGER NOT NULL REFERENCES work (id),
    retrieval_id INTEGER NOT NULL REFERENCES retrieval (id),
    sha256 TEXT NOT NULL UNIQUE,
    parser TEXT NOT NULL,
    document TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE block (
    text_id INTEGER NOT NULL REFERENCES text (id),
    key TEXT NOT NULL,
    position INTEGER NOT NULL,
    kind TEXT NOT NULL,
    label TEXT,
    section TEXT,
    content TEXT NOT NULL,
    PRIMARY KEY (text_id, key)
);

CREATE TABLE notice (
    id INTEGER PRIMARY KEY,
    work_id INTEGER NOT NULL REFERENCES work (id),
    kind TEXT NOT NULL,
    reported_type TEXT NOT NULL,
    source TEXT NOT NULL,
    notice_doi TEXT,
    notice_pmid TEXT,
    notice_citation TEXT,
    date TEXT,
    source_record TEXT,
    retrieved_at TEXT NOT NULL
);
CREATE UNIQUE INDEX notice_identity ON notice (
    work_id, kind, source, COALESCE(notice_doi, ''), COALESCE(notice_pmid, ''), COALESCE(date, '')
);

CREATE TABLE run (
    id INTEGER PRIMARY KEY,
    work_id INTEGER NOT NULL REFERENCES work (id),
    text_id INTEGER NOT NULL REFERENCES text (id),
    submission_sha256 TEXT NOT NULL UNIQUE,
    submission TEXT NOT NULL,
    reader_id INTEGER NOT NULL REFERENCES agent (id),
    responsible_id INTEGER NOT NULL REFERENCES agent (id),
    software TEXT NOT NULL,
    warnings TEXT NOT NULL,
    submitted_at TEXT NOT NULL
);

CREATE TABLE entity (
    id INTEGER PRIMARY KEY,
    kind TEXT NOT NULL,
    label TEXT NOT NULL,
    run_id INTEGER NOT NULL REFERENCES run (id)
);

CREATE TABLE entity_xref (
    curie TEXT PRIMARY KEY,
    entity_id INTEGER NOT NULL REFERENCES entity (id)
);

CREATE TABLE entity_name (
    entity_id INTEGER NOT NULL REFERENCES entity (id),
    name TEXT NOT NULL,
    PRIMARY KEY (entity_id, name)
);

CREATE TABLE property (
    key TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    value_type TEXT NOT NULL CHECK (value_type IN ('quantity', 'boolean', 'category')),
    unit TEXT,
    dimension TEXT,
    categories TEXT NOT NULL DEFAULT '[]',
    xrefs TEXT NOT NULL DEFAULT '[]',
    description TEXT,
    run_id INTEGER REFERENCES run (id)
);

CREATE TABLE report (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL REFERENCES run (id),
    work_id INTEGER NOT NULL REFERENCES work (id),
    local_id TEXT NOT NULL,
    type TEXT NOT NULL CHECK (type IN ('value', 'effect')),
    body TEXT NOT NULL,
    property_key TEXT REFERENCES property (key),
    number TEXT,
    low TEXT,
    high TEXT,
    unit TEXT,
    unit_text TEXT,
    coherent REAL,
    coherent_low REAL,
    coherent_high REAL,
    coherent_unit TEXT,
    bound TEXT,
    statistic TEXT,
    n INTEGER,
    boolean INTEGER,
    category TEXT,
    uncertainty_kind TEXT,
    uncertainty_number TEXT,
    uncertainty_low TEXT,
    uncertainty_high TEXT,
    uncertainty_level TEXT,
    read_from_figure INTEGER NOT NULL DEFAULT 0 CHECK (read_from_figure IN (0, 1)),
    figure_block TEXT,
    figure_method TEXT,
    figure_resolution TEXT,
    target_entity_id INTEGER REFERENCES entity (id),
    target_property_key TEXT REFERENCES property (key),
    direction TEXT,
    relation TEXT,
    magnitude_measure TEXT,
    magnitude_number TEXT,
    magnitude_unit TEXT,
    comparator TEXT,
    significance TEXT,
    p_comparison TEXT,
    p_value TEXT,
    test TEXT,
    method_label TEXT,
    method_xrefs TEXT NOT NULL DEFAULT '[]',
    note TEXT,
    UNIQUE (run_id, local_id)
);
CREATE INDEX report_property ON report (property_key);
CREATE INDEX report_target ON report (target_entity_id, target_property_key);
CREATE INDEX report_work ON report (work_id);

CREATE TABLE report_entity (
    report_id INTEGER NOT NULL REFERENCES report (id),
    role TEXT NOT NULL CHECK (role IN ('subject', 'agent')),
    position INTEGER NOT NULL,
    entity_id INTEGER NOT NULL REFERENCES entity (id),
    dose_number TEXT,
    dose_unit TEXT,
    dose_unit_text TEXT,
    dose_coherent REAL,
    PRIMARY KEY (report_id, role, position)
);
CREATE INDEX report_entity_entity ON report_entity (entity_id, role);

CREATE TABLE condition (
    report_id INTEGER NOT NULL REFERENCES report (id),
    position INTEGER NOT NULL,
    role TEXT,
    entity_id INTEGER REFERENCES entity (id),
    text TEXT,
    property_key TEXT REFERENCES property (key),
    number TEXT,
    low TEXT,
    high TEXT,
    unit TEXT,
    unit_text TEXT,
    coherent REAL,
    boolean INTEGER,
    category TEXT,
    PRIMARY KEY (report_id, position)
);

CREATE TABLE evidence (
    id INTEGER PRIMARY KEY,
    run_id INTEGER NOT NULL REFERENCES run (id),
    text_id INTEGER NOT NULL REFERENCES text (id),
    report_id INTEGER REFERENCES report (id),
    entity_id INTEGER REFERENCES entity (id),
    path TEXT NOT NULL,
    block_key TEXT NOT NULL,
    quote TEXT NOT NULL,
    start INTEGER NOT NULL,
    "end" INTEGER NOT NULL,
    CHECK ((report_id IS NULL) != (entity_id IS NULL))
);
CREATE INDEX evidence_report ON evidence (report_id);
CREATE INDEX evidence_text ON evidence (text_id, block_key);

CREATE TABLE check_event (
    id INTEGER PRIMARY KEY,
    report_id INTEGER NOT NULL REFERENCES report (id),
    state TEXT NOT NULL CHECK (state IN ('unchecked', 'confirmed', 'rejected')),
    person_id INTEGER NOT NULL REFERENCES agent (id),
    note TEXT,
    at TEXT NOT NULL
);
CREATE INDEX check_event_report ON check_event (report_id, at);

CREATE VIEW report_state AS
SELECT
    report.id AS report_id,
    COALESCE(
        (
            SELECT check_event.state
            FROM check_event
            WHERE check_event.report_id = report.id
            ORDER BY check_event.at DESC, check_event.id DESC
            LIMIT 1
        ),
        'unchecked'
    ) AS state
FROM report;
