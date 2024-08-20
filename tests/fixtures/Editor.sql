CREATE SCHEMA IF NOT EXISTS musicbrainz;

CREATE TABLE musicbrainz.editor
(
    id                  SERIAL,
    name                VARCHAR(64) NOT NULL,
    privs               INTEGER DEFAULT 0,
    email               VARCHAR(64) DEFAULT NULL,
    website             VARCHAR(255) DEFAULT NULL,
    bio                 TEXT DEFAULT NULL,
    member_since        TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    email_confirm_date  TIMESTAMP WITH TIME ZONE,
    last_login_date     TIMESTAMP WITH TIME ZONE DEFAULT now(),
    last_updated        TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    birth_date          DATE,
    gender              INTEGER, -- references gender.id
    area                INTEGER, -- references area.id
    password            VARCHAR(128) NOT NULL,
    ha1                 CHAR(32) NOT NULL,
    deleted             BOOLEAN NOT NULL DEFAULT FALSE
);
-- Spammer
INSERT INTO musicbrainz.editor (id, name, privs, email, website, bio, member_since, email_confirm_date, last_login_date, last_updated, birth_date, gender, area, password, ha1, deleted) VALUES (1, 'Anonymous', 4096, '', NULL, NULL, NULL, '2009-10-18 18:20:17.333759+00', '2024-05-04 00:17:56.699735+00', '2018-03-15 08:15:36.728395+00', NULL, NULL, NULL, '{CLEARTEXT}mb', 'fad9cdfaf96a2ddb0eceb7b07d269bea', false);
