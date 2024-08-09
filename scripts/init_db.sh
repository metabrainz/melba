#!/bin/bash

# Load environment variables from .env file
export $(grep -v '^#' ../.env | xargs)

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
