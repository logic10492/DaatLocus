#!/bin/sh
set -eu

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

chmod +x scripts/git-hooks/pre-commit
git config core.hooksPath scripts/git-hooks

printf 'Configured git core.hooksPath=scripts/git-hooks\n'
printf 'Installed pre-commit hook: scripts/git-hooks/pre-commit\n'
