#!/bin/bash

# Load environment variables from .env file
export $(grep -v '^#' ../.env | xargs)

# Get the project root directory using cargo
project_root=$(cargo locate-project --workspace --message-format plain | xargs dirname)

dump_dir="$project_root/tests/fixtures"
mkdir -p "$dump_dir"

tables=("external_url_archiver.internet_archive_urls" "external_url_archiver.last_unprocessed_rows" "musicbrainz.edit_data" "musicbrainz.edit_note" "musicbrainz.edit" "musicbrainz.editor")

# Array to keep track of created backup tables
backup_tables=()

# Function to create, verify, dump a new table
dump_table() {
    local table=$1
    local schema_name=$(echo $table | awk -F '.' '{print $1}')
    local base_name=$(echo $table | awk -F '.' '{print $2}')
    local new_table_name="backup_${base_name}"
    local filepath="$dump_dir/${base_name}_dump.sql"

    echo "Attempting to create new table $schema_name.$new_table_name for $table..."

    # Create the new table in the correct schema
    if [[ "$table" == "musicbrainz.edit_data" ]]; then
        PGPASSWORD=$PGPASSWORD psql -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE -c "
        CREATE TABLE $schema_name.$new_table_name AS
        SELECT * FROM $table WHERE edit >= 111450838 ORDER BY edit LIMIT 1000;
        "
    elif [[ "$table" == "musicbrainz.edit_note" ]]; then
        PGPASSWORD=$PGPASSWORD psql -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE -c "
        CREATE TABLE $schema_name.$new_table_name AS
        SELECT * FROM $table WHERE id >= 71024901 ORDER BY id LIMIT 1000;
        "
    elif [[ "$table" == "musicbrainz.edit" ]]; then
        PGPASSWORD=$PGPASSWORD psql -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE -c "
        CREATE TABLE $schema_name.$new_table_name AS
        SELECT * FROM $table WHERE id IN (SELECT edit FROM $schema_name.backup_edit_data);
        "
    elif [[ "$table" == "musicbrainz.editor" ]]; then
        echo "Selected editor IDs for $table:"
        PGPASSWORD=$PGPASSWORD psql -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE -c "
        CREATE TABLE $schema_name.$new_table_name AS
        SELECT * FROM $table WHERE id IN (
            SELECT editor FROM $schema_name.backup_edit_note
            UNION
            SELECT editor FROM $schema_name.edit WHERE id IN (SELECT edit FROM $schema_name.backup_edit_data)
        );
        "
    else
        PGPASSWORD=$PGPASSWORD psql -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE -c "
        CREATE TABLE $schema_name.$new_table_name AS
        SELECT * FROM $table LIMIT 1000;
        "
    fi

    if [ $? -ne 0 ]; then
        echo "Failed to create table $schema_name.$new_table_name for $table"
        exit 1
    fi

    echo "Table $schema_name.$new_table_name created successfully."

    # Track the created table
    backup_tables+=("$schema_name.$new_table_name")

    # Dump the new table
    touch "$filepath"
    echo "Dumping table $schema_name.$new_table_name to $filepath..."
    PGPASSWORD=$PGPASSWORD pg_dump -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE --data-only --table=$schema_name.$new_table_name --inserts --column-inserts --no-owner --no-privileges -f "$filepath"

    if [ $? -ne 0 ]; then
        echo "Failed to dump table $schema_name.$new_table_name"
        exit 1
    fi

    echo "Table $schema_name.$new_table_name dumped successfully to $filepath."

    # Replace the table name in the dump file
    sed -i "s/$schema_name.$new_table_name/$table/g" "$filepath"
}

# Dump the specific tables first
for table in "${tables[@]}"; do
    dump_table $table
done

# Drop all backup tables after dumping
for backup_table in "${backup_tables[@]}"; do
    echo "Dropping table $backup_table..."
    PGPASSWORD=$PGPASSWORD psql -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE -c "
    DROP TABLE IF EXISTS $backup_table;
    "

    if [ $? -ne 0 ]; then
        echo "Failed to drop table $backup_table"
        exit 1
    fi
done

echo "All tables dumped and backup tables dropped successfully."
