# Run checks for both server and web
check:
	cargo check --features server
	cargo check --features web

# Serve website
serve:
	dx serve --web

# Generate sea-orm db entities
generate-entities:
	sea-orm-cli generate entity -o src/db/entities --compact-format --with-copy-enums --database-url "sqlite://mng.db" --date-time-crate time --with-serde both

# Generate a sea-orm migration
generate-migration name:
	sea-orm-cli migrate generate -d src/db/migrations {{name}}
