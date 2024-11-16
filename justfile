# Tests are run by nextest
# --all-features is not passed since `minify-html` is a optional dependency which takes long to compile,
# and is not going to be included in local development.
# CI will run with the `--all-features` flag.

set dotenv-required := true
set dotenv-filename := ".env"

watch_base := "cargo watch -q -c -i 'tests/resources/**/*'"
no_capture := if env_var("TEST_LOG") == "true" { "--no-capture" } else { "" }

run bin="":
    clear
    cargo run --bin {{ bin }} -r

# Watch

watch:
    {{ watch_base }} -x "c --all-targets"

watch-test name="":
    {{ watch_base }} -s "just test {{ name }}"

watch-test-pkg pkg:
    {{ watch_base }} -s "just test-pkg {{ pkg }}"

watch-example package name:
    {{ watch_base }} -s "just example {{ package }} {{ name }}"

watch-test-integration:
    {{ watch_base }} -x "nextest run -E 'kind(test)'"

watch-bench name="":
    {{ watch_base }} -s "just bench {{ name }}"

# Test commands

test name="":
    clear
    cargo nextest run {{ no_capture }} --all-targets {{ name }}

test-pkg pkg:
    clear
    cargo nextest run --all-targets --package {{ pkg }}

test-doc:
    clear
    cargo test --doc

check-lib-bins:
    clear
    cargo check --lib --bins

cov:
    clear
    rustup run nightly cargo llvm-cov nextest --open --lib --locked

# Other cargo commands

example package name:
    clear
    cargo run -p {{ package }} --example {{ name }}

bench package name="":
    clear
    cargo bench --all-features --all-targets -p {{ package }} {{ name }}

lint:
    clear
    cargo clippy --all-targets --locked

tree crate:
    clear
    cargo tree --all-features --all-targets -i {{ crate }}

doc package="":
    clear
    cargo doc --all-features --no-deps -p {{ package }} --open

# DB
backup-db:
    scripts/backup_postgres_db.sh

dm-list:
    diesel migration list

dm-run:
    diesel migration run

dm-revert:
    diesel migration revert

dm-redo:
    diesel migration redo

# Postgres
compose-up:
    docker compose up -d

compose-config:
    echo $POSTGRES_PASSWORD
    docker compose config

# Others
git-gc:
    git gc --prune=now --aggressive

# SeaORM
# The migration itself runs on the public schema,
# but the actual data should be stored in the appropraite schema, other than `public`.
# They should be explicitly defined in the SQL script,
# rather than relying on the search_path.
#
# SeaORM supports running scripts on specific schemas(using the `-s` flag),
# but this isn't appropriate since the initial script for creating schemas won't work when there is no schema to run on.
# (e.g. `CREATE SCHEMA dart;` won't work if the schema `dart` doesn't exist)
#
# TODO
# There's a pull request for specifying the migration directory with a env variable,
# which will remove the need for using the `-d` flag every time.
# <https://github.com/SeaQL/sea-orm/pull/2419>

sea:
    sea-orm-cli migrate status -d crates/migration

sea-status:
    sea-orm-cli migrate status -d crates/migration

sea-up:
    sea-orm-cli migrate up -d crates/migration

sea-down:
    sea-orm-cli migrate down -d crates/migration

sea-generate:
    just sea-generate-smes
    just sea-generate-dart

sea-generate-smes:
    sea-orm-cli generate entity -o crates/db/src/entities/smes -s smes

sea-generate-dart:
    sea-orm-cli generate entity -o crates/db/src/entities/dart -s dart
