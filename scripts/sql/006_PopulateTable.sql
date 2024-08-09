-- Insert the latest row from edit_data table to begin the poller
INSERT INTO external_url_archiver.last_unprocessed_rows (id_column, table_name)
SELECT edit, 'edit_data'
FROM (
    SELECT edit
    FROM edit_data
    ORDER BY edit  DESC LIMIT 1
) AS latest_edit_data;

-- Insert the latest row from edit_note table to begin the poller
INSERT INTO external_url_archiver.last_unprocessed_rows (id_column, table_name)
SELECT id, 'edit_note'
FROM (
    SELECT id
    FROM edit_note
    ORDER BY id  DESC LIMIT 1
) AS latest_edit_note;
