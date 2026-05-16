# Development Setup Guide

## Prerequisites
- Rust (install via rustup from https://rustup.rs/)
- Python 3.8+
- pip

## Initial Setup

1. Clone the repository
2. Create a virtual environment:
   ```bash
   python -m venv .venv
   .venv\Scripts\Activate  # On Windows
   # or
   source .venv/bin/activate  # On macOS/Linux
   ```

3. Install development dependencies:
   ```bash
   pip install maturin pytest
   ```

4. Enable the repo-local git hooks for this clone:
   ```bash
   bash setup-hooks.sh
   ```

5. Build the extension:
   ```bash
   maturin develop
   ```

## Repository Layout

- `crates/core/` holds the shared search engine
- `pyfastgrep/` holds the Python binding layer
- `cli/` holds the thin command line interface
- `.githooks/pre-push` holds the tracked repo-local pre-push hook

## Running Tests

```bash
python tests/mandatory_tests.py
```

## Development Workflow

1. Make your changes to the Rust code or Python API
2. Run `maturin develop` to rebuild the extension
3. Test your changes: `python tests/mandatory_tests.py`
4. Commit and push once the local checks are green

## Git Hooks

After you run `bash setup-hooks.sh`, this clone uses the tracked `.githooks/pre-push` hook.

The pre-push hook will:
- Run `cargo check --all-targets`
- Rebuild the extension with `maturin develop`
- Run `python tests/mandatory_tests.py`

If any step fails, the push is aborted.
