#!/usr/bin/env bash
# Seed script for PostgreSQL integration tests.
# Idempotent — safe to run multiple times.
#
# Expected environment:
#   PG on localhost:54320, user=postgres, password=password, db=testdb
#   (matches the GitHub Actions service and existing integration tests)
set -euo pipefail

PGHOST="${PGHOST:-127.0.0.1}"
PGPORT="${PGPORT:-54320}"
PGUSER="${PGUSER:-postgres}"
PGPASSWORD="${PGPASSWORD:-password}"
export PGHOST PGPORT PGUSER PGPASSWORD

# The existing integration tests expect a database called "testdb" with their
# own tables in the public schema. We don't touch those — our parity tests use
# a dedicated "test_schema" within the same database.

echo "==> Seeding primary database (testdb)..."
psql -d testdb -f "$(dirname "$0")/postgres_seed.sql"

# Secondary database for multi-database testing
echo "==> Creating secondary database (superdba_test_secondary)..."
psql -d postgres -c "
  SELECT 'exists' FROM pg_database WHERE datname = 'superdba_test_secondary'
" | grep -q exists || createdb superdba_test_secondary

echo "==> Seeding secondary database..."
psql -d superdba_test_secondary -c "
  CREATE SCHEMA IF NOT EXISTS secondary_schema;

  CREATE TABLE IF NOT EXISTS secondary_schema.remote_data (
    id SERIAL PRIMARY KEY,
    value TEXT NOT NULL
  );

  INSERT INTO secondary_schema.remote_data (value)
    SELECT 'row_' || g
    FROM generate_series(1, 5) g
    WHERE NOT EXISTS (SELECT 1 FROM secondary_schema.remote_data LIMIT 1);
"

echo "==> PostgreSQL seed complete."
