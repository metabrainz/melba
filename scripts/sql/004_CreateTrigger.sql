CREATE OR REPLACE FUNCTION external_url_archiver.log_edit_data()
RETURNS TRIGGER AS $$
    DECLARE
        url_value TEXT;
    BEGIN
    --  Edit Type: Add Relationship (type1 is URL)
        IF NEW.data ? 'type1' AND NEW.data->>'type1' = 'url' THEN
            IF NEW.data ? 'entity1' THEN
                url_value := NEW.data->'entity1'->>'name';
            END IF;

    --  Edit Type: Add Relationship (type0 is URL)
        ELSIF NEW.data ? 'type0' AND NEW.data->>'type0' = 'url' THEN
            IF NEW.data ? 'entity0' THEN
                url_value := NEW.data->'entity0'->>'name';
            END IF;

    -- Edit Type: Edit URL TODO: what fields to save though?
        ELSIF NEW.data ? 'new' AND NEW.data->'new' ? 'url' THEN
            url_value := NEW.data->'new'->>'url';

        END IF;

        IF url_value IS NOT NULL THEN
                INSERT INTO external_url_archiver.internet_archive_urls (
                    url, job_id, from_table, from_table_id, created_at, retry_count, is_saved
                ) VALUES (
                    url_value,  -- url
                    NULL,       -- job_id
                    'edit_data', -- from_table
                    NEW.edit,   -- from_table_id
                    NOW(),      -- created_at
                    0,          -- retry_count
                    FALSE       -- is_saved
                );
        END IF;
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_extract_nested_url_and_log
AFTER INSERT ON edit_data
FOR EACH ROW
EXECUTE FUNCTION external_url_archiver.log_edit_data();