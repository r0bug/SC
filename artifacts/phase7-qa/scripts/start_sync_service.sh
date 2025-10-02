#!/bin/bash
export DATABASE_URL="${DATABASE_URL:-sqlite:./data/contacts.db}"
export PORT="${PORT:-3002}"
export JWT_SECRET="${JWT_SECRET:-CHANGE_ME_IN_PRODUCTION}"
export LOG_FORMAT="${LOG_FORMAT:-json}"
export ATTACHMENT_STORAGE_PATH="${ATTACHMENT_STORAGE_PATH:-./data/attachments}"

mkdir -p data/attachments
exec ../binaries/sync_service
