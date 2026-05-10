from .pyfastgrep import search as _search
from .pyfastgrep import search_iter as _search_iter

def search(pattern, path=".", glob=None, max_results=None, ignore_case=False, json=False):
    """
    Search for a pattern in files.
    
    Args:
        pattern: Regex pattern to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)
        max_results: Maximum number of results to return (default: None)
        ignore_case: Case insensitive search (default: False)
        json: Return results as JSON objects (default: False)
    
    Returns:
        List of tuples (file, line, content) or JSON objects if json=True
    """
    return _search(pattern, path, glob, max_results, ignore_case, json)

def search_iter(pattern, path=".", glob=None, ignore_case=False, json=False):
    """
    Streaming iterator search for a pattern in files.
    
    Args:
        pattern: Regex pattern to search for
        path: Root directory to search in (default: ".")
        glob: File pattern to match (default: None)
        ignore_case: Case insensitive search (default: False)
        json: Return results as JSON objects (default: False)
    
    Returns:
        Iterator yielding tuples (file, line, content) or JSON objects if json=True
    """
    return _search_iter(pattern, path, glob, ignore_case, json)