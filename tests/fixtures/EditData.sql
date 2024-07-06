CREATE TABLE edit_data (
    edit                INTEGER NOT NULL, -- PK, references edit.id
    data                JSONB NOT NULL
);

INSERT INTO edit_data(edit, data) VALUES
(1, ' {"new": {"url": "https://twitter.com/asceaaspade"}, "old": {"url": "https://twitter.com/AceASpadeWorld1"}, "entity": {"id": 6493379, "gid": "96386921-fdec-4fab-809a-c2ad465e12e7","name": "https://twitter.com/AceASpadeWorld1"}, "affects": 1, "is_merge": 0} ' ),
(2, '{"ended": 0, "type0": "artist", "type1": "url", "entity0": {"id" : 2096218, "gid": "90a41009-8496-49dd-817c-88f4c9416a2f", "name": "The Meadows"}, "entity1": {"id": 7718277, "gid": "77779934-81a5-43a1-81ec-ba04919f873f", "name": "https://www.discogs.com/artist/296705"}, "entity_id": 3134570, "link_type": {"id": 180, "name": "discogs", "link_phrase": "Discogs", "long_link_phrase": "has a Discogs page at", "reverse_link_phrase": "Discogs page for"}, "edit_version": 2}'),
(3, '{}')