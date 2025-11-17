#!/bin/sh
set -e

# Pastikan ada DB env (tapi tidak echo nilainya)
: "${DATABASE_URL:?DATABASE_URL must be set}"

: "${DB_HOST:=db}"
: "${DB_PORT:=5432}"
: "${DB_USER:=postgres}"

echo "Waiting for Postgres at ${DB_HOST}:${DB_PORT}..."
until pg_isready -h "${DB_HOST}" -p "${DB_PORT}" -U "${DB_USER}" >/dev/null 2>&1; do
  echo "Postgres not ready yet..."
  sleep 1
done
echo "Postgres is ready."

# Run loco migrate (silent env)
#if command -v loco >/dev/null 2>&1; then
#  echo "Running database migrations..."
#  if ! loco db migrate; then
#    echo "⚠️ loco migrate failed (continuing anyway)"
#  fi
#  if ! loco db entities; then
#    echo "⚠️ loco entities failed (continuing anyway)"
#  fi
#else
#  echo "loco CLI not found — skipping migrations."
#fi

# Start the app
if [ -x "./seco" ]; then
  echo "[entrypoint] Found release binary ./seco — exec it"
  exec ./seco
else
  if command -v cargo >/dev/null 2>&1 && command -v loco >/dev/null 2>&1; then
    echo "[entrypoint] Binary not found — using `cargo loco start --server-and-worker -b 0.0.0.0`"
    exec cargo loco start --server-and-worker -b 0.0.0.0
  elif command -v cargo >/dev/null 2>&1; then
    echo "[entrypoint] loco not found but cargo exists — exec cargo run"
    exec cargo run
  else
    echo "[entrypoint] ERROR: neither ./seco, nor cargo found. Cannot start."
    exit 1
  fi
fi
