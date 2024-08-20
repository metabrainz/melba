CREATE SCHEMA IF NOT EXISTS external_url_archiver;

CREATE TABLE external_url_archiver.internet_archive_urls (
        id                  serial,
        url                 text,
        job_id              text, -- response returned when we make the URL save request
        from_table          VARCHAR, -- table from where URL is taken
        from_table_id       INTEGER, -- id of the row from where the URL is taken
        created_at          TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
        retry_count         INTEGER, -- keeps track of number of retries made for the URL
        status              INTEGER DEFAULT 1, -- not started
        status_message      text -- keeps the status message of archival of URL
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
