# Tests are run by nextest

set dotenv-required
set dotenv-filename := ".env.dev"

watch_base := "cargo watch -q -c -i 'tests/resources/**/*'"
no_capture := if env_var("TEST_LOG") == "true" { "--no-capture" } else { "" }

run bin="":
    clear
    cargo run --bin {{bin}}

# Watch

watch:
     {{watch_base}} -x "c --workspace --all-features --all-targets"

watch-test name="":
    {{watch_base}} -s "just test {{name}}"

watch-test-pkg pkg:
    {{watch_base}} -s "just test-pkg {{pkg}}"

watch-example package name:
    {{watch_base}} -s "just example {{package}} {{name}}"

watch-test-integration:
    {{watch_base}} -x "nextest run --workspace --all-features -E 'kind(test)'"

watch-bench name="":
    {{watch_base}} -s "just bench {{name}}"


# Individual commands

test name="":
    clear
    cargo nextest run {{no_capture}} --workspace --all-features --all-targets {{name}}

test-pkg pkg:
    clear
    cargo nextest run --all-features --all-targets --package {{pkg}}

test-doc:
    clear
    cargo test --workspace --all-features --doc

check-lib-bins:
    clear
    cargo check --workspace --all-features --lib --bins

example package name:
    clear
    cargo run -p {{package}} --example {{name}}

bench name="":
    clear
    cargo bench --workspace --all-features --all-targets {{name}}

cov:
    clear
    rustup run nightly cargo llvm-cov nextest --open --workspace --all-features --lib --locked

lint:
    clear
    cargo clippy --workspace --all-features --all-targets --locked

tree:
    clear
    cargo tree --workspace --all-features --all-targets -i derive_more-impl

## DB
turso-dev:
    turso dev --db-file db/local.db

geni-up-local:
    geni up

geni-down-local:
    geni down

backup-db:
    scripts/backup_db.sh

reset-db:
    just backup-db
    just geni-down-local
    just geni-up-local

restore-db:
    sqlite3 db/local.db < db/restore.sql

## Others
git-gc:
    git gc --prune=now --aggressive