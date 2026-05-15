CREATE TABLE object_metadata (
    id INTEGER NOT NULL PRIMARY KEY,
    is_pending BOOLEAN NOT NULL,
    object_type TEXT CHECK(object_type IN ('WORKFLOW')) NOT NULL,
    revision_ts INTEGER,
    stable_object_id TEXT,
    client_id TEXT,
    local_object_id INTEGER NOT NULL,
    last_edited_by TEXT,
    author_id INTEGER,
    retry_count INTEGER NOT NULL
);

INSERT INTO object_metadata SELECT null, false, 'WORKFLOW', null, null, null, id, null, null, 0 from workflows;
