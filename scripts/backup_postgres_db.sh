#!/bin/bash

# Set variables
CONTAINER_NAME="bspl-db-1"
DB_NAME="${POSTGRES_DB}"
DB_USER="${POSTGRES_USER}"
DB_PASSWORD="${POSTGRES_PASSWORD}"
BACKUP_DIR="db/backup"
DATE=$(date +"%Y%m%d_%H%M%S")
BACKUP_FILE="${BACKUP_DIR}/backup_${DATE}.sql"

# Create backup
PGPASSWORD="${DB_PASSWORD}" docker exec -e PGPASSWORD -t ${CONTAINER_NAME} pg_dump -U "${DB_USER}" "${DB_NAME}" >"${BACKUP_FILE}"

echo "Backup completed: ${BACKUP_FILE}"
