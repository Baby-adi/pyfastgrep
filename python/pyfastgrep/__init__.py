from .pyfastgrep import search as _search
from .pyfastgrep import search_iter as _search_iter

def search(pattern, path=".", glob=None, max_results=None):
    return _search(pattern, path, glob, max_results)

def search_iter(pattern, path=".", glob=None):
    return _search_iter(pattern, path, glob)