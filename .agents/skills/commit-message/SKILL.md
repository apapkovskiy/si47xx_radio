---
name: commit-message
description: Draft a semantic commit message for the current repository changes, using the branch name to extract a JIRA key when present and returning message text without creating a commit.
---

# Commit Message

Use this skill when the goal is to prepare commit message text for the current changes without running `git commit`.

## Workflow

1. Inspect the current branch name.
2. Inspect only staged changes.
3. Determine the change intent from the diff.

## Commit type selection

Pick one type based on the change:

- `Feat`: new user-facing feature
- `Fix`: user-facing bug fix
- `Docs`: documentation only
- `Style`: formatting only, no behavior change
- `Refactor`: code restructuring without intended behavior change
- `Test`: test-only changes
- `Infra`: CI or infrastructure changes
- `Chore`: tooling or maintenance work without production behavior change

## Format rules

- Summary format: `<Type>: <imperative summary>`
- Prefer a summary length of 50 characters or less, with a hard cap of 77.
- Use imperative mood.
- Add a blank line after the summary.
- In the body, explain what changed and why it matters.

## Output template

```text
<Type>: Use imperative style summary

Explain what and why this commit was done.
```

Return only the final commit message text.
