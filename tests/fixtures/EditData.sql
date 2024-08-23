CREATE SCHEMA IF NOT EXISTS musicbrainz;

CREATE TABLE musicbrainz.edit_data (
    edit                INTEGER NOT NULL, -- PK, references edit.id
    data                JSONB NOT NULL
);