#!/usr/bin/env bash
set -euo pipefail

# Seeds the database with a test round so endpoints are usable immediately.
# Safe to run multiple times — skips if an active round already exists.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

if [[ -f "$PROJECT_ROOT/.env" ]]; then
    set -a
    source "$PROJECT_ROOT/.env"
    set +a
fi

DATABASE_URL="${DATABASE_URL:-postgres://pixelwar:pixelwar@localhost:5432/pixelwar}"

echo "==> Seeding database..."

psql "$DATABASE_URL" --quiet --set ON_ERROR_STOP=on <<'SQL'
-- Create a 30-day test round (only if none active)
INSERT INTO rounds (id, started_at, ends_at, is_active)
SELECT
    'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa'::uuid,
    NOW(),
    NOW() + INTERVAL '30 days',
    true
WHERE NOT EXISTS (SELECT 1 FROM rounds WHERE is_active = true);
SQL

ROUND_ID=$(psql "$DATABASE_URL" -t -A -c "SELECT id FROM rounds WHERE is_active = true LIMIT 1")
echo "==> Active round: $ROUND_ID"
echo "==> Seed complete. You can now register users and claim parcels."
