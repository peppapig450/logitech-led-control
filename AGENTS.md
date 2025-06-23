# Repository Guidelines for Codex

## Coding Conventions
- Write idiomatic Rust targeting the edition specified in `Cargo.toml`.
- Use `cargo fmt` for formatting. You may check formatting with `cargo fmt --all -- --check` and fix with `cargo fmt --all`.
- Run `cargo clippy --all-targets -- -D warnings` and resolve all warnings before committing.
- Keep the code warnings free and prefer expressive types, clear error handling and small focused functions.

## Programmatic Checks
Run the following commands after every change:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The repository does not contain tests yet, but `cargo test` must still be invoked to ensure the project builds.

## Commit Message Style
- Use [Conventional Commits](https://www.conventionalcommits.org) for commit messages.
- Provide an extended commit body explaining what and why, wrapping lines at 72 characters when possible.
- Include footers as needed (e.g. `Signed-off-by`, `Co-authored-by`).

## Pull Requests
Include a summary of changes and cite relevant files and test outputs in the PR description. Use the instructions in this file and the top-level AGENTS guidance when generating PR messages.


