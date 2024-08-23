CREATE SCHEMA IF NOT EXISTS musicbrainz;

CREATE TABLE musicbrainz.edit
(
    id                  SERIAL,
    editor              INTEGER NOT NULL, -- references editor.id
    type                SMALLINT NOT NULL,
    status              SMALLINT NOT NULL,
    autoedit            SMALLINT NOT NULL DEFAULT 0,
    open_time            TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    close_time           TIMESTAMP WITH TIME ZONE,
    expire_time          TIMESTAMP WITH TIME ZONE NOT NULL,
    language            INTEGER, -- references language.id
    quality             SMALLINT NOT NULL DEFAULT 1
);