CREATE SCHEMA IF NOT EXISTS external_url_archiver;


CREATE TABLE external_url_archiver.last_unprocessed_rows (
        id_column           INTEGER,
        table_name          VARCHAR
);