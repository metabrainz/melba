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

CREATE TABLE external_url_archiver.log_edit_note_data_rows (
    id SERIAL PRIMARY KEY,
    from_table TEXT NOT NULL,
    from_table_id INTEGER NOT NULL,
);