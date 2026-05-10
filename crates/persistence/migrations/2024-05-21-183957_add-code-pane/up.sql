CREATE TABLE code_panes (
  id INTEGER PRIMARY KEY NOT NULL,
  kind TEXT NOT NULL DEFAULT 'code' CHECK (kind = 'code'),
  local_path BLOB,

  FOREIGN KEY (id, kind) REFERENCES pane_leaves (pane_node_id, kind)
);
