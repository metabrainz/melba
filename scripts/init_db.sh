#!/bin/bash

# Check if yq is installed
if ! command -v yq &> /dev/null; then
    echo "Error: 'yq' is not installed. Please install 'yq' to run this script."
    exit 1
fi


CONFIG_FILE="../config/development.yaml"

PG_HOST=$(yq -r '.database.pg_host' "$CONFIG_FILE")
PG_PORT=$(yq -r '.database.pg_port' "$CONFIG_FILE")
PG_USER=$(yq -r '.database.pg_user' "$CONFIG_FILE")
PG_PASSWORD=$(yq -r '.database.pg_password' "$CONFIG_FILE")
PG_DATABASE=$(yq -r '.database.pg_database' "$CONFIG_FILE")
DATABASE_URL=$(yq -r '.database.database_url' "$CONFIG_FILE")

export PGHOST=$PG_HOST
export PGPORT=$PG_PORT
export PGUSER=$PG_USER
export PGPASSWORD=$PG_PASSWORD
export PGDATABASE=$PG_DATABASE

sql_dir="sql"

# Array of SQL files
sql_files=("001_CreateSchema.sql" "002_CreateTable.sql" "003_CreateFunction.sql" "004_CreateTrigger.sql" "005_CreateIndex.sql")

# Add the appropriate populate SQL file based on RUN_MODE
if [ "$RUN_MODE" = "development" ]; then
    sql_files+=("006_PopulateTableDEV.sql")
else
    sql_files+=("006_PopulateTable.sql")
fi

# Function to run a single SQL file
run_sql_file() {
    local file=$1
    local filepath="$sql_dir/$file"
    echo "Running $filepath..."
    PGPASSWORD=$PGPASSWORD psql -h $PGHOST -p $PGPORT -U $PGUSER -d $PGDATABASE -f $filepath
    if [ $? -ne 0 ]; then
        echo "Failed to run $filepath"
        exit 1
    fi
}

# Run all SQL files
for file in "${sql_files[@]}"; do
    run_sql_file $file
done

echo "All SQL scripts executed successfully."
