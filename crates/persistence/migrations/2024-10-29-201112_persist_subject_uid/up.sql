CREATE TABLE IF NOT EXISTS object_permissions_new (
  id INTEGER NOT NULL PRIMARY KEY,
  object_metadata_id INTEGER NOT NULL REFERENCES object_metadata(id) ON DELETE CASCADE,
  subject_type TEXT NOT NULL,
  subject_id TEXT,
  subject_uid TEXT NOT NULL
);

INSERT INTO object_permissions_new (id, object_metadata_id, subject_type, subject_id, subject_uid)
SELECT id, object_metadata_id, subject_type, subject_id, subject_id
FROM object_permissions
WHERE subject_type = 'USER' AND subject_id IS NOT NULL;

DROP TABLE object_permissions;

ALTER TABLE object_permissions_new RENAME TO object_permissions;
