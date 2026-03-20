# Test Skill

## Description
Runs comprehensive tests across the entire Luce workspace

## Usage
Use this skill to run all unit tests, integration tests, and validate code quality across all packages in the workspace.

## Commands
```bash
cargo test
```

## Expected Output
- Test results for all workspace packages (shared, core, cli, api, mcp, ui)
- Pass/fail status for each test
- Coverage information if available
- Any test failures with detailed error messages

## Notes
- Runs tests for all packages in parallel by default
- Use `cargo test -p <package-name>` for specific package testing
- Add `-- --nocapture` for verbose test output if needed