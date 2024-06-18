CREATE OR REPLACE FUNCTION external_url_archiver.log_edit_note_data_rows_trigger()
RETURNS TRIGGER AS $$
    BEGIN
        IF TG_TABLE_NAME = 'edit_data' THEN
            INSERT INTO external_url_archiver.log_edit_note_data_rows (from_table, from_table_id)
            VALUES ('edit_data', NEW.edit);
        ELSIF TG_TABLE_NAME = 'edit_note' THEN
            INSERT INTO external_url_archiver.log_edit_note_data_rows (from_table, from_table_id)
            VALUES ('edit_note', NEW.id);
        END IF;
        RETURN NEW;
    END
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_log_edit_data_for_urls
AFTER INSERT OR UPDATE ON musicbrainz.edit_data
FOR EACH ROW
EXECUTE FUNCTION external_url_archiver.log_edit_note_data_rows_trigger();

CREATE TRIGGER trigger_log_edit_note_for_urls
AFTER INSERT OR UPDATE ON musicbrainz.edit_note
FOR EACH ROW
EXECUTE FUNCTION external_url_archiver.log_edit_note_data_rows_trigger();