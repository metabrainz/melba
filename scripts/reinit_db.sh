#!/bin/bash

# Check if yq is installed
if ! command -v yq &> /dev/null; then
    echo "Error: 'yq' is not installed. Please install 'yq' to run this script."
    exit 1
fi

export $(grep -v '^#' ../.env | xargs)

CONFIG_FILE="../config/${RUN_MODE}.toml"

PG_HOST=$(yq -p toml -r '.database.pg_host' "$CONFIG_FILE")
PG_PORT=$(yq -p toml -r '.database.pg_port' "$CONFIG_FILE")
PG_USER=$(yq -p toml -r '.database.pg_user' "$CONFIG_FILE")
PG_PASSWORD=$(yq -p toml -r '.database.pg_password' "$CONFIG_FILE")
PG_DATABASE=$(yq -p toml -r '.database.pg_database' "$CONFIG_FILE")
DATABASE_URL=$(yq -p toml -r '.database.database_url' "$CONFIG_FILE")

export PGHOST=$PG_HOST
export PGPORT=$PG_PORT
export PGUSER=$PG_USER
export PGPASSWORD=$PG_PASSWORD
export PGDATABASE=$PG_DATABASE

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