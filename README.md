# pyfastgrep

Fast file search for Python powered by ripgrep's engine.

`pyfastgrep` is now organized as a small workspace:
- `crates/core/` contains the shared Rust search engine
- `pyfastgrep/` contains the Python bindings
- `cli/` contains the thin CLI binary

## Install

pip install pyfastgrep

## Usage

### Python API

```python
import pyfastgrep

results = pyfastgrep.search(r'"/[^"]*-[^"]*"', "src")

for r in results:
    print(r)
```

Ergonomic keyword aliases are also supported:

```python
results = pyfastgrep.search("FN", root="src", glob="*.rs", case_insensitive=True, limit=10)
json_results = pyfastgrep.search("FN", root="src", glob="*.rs", case_insensitive=True, as_json=True)
```

### CLI

The CLI is a thin interface over the same Rust core:

```bash
pyfastgrep fn src --glob "*.rs" --ignore-case
pyfastgrep fn src --glob "*.rs" --ignore-case --json
```

You can also run it directly from the workspace while developing:

```bash
cargo run -p pyfastgrep-cli -- fn src --glob "*.rs" --ignore-case
```

## Features
1. Uses ripgrep internals (fast)
2. Parallel search
3. Respects .gitignore
4. Python-friendly API
5. Thin CLI over the same Rust core