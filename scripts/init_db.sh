set -x
set -eo pipefail

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Please install sqlx by running:"
  echo >&2 "cargo install sqlx-cli --no-default-features --features rustls,postgres"
  exit 1
fi

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

if [[ -z "${SKIP_DOCKER}" ]]; then
  DB_CONTAINER_ID=$(docker run \
    --env POSTGRES_USER=${DB_USER} \
    --env POSTGRES_PASSWORD=${DB_PASSWORD} \
    --env POSTGRES_DB=${DB_NAME} \
    --publish "${DB_PORT}":5432 \
    --detach postgres \
    postgres -N 1000)

  export PGPASSWORD="${DB_PASSWORD}"
  until docker exec -it "${DB_CONTAINER_ID}" psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "${DB_NAME}" -c '\q'; do
    echo >&2 "Postgres is still unavailable - sleeping"
    sleep 1
  done
fi

DATABASE_URL=postgresql://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL
sqlx database create
sqlx migrate run

echo >&2 "Migration finished - database is up"
