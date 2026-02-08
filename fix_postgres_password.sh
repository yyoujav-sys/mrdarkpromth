#!/bin/bash
echo "Fixing PostgreSQL password..."
docker exec mr_darkpromth_postgres psql -U postgres -d mr_darkpromth -c "ALTER USER postgres WITH PASSWORD 'postgres';"
echo "PostgreSQL password fixed!"
