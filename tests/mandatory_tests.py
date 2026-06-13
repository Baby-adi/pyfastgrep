import os
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]

sys.path.insert(0, str(REPO_ROOT))

import pyfastgrep  # noqa: E402


def run_test(name, func):
    try:
        func()
        print(f"PASS: {name}")
        return True
    except AssertionError as exc:
        print(f"FAIL: {name} - {exc}")
        return False


def main():
    print("Running pyfastgrep test suite...")
    source_root = str(REPO_ROOT / "src")

    def test_case_sensitive_search():
        res_sensitive = pyfastgrep.search("FN", source_root, "*.rs", None, False, False)
        assert len(res_sensitive) == 0, f"Expected 0 results for case-sensitive 'FN', got {len(res_sensitive)}"

    def test_ignore_case_search():
        res_ignore = pyfastgrep.search("FN", source_root, "*.rs", None, True, False)
        assert len(res_ignore) > 0, "Expected >0 results for 'FN' with ignore_case=True"

    def test_iterator_matches_batch():
        res_ignore = pyfastgrep.search("FN", source_root, "*.rs", None, True, False)
        iter_ignore = list(pyfastgrep.search_iter("FN", source_root, "*.rs", True, False))
        assert len(iter_ignore) == len(res_ignore), "Batch and iterator search result counts should match"

    def test_json_output():
        json_results = pyfastgrep.search("fn", source_root, "*.rs", None, False, True)
        json_iter = list(pyfastgrep.search_iter("fn", source_root, "*.rs", False, True))

        assert len(json_results) > 0, "Expected >0 results for JSON batch search"
        assert len(json_iter) > 0, "Expected >0 results for JSON iterator search"
        assert isinstance(json_results[0], dict), "JSON batch results should contain dicts"
        assert isinstance(json_iter[0], dict), "JSON iterator results should contain dicts"
        assert {'file', 'line', 'content'} <= set(json_results[0].keys()), "JSON results should have file, line, and content keys"

    def test_csv_output():
        csv_path = Path(tempfile.gettempdir()) / "pyfastgrep_api_output.csv"
        csv_iter_path = Path(tempfile.gettempdir()) / "pyfastgrep_api_iter_output.csv"

        for candidate in (csv_path, csv_iter_path):
            if candidate.exists():
                candidate.unlink()

        csv_results = pyfastgrep.search("fn", source_root, "*.rs", None, False, False, as_csv=True, output_path=str(csv_path))
        csv_iter = list(pyfastgrep.search_iter("fn", source_root, "*.rs", False, False, csv=True, output_path=str(csv_iter_path)))

        assert isinstance(csv_results, str), "CSV batch results should be a string"
        assert csv_results.startswith("file,line,content"), "CSV batch results should start with a header"
        assert len(csv_iter) > 1, "CSV iterator should include a header and at least one row"
        assert csv_iter[0] == "file,line,content\n", "CSV iterator should yield the header first"
        assert csv_iter[1].endswith("\n"), "CSV iterator rows should end with a newline"
        assert csv_path.exists(), "CSV batch should write a file"
        assert csv_iter_path.exists(), "CSV iterator should write a file"
        assert csv_path.read_text(encoding="utf-8").startswith("file,line,content"), "CSV batch file should start with a header"
        assert csv_iter_path.read_text(encoding="utf-8").startswith("file,line,content"), "CSV iterator file should start with a header"

    def test_cli_csv():
        csv_path = Path(tempfile.gettempdir()) / "pyfastgrep_cli_output.csv"

        if csv_path.exists():
            csv_path.unlink()

        cli_result = subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "pyfastgrep-cli",
                "--",
                "fn",
                "src",
                "--glob",
                "*.rs",
                "--ignore-case",
                "--csv",
                "--output",
                str(csv_path),
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
        )

        assert cli_result.returncode == 0, f"CLI CSV exited with {cli_result.returncode}: {cli_result.stderr}"
        assert cli_result.stdout.startswith("file,line,content"), "CLI CSV output should start with a header"
        assert csv_path.exists(), "CLI CSV should write a file"
        assert csv_path.read_text(encoding="utf-8").startswith("file,line,content"), "CLI CSV file should start with a header"

    def test_legacy_output_and_consistency():
        json_results = pyfastgrep.search("fn", source_root, "*.rs", None, False, True)
        legacy_results = pyfastgrep.search("fn", source_root, "*.rs", None, False, False)

        assert len(legacy_results) > 0, "Expected >0 results for legacy search"
        assert isinstance(legacy_results[0], tuple), "Legacy results should contain tuples"
        assert len(legacy_results[0]) == 3, "Legacy tuples should have 3 elements"
        assert json_results[0]['file'] == legacy_results[0][0], "File paths should match between JSON and legacy"
        assert json_results[0]['line'] == legacy_results[0][1], "Line numbers should match between JSON and legacy"
        assert json_results[0]['content'].strip() == legacy_results[0][2].strip(), "Content should match between JSON and legacy"

    def test_cli_smoke():
        cli_result = subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "pyfastgrep-cli",
                "--",
                "fn",
                "src",
                "--glob",
                "*.rs",
                "--ignore-case",
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
        )

        assert cli_result.returncode == 0, f"CLI exited with {cli_result.returncode}: {cli_result.stderr}"
        assert os.path.join("src", "lib.rs") in cli_result.stdout, "CLI output should include the Rust source file"

    def test_ergonomic_aliases():
        alias_results = pyfastgrep.search("FN", root=source_root, glob="*.rs", case_insensitive=True, limit=2)
        alias_iter = list(pyfastgrep.search_iter("FN", root=source_root, glob="*.rs", case_insensitive=True))

        assert len(alias_results) > 0, "Alias-based search should find results"
        assert len(alias_iter) > 0, "Alias-based iterator search should find results"
        assert len(alias_results) <= 2, "limit alias should cap the batch results"

    def test_ast_functions():
        results = pyfastgrep.search_functions("build_config", source_root, "*.rs")
        assert len(results) > 0, "AST function search should find build_config"
        assert any("src/" in r[0].replace(os.sep, "/") for r in results), "Should be in a src/ file"

    def test_ast_classes():
        results = pyfastgrep.search_classes("PyResultIterator", source_root, "*.rs")
        assert len(results) > 0, "AST class search should find PyResultIterator"

    def test_ast_imports():
        results = pyfastgrep.search_imports("pyo3", source_root, "*.rs")
        assert len(results) > 0, "AST import search should find pyo3 imports"

    def test_ast_iterator_matches_batch():
        batch = pyfastgrep.search_functions("build_config", source_root, "*.rs")
        streamed = list(pyfastgrep.search_functions_iter("build_config", source_root, "*.rs"))
        assert len(streamed) == len(batch), "AST batch and iterator counts should match"

    def test_ast_glob_filter():
        with_glob = pyfastgrep.search_functions("build_config", source_root, "*.rs")
        without_glob = pyfastgrep.search_functions("build_config", source_root)
        assert len(with_glob) > 0, "With glob should find results"
        assert len(without_glob) >= len(with_glob), "Without glob should find equal or more"

    def test_python_count():
        results = pyfastgrep.count("fn", source_root, "*.rs")
        assert len(results) > 0, "count should find matches in at least one file"
        assert all(isinstance(r, tuple) and len(r) == 2 for r in results), "Each result should be a (file, count) tuple"
        assert all(isinstance(r[0], str) and isinstance(r[1], int) and r[1] > 0 for r in results), "Counts should be positive integers"

    def test_python_files_with_matches():
        results = pyfastgrep.files_with_matches("fn", source_root, "*.rs")
        assert len(results) > 0, "files_with_matches should find matches"
        assert all(isinstance(r, str) for r in results), "Each result should be a filename string"
        assert len(set(results)) == len(results), "Results should not contain duplicates"

    def test_count_respects_ignore_case():
        sensitive = pyfastgrep.count("FN", source_root, "*.rs")
        insensitive = pyfastgrep.count("FN", source_root, "*.rs", ignore_case=True)
        assert sum(c for _, c in sensitive) < sum(c for _, c in insensitive), "Ignore case should find more or equal matches"

    def test_cli_count():
        cli_result = subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "pyfastgrep-cli",
                "--",
                "fn",
                "src",
                "--glob",
                "*.rs",
                "--ignore-case",
                "--count",
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
        )
        assert cli_result.returncode == 0, f"CLI --count exited with {cli_result.returncode}: {cli_result.stderr}"
        assert os.path.join("src", "lib.rs") in cli_result.stdout, "CLI count should include lib.rs"
        assert ":" in cli_result.stdout, "CLI count output should be file:count format"

    def test_cli_files_with_matches():
        cli_result = subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "pyfastgrep-cli",
                "--",
                "fn",
                "src",
                "--glob",
                "*.rs",
                "--ignore-case",
                "--files-with-matches",
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
        )
        assert cli_result.returncode == 0, f"CLI --files-with-matches exited with {cli_result.returncode}: {cli_result.stderr}"
        assert os.path.join("src", "lib.rs") in cli_result.stdout, "CLI files-with-matches should include lib.rs"
        # Output should be just filenames, no colons or line numbers
        lines = cli_result.stdout.strip().splitlines()
        assert len(lines) > 0, "Should have at least one filename"

    def test_cli_count_json():
        cli_result = subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "pyfastgrep-cli",
                "--",
                "fn",
                "src",
                "--glob",
                "*.rs",
                "--ignore-case",
                "--count",
                "--json",
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
        )
        assert cli_result.returncode == 0
        parsed = __import__("json").loads(cli_result.stdout)
        assert isinstance(parsed, list), "JSON count should be a list"
        assert all("file" in item and "count" in item for item in parsed), "Each item should have file and count keys"

    def test_fixed_strings_vs_regex():
        regex_results = pyfastgrep.search(".", source_root, "*.rs")
        literal_results = pyfastgrep.search(".", source_root, "*.rs", fixed_strings=True)
        assert len(literal_results) < len(regex_results), "Fixed strings should match fewer occurrences of literal '.'"

    def test_fixed_strings_count():
        count_regex = sum(c for _, c in pyfastgrep.count("fn", source_root, "*.rs"))
        count_literal = sum(c for _, c in pyfastgrep.count("fn", source_root, "*.rs", fixed_strings=True))
        assert count_regex == count_literal, "Literal word should match identically under regex and fixed-strings"

    def test_cli_fixed_strings():
        cli_regex = subprocess.run(
            ["cargo", "run", "-p", "pyfastgrep-cli", "--", ".", "src", "--glob", "*.rs", "--fixed-strings"],
            cwd=str(REPO_ROOT), capture_output=True, text=True,
        )
        assert cli_regex.returncode == 0, f"CLI fixed-strings exited with {cli_regex.returncode}: {cli_regex.stderr}"
        lines = cli_regex.stdout.strip().splitlines()
        # Literal dot should find far fewer matches than regex dot
        assert len(lines) < 300, f"Fixed strings should match fewer than 300 lines, got {len(lines)}"

    def test_cli_ast_functions():
        cli_result = subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "pyfastgrep-cli",
                "--",
                "build_config",
                "src",
                "--glob",
                "*.rs",
                "--functions",
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
        )

        assert cli_result.returncode == 0, f"CLI AST exited with {cli_result.returncode}: {cli_result.stderr}"
        assert "src/" in cli_result.stdout.replace(os.sep, "/"), "CLI AST output should include a src/ file"

    def test_context_search():
        results = pyfastgrep.search_with_context("fn", source_root, "*.rs", before_context=2, after_context=2)
        assert len(results) > 0, "Context search should find matches"
        first = results[0]
        assert len(first) == 5, "Result should be (file, line, content, before, after)"
        assert len(first[3]) <= 2, "before_context should be at most 2 lines"
        assert len(first[4]) <= 2, "after_context should be at most 2 lines"

    def test_context_search_zero():
        results = pyfastgrep.search_with_context("fn", source_root, "*.rs")
        first = results[0]
        assert len(first[3]) == 0, "Zero before_context by default"
        assert len(first[4]) == 0, "Zero after_context by default"

    def test_cli_context():
        cli_result = subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "pyfastgrep-cli",
                "--",
                "fn",
                "src",
                "--glob",
                "*.rs",
                "--context",
                "1",
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
        )
        assert cli_result.returncode == 0, f"CLI context exited with {cli_result.returncode}: {cli_result.stderr}"
        assert "-" in cli_result.stdout, "CLI context should have context lines"
        assert ":" in cli_result.stdout, "CLI context should have match lines"

    def test_byte_offset_json():
        results = pyfastgrep.search("fn", source_root, "*.rs", json=True, byte_offset=True)
        assert len(results) > 0, "Byte offset search should find matches"
        assert "byte_offset" in results[0], "JSON should include byte_offset key"
        assert isinstance(results[0]["byte_offset"], int), "byte_offset should be an integer"

    def test_byte_offset_json_absent_when_disabled():
        results = pyfastgrep.search("fn", source_root, "*.rs", json=True, byte_offset=False)
        assert "byte_offset" not in results[0], "byte_offset should not appear when disabled"

    def test_cli_byte_offset():
        cli_result = subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "pyfastgrep-cli",
                "--",
                "fn",
                "src",
                "--glob",
                "*.rs",
                "--byte-offset",
            ],
            cwd=str(REPO_ROOT),
            capture_output=True,
            text=True,
        )
        assert cli_result.returncode == 0, f"CLI byte-offset exited with {cli_result.returncode}: {cli_result.stderr}"
        # Output format: file:byte_offset:line:content
        lines = cli_result.stdout.strip().splitlines()
        assert len(lines) > 0, "Should have output"
        first_parts = lines[0].split(":")
        assert len(first_parts) >= 4, f"Format should be file:byte:line:content, got {lines[0]}"

    tests = [
        ("Case-sensitive search returns no matches", test_case_sensitive_search),
        ("Ignore-case batch search finds matches", test_ignore_case_search),
        ("Iterator search matches batch count", test_iterator_matches_batch),
        ("JSON output works for batch and iterator", test_json_output),
        ("CSV output works for batch and iterator", test_csv_output),
        ("Legacy tuple output stays compatible", test_legacy_output_and_consistency),
        ("CLI smoke test passes", test_cli_smoke),
        ("CLI CSV output passes", test_cli_csv),
        ("Python count works", test_python_count),
        ("Python files_with_matches works", test_python_files_with_matches),
        ("Count respects ignore_case", test_count_respects_ignore_case),
        ("CLI --count works", test_cli_count),
        ("CLI --files-with-matches works", test_cli_files_with_matches),
        ("CLI --count with --json works", test_cli_count_json),
        ("Fixed strings matches fewer than regex", test_fixed_strings_vs_regex),
        ("Fixed strings count matches regex for literal", test_fixed_strings_count),
        ("CLI --fixed-strings works", test_cli_fixed_strings),
        ("Context search returns correct structure", test_context_search),
        ("Context search zero context works", test_context_search_zero),
        ("CLI --context works", test_cli_context),
        ("Byte offset in JSON when enabled", test_byte_offset_json),
        ("Byte offset absent in JSON when disabled", test_byte_offset_json_absent_when_disabled),
        ("CLI --byte-offset works", test_cli_byte_offset),
        ("Ergonomic aliases work", test_ergonomic_aliases),
        ("AST function search finds matches", test_ast_functions),
        ("AST class search finds matches", test_ast_classes),
        ("AST import search finds matches", test_ast_imports),
        ("AST iterator matches batch count", test_ast_iterator_matches_batch),
        ("AST glob filter works", test_ast_glob_filter),
        ("CLI AST functions works", test_cli_ast_functions),
    ]

    passed = 0
    failed = 0

    for name, func in tests:
        if run_test(name, func):
            passed += 1
        else:
            failed += 1

    print("\nTest Summary")
    print(f"Total: {len(tests)}")
    print(f"Passed: {passed}")
    print(f"Failed: {failed}")

    if failed:
        sys.exit(1)

    print("All tests passed successfully!")


if __name__ == "__main__":
    main()
