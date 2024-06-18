CREATE OR REPLACE FUNCTION external_url_archiver.notify_archive_urls(start_id INTEGER)
RETURNS INTEGER AS $$
    DECLARE
        rec RECORD;
        count INTEGER := 0;
    BEGIN
        FOR rec IN SELECT * FROM external_url_archiver.internet_archive_urls WHERE id >= start_id ORDER BY id LIMIT 2
        LOOP
            PERFORM pg_notify('archive_urls', row_to_json(rec)::text);
            count := count + 1;
        END LOOP;
        RETURN count;
    END;
$$ LANGUAGE 'plpgsql';