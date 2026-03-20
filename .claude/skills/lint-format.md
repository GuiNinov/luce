# Lint and Format Skill

## Description
Runs Clippy linting and rustfmt formatting across the entire Luce workspace

## Usage
Use this skill to ensure code quality, catch potential issues, and maintain consistent formatting across all packages.

## Commands
```bash
# Run Clippy linter for all packages
cargo clippy --all-targets --all-features -- -D warnings

# Format all code using rustfmt
cargo fmt --all

# Check formatting without making changes
cargo fmt --all -- --check

# Run both linting and formatting in sequence
cargo fmt --all && cargo clippy --all-targets --all-features -- -D warnings
```

## Expected Output
- Clippy warnings and errors with file locations
- Formatting changes applied (or check results)
- Success/failure status for each operation
- Detailed error messages for any issues found

## Usage Examples
```bash
# Quick lint and format
cargo fmt --all && cargo clippy --all-targets --all-features

# Check only (no changes)
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings

# Lint specific package
cargo clippy -p luce-shared --all-features -- -D warnings
```

## Notes
- Uses `-D warnings` to treat warnings as errors for strict quality
- Formats all packages in the workspace simultaneously
- `--all-targets` includes tests, examples, and benchmarks
- `--all-features` ensures all conditional code is checked
- Essential before committing code or creating PRs