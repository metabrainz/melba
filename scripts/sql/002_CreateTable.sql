CREATE TYPE external_url_archiver.url_status AS ENUM ('Waiting', 'Processing', 'WaitingStatus', 'Archived', 'Errored', 'Failed');

CREATE TABLE external_url_archiver.internet_archive_urls (
        id                  serial,
        url                 text,
        job_id              text, -- response returned when we make the URL save request
        from_table          VARCHAR, -- table from where URL is taken
        from_table_id       INTEGER, -- id of the row from where the URL is taken
        created_at          TIMESTAMP WITH TIME ZONE DEFAULT NOW(),



        -- The status of the archiving
        status              external_url_archiver.url_status DEFAULT 'waiting',
        status_message      text -- keeps the status message of archival of URL

        -- The number of time the url has been submitted to IA for archival
        try_count           INTEGER DEFAULT 0 NOT NULL,

        -- The timestamp of when the url can be retried
        retry_after         TIMESTAMP WITH TIME ZONE DEFAULT NOW() NOT NULL
);

CREATE TABLE external_url_archiver.last_unprocessed_rows (
        id_column           INTEGER,
        table_name          VARCHAR
);