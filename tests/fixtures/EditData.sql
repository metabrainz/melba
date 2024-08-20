CREATE SCHEMA IF NOT EXISTS musicbrainz;

CREATE TABLE musicbrainz.edit_data (
    edit                INTEGER NOT NULL, -- PK, references edit.id
    data                JSONB NOT NULL
);

-- Spammer editor
INSERT INTO musicbrainz.edit_data (edit, data) VALUES (21965, '{"new": {"sort_name": "Animals, The"}, "old": {"sort_name": "The Animals"}, "entity": {"id": "1433", "name": "The Animals"}}');
