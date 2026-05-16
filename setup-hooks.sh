#!/bin/sh
set -eu

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

HOOKS_PATH=".githooks"
PRE_PUSH_HOOK="$HOOKS_PATH/pre-push"

if [ ! -f "$PRE_PUSH_HOOK" ]; then
    echo "Error: missing $PRE_PUSH_HOOK"
    exit 1
fi

chmod +x "$PRE_PUSH_HOOK"
git config --local core.hooksPath "$HOOKS_PATH"

echo "Configured repo-local hooks path: $(git config --local --get core.hooksPath)"
echo "Git will now run $PRE_PUSH_HOOK on push for this clone."
