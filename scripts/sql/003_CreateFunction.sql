CREATE OR REPLACE FUNCTION external_url_archiver.notify_archive_urls(start_id INTEGER)
RETURNS INTEGER AS $$
    DECLARE
        rec RECORD;
        last_notified_id INTEGER;

    BEGIN
        FOR rec IN SELECT * FROM external_url_archiver.internet_archive_urls WHERE id >= start_id ORDER BY id LIMIT 2
        LOOP
            PERFORM pg_notify('archive_urls', row_to_json(rec)::text);
            last_notified_id := rec.id;
        END LOOP;
        RETURN last_notified_id;
    END;
$$ LANGUAGE 'plpgsql';