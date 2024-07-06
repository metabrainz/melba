CREATE TABLE edit
(
    id                  SERIAL,
    editor              INTEGER NOT NULL, -- references editor.id
    type                SMALLINT NOT NULL
);

INSERT INTO edit(id, editor, type) VALUES
(1,3,91),
(2,1, 90),
(3,2, 12)
