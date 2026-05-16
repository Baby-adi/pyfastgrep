#!/bin/bash
# setup-hooks.sh - Install git hooks for development

HOOKS_DIR=".git/hooks"
SCRIPTS_DIR="scripts"

echo "Setting up git hooks for pyfastgrep development..."

# Copy pre-push hook
if [ -f "$SCRIPTS_DIR/pre-push" ]; then
    cp "$SCRIPTS_DIR/pre-push" "$HOOKS_DIR/pre-push"
    chmod +x "$HOOKS_DIR/pre-push"
    echo "pre-push hook installed"
else
    echo "Error: $SCRIPTS_DIR/pre-push not found"
    exit 1
fi

echo "All git hooks have been installed!"
