#! /bin/bash

sea-orm-cli generate entity -o src/db/entities --compact-format --with-copy-enums --database-url "sqlite://mng.db" --date-time-crate time --with-serde both
