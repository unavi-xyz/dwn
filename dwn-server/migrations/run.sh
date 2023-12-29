#!/usr/bin/env bash

# Initialize flags
DB_NAME=dwn
DROP_DB=false
ENV_FILE=.env

SCRIPT_DIR="./dwn-server/migrations"

# Function to parse arguments
parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --drop)
                DROP_DB=true
                ;;
            --env)
                ENV_FILE=$2
                shift
                ;;
            *)
                echo "Invalid argument: $1"
                exit 1
                ;;
        esac
        shift
    done
}

# Function to run MySQL with or without password
run_mysql() {
    local db_user=$1
    local db_pass=$2
    local db_host=$3
    local db_name=$4
    local sql_file=$5

    if [ -z "$db_pass" ]; then
        mysql -u "$db_user" -h "$db_host" "$db_name" < "$sql_file"
    else
        mysql -u "$db_user" -p"$db_pass" -h "$db_host" "$db_name" < "$sql_file"
    fi
}

drop_tables() {
    # Get all tables
    echo "SHOW TABLES;" > "$SCRIPT_DIR/tables.sql"
    run_mysql "$DB_USER" "$DB_PASS" "$DB_HOST" "$DB_NAME" "$SCRIPT_DIR/tables.sql" > "$SCRIPT_DIR/tables.txt"
    rm "$SCRIPT_DIR/tables.sql"

    # Remove the first line
    sed -i '1d' "$SCRIPT_DIR/tables.txt"

    # Drop each table
    while read -r table; do
        echo "Dropping $table..."
        echo "DROP TABLE $table;" > "$SCRIPT_DIR/drop.sql"
        run_mysql "$DB_USER" "$DB_PASS" "$DB_HOST" "$DB_NAME" "$SCRIPT_DIR/drop.sql"
        rm "$SCRIPT_DIR/drop.sql"
    done < "$SCRIPT_DIR/tables.txt"

    rm "$SCRIPT_DIR/tables.txt"
}

# Parse the command line arguments
parse_args "$@"

# Load .env
export $(cat "$ENV_FILE" | xargs)

if [ "$DROP_DB" = true ]; then
    drop_tables
fi

# Create the database if it doesn't exist
echo "CREATE DATABASE IF NOT EXISTS $DB_NAME;" > "$SCRIPT_DIR/create.sql"
run_mysql "$DB_USER" "$DB_PASS" "$DB_HOST" "" "$SCRIPT_DIR/create.sql"
rm "$SCRIPT_DIR/create.sql"

# Loop through each SQL file in the script's directory, in alphabetical order
for SQL_FILE in "$SCRIPT_DIR"/*.sql; do
    echo "Running $SQL_FILE..."
    run_mysql "$DB_USER" "$DB_PASS" "$DB_HOST" "$DB_NAME" "$SQL_FILE"
done
