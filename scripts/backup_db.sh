#!/bin/bash
set -x           # Enable debug mode
pwd              # Print current working directory
ls -l "$SRC_DIR" # List contents of source directory

# Set the source and destination directories
SRC_DIR="db/libsql"
BACKUP_DIR="db/libsql/backup"

# Set the source filename
SRC_FILE="local.db"

# Timestamp format
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# Set the destination filename with timestamp
DEST_FILE="local-${TIMESTAMP}.db"

# Create the backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Perform the copy
cp "${SRC_DIR}/${SRC_FILE}" "${BACKUP_DIR}/${DEST_FILE}"

# Check if the copy was successful
if [ $? -eq 0 ]; then
  echo "Backup created successfully: ${BACKUP_DIR}/${DEST_FILE}"
else
  echo "Error: Backup creation failed"
  exit 1
fi
