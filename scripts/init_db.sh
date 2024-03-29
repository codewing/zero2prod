#!/usr/bin/env bash
set -x
set -eo pipefail

if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 " cargo install --version='~0.6' sqlx-cli \
  --no-default-features --features rustls,postgres"
  echo >&2 "to install it."
  exit 1
fi

if ! [ -x "$(command -v docker)" ]; then
  echo >&2 "Error: docker is not installed."
  exit 1
fi

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=my_very_secure_database_password_1}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
DB_HOST="${POSTGRES_HOST:=localhost}"

DOCKER_POSTGRES_NAME=${DOCKER_POSTGRES_CONTAINER:=zero2prod_db}

DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@${DB_HOST}:${DB_PORT}/${DB_NAME}
export DATABASE_URL=${DATABASE_URL}

# Launch postgres using Docker
if [[ -z "${SKIP_DOCKER}" ]]
then
  if [ ! "$(docker ps -q -f name=${DOCKER_POSTGRES_NAME})" ]; then
    if [ "$(docker ps -aq -f status=exited -f name=${DOCKER_POSTGRES_NAME})" ]; then
        # cleanup
        docker rm ${DOCKER_POSTGRES_NAME}
    fi
    
    docker run \
    --name=${DOCKER_POSTGRES_NAME} \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    -d postgres \
    postgres -N 1000
    # ^ Increased maximum number of connections for testing purposes
  fi
fi

# Keep pinging Postgres until it's ready to accept commands
until PGPASSWORD="${DB_PASSWORD}" psql -h "${DB_HOST}" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
 echo "Postgres is still unavailable - sleeping"
 sleep 1
done
echo "Postgres is up and running on port ${DB_PORT}!"

sqlx database create
sqlx migrate run
