#!/bin/bash

# Exit immediately if a command exits with a non-zero status.
set -e

# Create the test database
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE DATABASE mr_darkpromth;
EOSQL
