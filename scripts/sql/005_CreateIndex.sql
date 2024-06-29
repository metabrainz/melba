CREATE INDEX idx_from_table_from_table_id_desc
    ON external_url_archiver.internet_archive_urls (from_table, from_table_id DESC);