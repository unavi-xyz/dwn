#!/bin/bash

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Loop through each SQL file in the script's directory, in alphabetical order
for SQL_FILE in "$SCRIPT_DIR"/*.sql; do
    echo "Running $SQL_FILE..."
    mysql -u root < "$SQL_FILE"
done
