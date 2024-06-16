#!/bin/bash

# Load .env
export $(grep -v '^#' ../.env | xargs)

schema_name="external_url_archiver"

drop_schema() {
    echo "Dropping schema $schema_name..."
    PGPASSWORD=$PGPASSWORD psql -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE -c "DROP SCHEMA IF EXISTS $schema_name CASCADE;"
    if [ $? -ne 0 ]; then
        echo "Failed to drop schema $schema_name"
        exit 1
    fi
    echo "Schema $schema_name dropped successfully."
}

# Function to run the SQL scripts
run_sql_scripts() {
    ./init_db.sh
    if [ $? -ne 0 ]; then
        echo "Failed to run SQL scripts"
        exit 1
    fi
}

# Drop the schema and run the SQL scripts
drop_schema
run_sql_scripts

echo "Schema recreated and SQL scripts executed successfully."