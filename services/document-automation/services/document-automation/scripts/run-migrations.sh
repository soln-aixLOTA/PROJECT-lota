#!/bin/bash
set -e
psql "$DATABASE_URL" -f migrations/20240105_init.sql
