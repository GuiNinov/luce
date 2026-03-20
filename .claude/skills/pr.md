# Pull Request Skill

## Description
Creates a pull request with proper formatting and follows project conventions

## Usage
Use this skill to create a well-formatted pull request for the Luce project with appropriate title, description, and test plan.

## Commands
```bash
# Check current branch and status
git status
git diff main...HEAD

# Push current branch to remote if needed
git push -u origin <branch-name>

# Create PR using GitHub CLI
gh pr create --title "<descriptive-title>" --body "$(cat <<'EOF'
## Summary
- Brief description of changes
- Key features or fixes implemented
- Impact on system architecture

## Test Plan
- [ ] All existing tests pass (`cargo test`)
- [ ] New tests added for new functionality
- [ ] Manual testing performed
- [ ] Integration testing completed

## Changes
- List specific files/modules modified
- Highlight any breaking changes
- Note any new dependencies

🤖 Generated with [Claude Code](https://claude.ai/code)
EOF
)"
```

## Expected Output
- Pull request URL
- PR number and title
- Link to view the PR in GitHub

## Notes
- Ensures current branch is up to date before creating PR
- Uses main branch as the base branch (per CLAUDE.md)
- Includes comprehensive test plan checklist
- Follows project's commit message and PR conventions