# pyfastgrep

Fast file search for Python powered by ripgrep's engine.

## Install

pip install pyfastgrep

## Usage

```python
import pyfastgrep

results = pyfastgrep.search(r'"/[^"]*-[^"]*"', "src")

for r in results:
    print(r)
```

## Features
1. Uses ripgrep internals (fast)
2. Parallel search
3. Respects .gitignore
4. Python-friendly API