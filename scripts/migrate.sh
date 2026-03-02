#!/usr/bin/env bash
set -euo pipefail

# Runs all SQL migrations in order against the local database.
# Usage: ./scripts/migrate.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Load .env if it exists
if [[ -f "$PROJECT_ROOT/.env" ]]; then
    set -a
    source "$PROJECT_ROOT/.env"
    set +a
fi

DATABASE_URL="${DATABASE_URL:-postgres://pixelwar:pixelwar@localhost:5432/pixelwar}"

echo "==> Running migrations against $DATABASE_URL"

for migration in "$PROJECT_ROOT"/migrations/*.sql; do
    echo "    applying $(basename "$migration") ..."
    psql "$DATABASE_URL" -f "$migration" --quiet --set ON_ERROR_STOP=on
done

echo "==> Migrations complete."
