---
name: pullrequest
description: Create GitHub pull requests with a consistent workflow. Use when asked to open/create a PR on GitHub, prepare a PR description, or summarize branch commits against main/master before creating a PR with gh CLI.
---

# Pull Request Skill

Create a PR with `gh` using a deterministic flow.

## Workflow

1. Select base branch.
   - Prefer `master` when it exists on `origin`.
   - Otherwise use `main`.
   - Resolve with:
     - `git show-ref --verify --quiet refs/remotes/origin/master && echo master || echo main`

1. Fetch remote state.
   - Run: `git fetch origin <base>`

3. List commits that will be in the PR.
   - Run: `git log --reverse --pretty=format:%s origin/<base>..HEAD`
   - Use commit titles only (no hashes).
   - Optional helper script: `./.agents/skills/pullrequest/scripts/commits_for_pr.sh <base>`

4. Build PR body.
   - Include only:
     - `## Summary` with 1-3 concise bullets about intent.
     - `## Commits` with bullet list of commit titles from step 3.
   - Do not include hashes, test logs, or extra sections.

5. Open PR with GitHub CLI.
   - Ensure the branch is pushed (if needed): `git push -u origin HEAD`
   - Create PR with heredoc body:

```bash
gh pr create --base "<base>" --title "<title>" --body "$(cat <<'EOF'
## Summary
- <bullet>

## Commits
- <commit title 1>
- <commit title 2>
EOF
)"
```

6. Return the PR URL.
   - Print the URL from `gh pr create` output.

## Constraints

- Use `gh` for PR creation.
- Keep description minimal: summary + commits only.
- Keep commit list order from oldest to newest.
