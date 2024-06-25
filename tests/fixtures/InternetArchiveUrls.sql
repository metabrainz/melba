CREATE SCHEMA external_url_archiver;

CREATE TABLE external_url_archiver.internet_archive_urls (
        id                  serial,
        url                 text,
        job_id              text, -- response returned when we make the URL save request
        from_table          VARCHAR, -- table from where URL is taken
        from_table_id       INTEGER, -- id of the row from where the URL is taken
        created_at          TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        retry_count         INTEGER, -- keeps track of number of retries made for the URL
        is_saved            boolean
);

CREATE FUNCTION external_url_archiver.notify_archive_urls(start_id INTEGER)
RETURNS INTEGER AS $$
    DECLARE
        rec RECORD;
        count INTEGER := 0;
    BEGIN
        FOR rec IN SELECT * FROM internet_archive_urls WHERE id >= start_id ORDER BY id LIMIT 2
        LOOP
            PERFORM pg_notify('archive_urls', row_to_json(rec)::text);
            count := count + 1;
        END LOOP;
        RETURN count;
    END;
$$ LANGUAGE 'plpgsql';

INSERT INTO external_url_archiver.internet_archive_urls(url, from_table, from_table_id, retry_count, is_saved) VALUES
('https://blackpaintingsdiscography.bndcamp.com/album/asmodea', 'edit_note', 70000000, 0, false),
('https://blackpaintingsdiscography.bandcamp.com/album/the-dog', 'edit_note', 70000003, 0, false),
('http://finaltape.bandcamp.com/', 'edit_data', 48470688, 0, false),
('https://myspace.com/killbillg', 'edit_data', 48470708, 0, false);