CREATE OR REPLACE FUNCTION external_url_archiver.notify_archive_urls(url_id INTEGER)
RETURNS VOID AS $$
    DECLARE
        rec RECORD;

    BEGIN
        SELECT * INTO rec
        FROM external_url_archiver.internet_archive_urls
        WHERE id = url_id
        ORDER BY id;

        IF FOUND THEN
            PERFORM pg_notify('archive_urls', row_to_json(rec)::text);
        END IF;
    END;
$$ LANGUAGE 'plpgsql';
