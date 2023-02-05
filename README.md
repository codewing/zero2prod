# Zero To Production

## Setup

Install the following tools

- `docker` (via os package manager)
- `sqlx-cli` (via cargo)
  - Make sure `sqlx` is available in the path.
- `psql` (installed via postgres)

IDE Settings:

- Use `rustfmt` to format the code
- Use `clippy` to check for code style errors

## Execution

1. Make sure docker is running
2. Run one of the _scripts/init_db_ scripts depending on the platform
3. Run the tests with `cargo test` or the server with `cargo run`
