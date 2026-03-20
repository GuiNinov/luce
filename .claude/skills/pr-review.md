# PR Review Reader Skill

## Description
Reads and analyzes pull request reviews, comments, and feedback

## Usage
Use this skill to fetch and analyze PR review comments, suggestions, and approval status for better understanding of feedback.

## Commands
```bash
# Get PR details and review status
gh pr view <pr-number>

# Get all review comments
gh api repos/:owner/:repo/pulls/<pr-number>/comments

# Get review summaries (approve/request changes/comment)
gh api repos/:owner/:repo/pulls/<pr-number>/reviews

# Get specific files changed in the PR
gh pr diff <pr-number>

# Check PR status and checks
gh pr checks <pr-number>
```

## Expected Output
- PR title, description, and current status
- List of reviewers and their approval status
- Individual review comments with file context
- Suggested changes or requested modifications
- CI/CD check status

## Usage Examples
```bash
# Read reviews for PR #123
gh pr view 123
gh api repos/GuiNinov/scarlet-coaster/pulls/123/reviews

# Check comments on specific files
gh api repos/GuiNinov/scarlet-coaster/pulls/123/comments

# View diff for context
gh pr diff 123
```

## Notes
- Combines PR metadata with detailed review feedback
- Shows both inline comments and general review comments
- Helps understand reviewer concerns and suggestions
- Useful for addressing feedback systematically