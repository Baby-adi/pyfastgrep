from .pyfastgrep import search as _search
from .pyfastgrep import search_iter as _search_iter
from .pyfastgrep import search_count as _search_count
from .pyfastgrep import search_files_with_matches as _search_files_with_matches
from .pyfastgrep import search_with_context as _search_with_context
from .pyfastgrep import search_functions as _search_functions
from .pyfastgrep import search_classes as _search_classes
from .pyfastgrep import search_imports as _search_imports
from .pyfastgrep import search_functions_iter as _search_functions_iter
from .pyfastgrep import search_classes_iter as _search_classes_iter
from .pyfastgrep import search_imports_iter as _search_imports_iter


def _normalize_search_options(
    path,
    glob,
    max_results,
    ignore_case,
    fixed_strings,
    byte_offset,
    json,
    csv,
    output_path,
    kwargs,
):
    if "root" in kwargs:
        path = kwargs.pop("root")
    if "limit" in kwargs:
        max_results = kwargs.pop("limit")
    if "case_insensitive" in kwargs:
        ignore_case = kwargs.pop("case_insensitive")
    if "fixed_strings" in kwargs:
        fixed_strings = kwargs.pop("fixed_strings")
    if "byte_offset" in kwargs:
        byte_offset = kwargs.pop("byte_offset")
    if "as_json" in kwargs:
        json = kwargs.pop("as_json")
    if "as_csv" in kwargs:
        kwargs["csv"] = kwargs.pop("as_csv")

    csv_output = csv
    if "csv" in kwargs:
        csv_output = kwargs.pop("csv")
    if "output_path" in kwargs:
        output_path = kwargs.pop("output_path")

    if kwargs:
        unexpected = ", ".join(sorted(kwargs.keys()))
        raise TypeError(f"unexpected keyword argument(s): {unexpected}")

    return (
        path,
        glob,
        max_results,
        ignore_case,
        fixed_strings,
        byte_offset,
        json,
        csv_output,
        output_path,
    )


def search(
    pattern,
    path=".",
    glob=None,
    max_results=None,
    ignore_case=False,
    json=False,
    csv=False,
    output_path=None,
    fixed_strings=False,
    byte_offset=False,
    **kwargs,
):
    """
    Search for a pattern in files.

    Args:
        pattern: Regex pattern to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)
        max_results: Maximum number of results to return (default: None)
        ignore_case: Case insensitive search (default: False)
        json: Return results as JSON objects (default: False)
        fixed_strings: Treat pattern as literal string (default: False)
        byte_offset: Include byte offset in results (default: False)

    Returns:
        List of tuples (file, line, content) or JSON objects if json=True
    """
    (
        path,
        glob,
        max_results,
        ignore_case,
        fixed_strings,
        byte_offset,
        json,
        csv,
        output_path,
    ) = _normalize_search_options(
        path,
        glob,
        max_results,
        ignore_case,
        fixed_strings,
        byte_offset,
        json,
        csv,
        output_path,
        kwargs,
    )
    return _search(
        pattern,
        path,
        glob,
        max_results,
        ignore_case,
        json,
        csv,
        output_path,
        fixed_strings,
        byte_offset,
    )


def search_iter(
    pattern,
    path=".",
    glob=None,
    ignore_case=False,
    json=False,
    csv=False,
    output_path=None,
    fixed_strings=False,
    byte_offset=False,
    **kwargs,
):
    """
    Streaming iterator search for a pattern in files.

    Args:
        pattern: Regex pattern to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)
        ignore_case: Case insensitive search (default: False)
        json: Return results as JSON objects (default: False)
        fixed_strings: Treat pattern as literal string (default: False)
        byte_offset: Include byte offset in results (default: False)

    Returns:
        Iterator yielding tuples (file, line, content) or JSON objects if json=True
    """
    path, glob, _, ignore_case, fixed_strings, byte_offset, json, csv, output_path = (
        _normalize_search_options(
            path,
            glob,
            None,
            ignore_case,
            fixed_strings,
            byte_offset,
            json,
            csv,
            output_path,
            kwargs,
        )
    )
    return _search_iter(
        pattern,
        path,
        glob,
        ignore_case,
        json,
        csv,
        output_path,
        fixed_strings,
        byte_offset,
    )


def count(
    pattern, path=".", glob=None, ignore_case=False, fixed_strings=False, **kwargs
):
    """
    Count matches per file.

    Args:
        pattern: Regex pattern to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)
        ignore_case: Case insensitive search (default: False)
        fixed_strings: Treat pattern as literal string (default: False)

    Returns:
        List of tuples (file, count)
    """
    if "root" in kwargs:
        path = kwargs.pop("root")
    if "case_insensitive" in kwargs:
        ignore_case = kwargs.pop("case_insensitive")
    if "fixed_strings" in kwargs:
        fixed_strings = kwargs.pop("fixed_strings")
    if kwargs:
        unexpected = ", ".join(sorted(kwargs.keys()))
        raise TypeError(f"unexpected keyword argument(s): {unexpected}")
    return _search_count(pattern, path, glob, ignore_case, fixed_strings)


def files_with_matches(
    pattern, path=".", glob=None, ignore_case=False, fixed_strings=False, **kwargs
):
    """
    Return filenames that contain at least one match.

    Args:
        pattern: Regex pattern to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)
        ignore_case: Case insensitive search (default: False)
        fixed_strings: Treat pattern as literal string (default: False)

    Returns:
        List of filenames
    """
    if "root" in kwargs:
        path = kwargs.pop("root")
    if "case_insensitive" in kwargs:
        ignore_case = kwargs.pop("case_insensitive")
    if "fixed_strings" in kwargs:
        fixed_strings = kwargs.pop("fixed_strings")
    if kwargs:
        unexpected = ", ".join(sorted(kwargs.keys()))
        raise TypeError(f"unexpected keyword argument(s): {unexpected}")
    return _search_files_with_matches(pattern, path, glob, ignore_case, fixed_strings)


def search_with_context(
    pattern,
    path=".",
    glob=None,
    before_context=0,
    after_context=0,
    ignore_case=False,
    fixed_strings=False,
    **kwargs,
):
    """
    Search with surrounding context lines.

    Args:
        pattern: Regex pattern to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)
        before_context: Number of lines before each match (default: 0)
        after_context: Number of lines after each match (default: 0)
        ignore_case: Case insensitive search (default: False)
        fixed_strings: Treat pattern as literal string (default: False)

    Returns:
        List of tuples (file, line, content, before_lines, after_lines)
    """
    if "root" in kwargs:
        path = kwargs.pop("root")
    if "case_insensitive" in kwargs:
        ignore_case = kwargs.pop("case_insensitive")
    if "fixed_strings" in kwargs:
        fixed_strings = kwargs.pop("fixed_strings")
    if kwargs:
        unexpected = ", ".join(sorted(kwargs.keys()))
        raise TypeError(f"unexpected keyword argument(s): {unexpected}")
    return _search_with_context(
        pattern, path, glob, before_context, after_context, ignore_case, fixed_strings
    )


def search_functions(target_name, path=".", glob=None):
    """
    Search for a function by name using AST parsing.

    Args:
        target_name: Function name to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)

    Returns:
        List of tuples (file, line, content)
    """
    return _search_functions(target_name, path, glob)


def search_classes(target_name, path=".", glob=None):
    """
    Search for a class/struct by name using AST parsing.

    Args:
        target_name: Class/struct name to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)

    Returns:
        List of tuples (file, line, content)
    """
    return _search_classes(target_name, path, glob)


def search_imports(target_name, path=".", glob=None):
    """
    Search for an import/use statement containing a name using AST parsing.

    Args:
        target_name: Import name to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)

    Returns:
        List of tuples (file, line, content)
    """
    return _search_imports(target_name, path, glob)


def search_functions_iter(target_name, path=".", glob=None):
    """
    Streaming iterator search for a function by name using AST parsing.

    Args:
        target_name: Function name to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)

    Returns:
        Iterator yielding tuples (file, line, content)
    """
    return _search_functions_iter(target_name, path, glob)


def search_classes_iter(target_name, path=".", glob=None):
    """
    Streaming iterator search for a class/struct by name using AST parsing.

    Args:
        target_name: Class/struct name to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)

    Returns:
        Iterator yielding tuples (file, line, content)
    """
    return _search_classes_iter(target_name, path, glob)


def search_imports_iter(target_name, path=".", glob=None):
    """
    Streaming iterator search for an import/use statement using AST parsing.

    Args:
        target_name: Import name to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)

    Returns:
        Iterator yielding tuples (file, line, content)
    """
    return _search_imports_iter(target_name, path, glob)
