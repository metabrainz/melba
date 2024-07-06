CREATE TABLE edit_note (
    id                  SERIAL,
    editor              INTEGER NOT NULL, -- references editor.id
    edit                INTEGER NOT NULL, -- references edit.id
    text                TEXT NOT NULL,
    post_time            TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

INSERT INTO edit_note(editor, edit, text) VALUES
(1, 111451706, 'Same recording; one has incorrect duration'),
(2, 111451706, 'https://example.com'),
(3, 111451711, 'https://seraphitus-seraphita.bandcamp.com/album/cosmic-horrors-iii * https://f4.bcbits.com/img/a2736839660_10.jpg → Maximised to https://f4.bcbits.com/img/a2736839660_0.jpg -  MB: Enhanced Cover Art Uploads 2024.5.1 https://github.com/ROpdebee/mb-userscripts')