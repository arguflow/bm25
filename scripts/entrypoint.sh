#!/bin/bash

# Exit on subcommand errors
set -Eeuo pipefail

# List of extensions to possibly install (if a version variable is set)
declare -A extensions=(
  [pg_bm25]=${PG_BM25_VERSION:-}
)

# List of extensions that must be added to shared_preload_libraries
declare -A preload_names=(
  [pg_cron]=pg_cron
  [pg_net]=pg_net
  [pgaudit]=pgaudit
)

# Build the shared_preload_libraries list, only including extensions that are installed
# and have a preload name specified
for extension in "${!extensions[@]}"; do
  version=${extensions[$extension]}
  if [ -n "$version" ] && [[ -n "${preload_names[$extension]:-}" ]]; then
    preload_name=${preload_names[$extension]}
    shared_preload_list+="${preload_name},"
  fi
done
# Remove the trailing comma
shared_preload_list=${shared_preload_list%,}

# Update the PostgreSQL configuration
echo "pg_net.database_name = '$POSTGRES_DB'" >> "${PGDATA}/postgresql.conf"
echo "cron.database_name = '$POSTGRES_DB'" >> "${PGDATA}/postgresql.conf"
sed -i "s/^#shared_preload_libraries = .*/shared_preload_libraries = '$shared_preload_list'  # (change requires restart)/" "${PGDATA}/postgresql.conf"

# Setup the database role (the user passed via -e POSTGRES_USER to the Docker run command)
POSTGRES_USER_ROLE_EXISTS=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -tAc "SELECT 1 FROM pg_roles WHERE rolname='$POSTGRES_USER'")
if [ -z "$POSTGRES_USER_ROLE_EXISTS" ]; then
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
  CREATE ROLE $POSTGRES_USER WITH SUPERUSER LOGIN;
EOSQL
fi

# Setup the postgres role (a user named postgres is necessary for pg_net to work)
POSTGRES_ROLE_EXISTS=$(psql -U "$POSTGRES_USER" -d "$POSTGRES_DB" -tAc "SELECT 1 FROM pg_roles WHERE rolname='postgres'")
if [ -z "$POSTGRES_ROLE_EXISTS" ]; then
  psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
  CREATE ROLE postgres WITH SUPERUSER LOGIN;
EOSQL
fi

# Configure search_path to include paradedb schema for template1, and default to public (by listing it first)
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
  ALTER DATABASE "$POSTGRES_DB" SET search_path TO public,paradedb;
EOSQL

# Configure search_path to include paradedb schema for template1, so that it is
# inherited by all new databases created, and default to public (by listing it first)
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "template1" <<-EOSQL
  ALTER DATABASE template1 SET search_path TO public,paradedb;
EOSQL

# We need to restart the server for the changes above to be reflected
pg_ctl restart

echo "PostgreSQL is up - installing extensions..."

# Preinstall extensions for which a version is specified
for extension in "${!extensions[@]}"; do
  version=${extensions[$extension]}
  if [ -n "$version" ]; then
    PGPASSWORD=$POSTGRES_PASSWORD psql -c "CREATE EXTENSION IF NOT EXISTS $extension CASCADE" -d "$POSTGRES_DB" -U "$POSTGRES_USER" || echo "Failed to install extension $extension"
  fi
done
