CREATE SCHEMA IF NOT EXISTS musicbrainz;

CREATE TABLE musicbrainz.edit_note (
    id                  SERIAL,
    editor              INTEGER NOT NULL, -- references editor.id
    edit                INTEGER NOT NULL, -- references edit.id
    text                TEXT NOT NULL,
    post_time            TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Spammer editor
INSERT INTO musicbrainz.edit_note (id, editor, edit, text, post_time) VALUES (771, 1, 85521, 'This edit moderation clashes with an existing item in the database.', NULL);
