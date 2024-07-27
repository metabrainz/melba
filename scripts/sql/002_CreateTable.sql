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

CREATE TABLE external_url_archiver.last_unprocessed_rows (
        id_column           INTEGER,
        table_name          VARCHAR
);

-- Insert the latest row from edit_data table to begin the poller
INSERT INTO external_url_archiver.last_unprocessed_rows (id_column, table_name)
SELECT edit, 'edit_data'
FROM (
    SELECT DISTINCT ON (edit) edit
    FROM edit_data
    ORDER BY edit  DESC LIMIT 1
) AS latest_edit_data;

-- Insert the latest row from edit_note table to begin the poller
INSERT INTO external_url_archiver.last_unprocessed_rows (id_column, table_name)
SELECT id, 'edit_note'
FROM (
    SELECT DISTINCT ON (id) id
    FROM edit_note
    ORDER BY id  DESC LIMIT 1
) AS latest_edit_note;
