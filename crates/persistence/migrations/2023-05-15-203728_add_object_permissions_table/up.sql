-- Create the object_permissions table
CREATE TABLE object_permissions (
  id INTEGER NOT NULL PRIMARY KEY,
  object_metadata_id INTEGER NOT NULL REFERENCES object_metadata(id) ON DELETE CASCADE,
  subject_type TEXT NOT NULL,
  subject_id INTEGER,
  permissions_last_updated_at BIGINTEGER NOT NULL DEFAULT 0
);

INSERT INTO object_permissions(object_metadata_id, subject_type)
SELECT id, 'USER' FROM object_metadata;
