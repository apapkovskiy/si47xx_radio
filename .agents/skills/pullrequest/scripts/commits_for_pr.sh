#!/usr/bin/env bash
set -euo pipefail

# Prints commit titles (oldest -> newest) for PR body.
# Usage:
#   commits_for_pr.sh [base-branch]
# Examples:
#   commits_for_pr.sh master
#   commits_for_pr.sh main

BASE="${1:-master}"

git log --reverse --pretty=format:'- %s' "origin/${BASE}..HEAD"
