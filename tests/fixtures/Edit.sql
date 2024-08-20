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

-- Entry for spammer editor
INSERT INTO musicbrainz.edit (id, editor, type, status, autoedit, open_time, close_time, expire_time, language, quality) VALUES (21965, 1, 207, 3, 0, '2000-11-14 11:51:55+00', '2000-11-16 11:51:55+00', '2000-11-16 03:51:55+00', NULL, 1);
