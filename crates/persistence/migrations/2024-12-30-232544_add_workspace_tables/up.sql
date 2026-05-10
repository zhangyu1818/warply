-- Your SQL goes here
CREATE TABLE workspaces (
    id integer NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    server_uid TEXT NOT NULL UNIQUE
);
