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
csv_output = pyfastgrep.search("FN", root="src", glob="*.rs", case_insensitive=True, as_csv=True)
pyfastgrep.search("FN", root="src", glob="*.rs", case_insensitive=True, as_csv=True, output_path="results.csv")
```

Usage by mode:

```python
# Plain tuples
pyfastgrep.search("fn", "src", "*.rs")

# JSON objects
pyfastgrep.search("fn", "src", "*.rs", as_json=True)

# CSV text
pyfastgrep.search("fn", "src", "*.rs", as_csv=True)

# CSV written to a file
pyfastgrep.search("fn", "src", "*.rs", as_csv=True, output_path="results.csv")

# Streaming iterator
for match in pyfastgrep.search_iter("fn", "src", "*.rs"):
    print(match)

# Count matches per file (returns [(file, count)])
pyfastgrep.count("fn", "src", "*.rs")

# Filenames with at least one match
pyfastgrep.files_with_matches("fn", "src", "*.rs")

# Search with context lines
pyfastgrep.search_with_context("fn", "src", "*.rs", before_context=2, after_context=2)
# Returns [(file, line, content, [before_lines], [after_lines])]
```

### CLI

The CLI is a thin interface over the same Rust core:

```bash
pyfastgrep fn src --glob "*.rs" --ignore-case
pyfastgrep fn src --glob "*.rs" --ignore-case --json
pyfastgrep fn src --glob "*.rs" --ignore-case --csv
pyfastgrep fn src --glob "*.rs" --ignore-case --csv --output results.csv
```

**New in this branch:** `--count`, `--files-with-matches`, `--fixed-strings`, and `--context` flags.

```bash
# Count matches per file
pyfastgrep fn src --glob "*.rs" --ignore-case --count
pyfastgrep fn src --glob "*.rs" --ignore-case --count --json

# Only show filenames with matches
pyfastgrep fn src --glob "*.rs" --ignore-case --files-with-matches

# Fixed-strings: treat pattern as literal text
pyfastgrep "." src --glob "*.rs" --fixed-strings
pyfastgrep "fn" src --glob "*.rs" --fixed-strings --count

# Context lines (like ripgrep -C/-A/-B)
pyfastgrep "fn" src --glob "*.rs" --context 2
pyfastgrep "fn" src --glob "*.rs" --after-context 3 --before-context 1
pyfastgrep "fn" src --glob "*.rs" --context 2 --json
```

You can also run it directly from the workspace while developing:

```bash
cargo run -p pyfastgrep-cli -- fn src --glob "*.rs" --ignore-case
```

CLI output modes:

```bash
cargo run -p pyfastgrep-cli -- fn src --glob "*.rs" --ignore-case --json
cargo run -p pyfastgrep-cli -- fn src --glob "*.rs" --ignore-case --csv
cargo run -p pyfastgrep-cli -- fn src --glob "*.rs" --ignore-case --csv --output results.csv
cargo run -p pyfastgrep-cli -- fn src --glob "*.rs" --ignore-case --count
cargo run -p pyfastgrep-cli -- fn src --glob "*.rs" --ignore-case --files-with-matches
cargo run -p pyfastgrep-cli -- fn src --glob "*.rs" --ignore-case --context 2
cargo run -p pyfastgrep-cli -- fn src --glob "*.rs" --ignore-case --context 2 --json
```

CLI flags at a glance:

```bash
pyfastgrep <pattern> [root] [--glob <pattern>] [--limit <n>] [--ignore-case] [--fixed-strings] [--json] [--csv] [--output <file>] [--root <path>] [--count] [--files-with-matches] [--context <n>] [--before-context <n>] [--after-context <n>]
```

> **Breaking change in this branch:** `-c` is now shorthand for `--count` (aligning with ripgrep). Use `--csv` for CSV output (the `-c` shortcut for `--csv` has been removed).

### AST-powered semantic search

Search by structure, not just text:

```python
# Find functions by name
pyfastgrep.search_functions("main", "src", "*.py")

# Find classes/structs by name
pyfastgrep.search_classes("MyClass", "src", "*.py")

# Find imports/use statements
pyfastgrep.search_imports("requests", "src", "*.py")

# Streaming AST search
for match in pyfastgrep.search_functions_iter("main", "src", "*.py"):
    print(match)
```

Supported languages: Rust, Python, C, C++, Go, JavaScript, TypeScript.

### CLI AST search

```bash
pyfastgrep build_config src --glob "*.rs" --functions
pyfastgrep PyResultIterator src --glob "*.rs" --classes
pyfastgrep pyo3 src --glob "*.rs" --imports
```

## Features
1. Uses ripgrep internals (fast regex search)
2. Parallel search
3. Respects .gitignore
4. Python-friendly API with ergonomic aliases
5. Thin CLI over the same Rust core
6. AST-powered semantic search (functions, classes, imports)
7. Streaming iterators for both regex and AST search
8. JSON, CSV, and tuple output modes
9. Context lines (-A, -B, -C) with ripgrep-compatible output
10. Fixed-strings mode for literal search