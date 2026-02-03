#!/bin/sh

sea-orm-cli migrate generate -d src/db/migrations $1
