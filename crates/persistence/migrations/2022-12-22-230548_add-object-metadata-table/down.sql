DROP TABLE object_metadata;

ALTER TABLE workflows ADD is_pending BOOLEAN NOT NULL DEFAULT FALSE;
ALTER TABLE workflows ADD server_id BIGINTEGER;
