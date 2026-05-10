import pyfastgrep
import sys

def main():
    print("Running pyfastgrep test suite...")
    
    # 1. Test basic case-sensitive search
    # Searching for uppercase 'FN' which shouldn't match lowercase 'fn'
    res_sensitive = pyfastgrep.search("FN", "src", "*.rs", None, False)
    assert len(res_sensitive) == 0, f"Expected 0 results for case-sensitive 'FN', got {len(res_sensitive)}"
    
    # 2. Test ignore_case search
    res_ignore = pyfastgrep.search("FN", "src", "*.rs", None, True)
    assert len(res_ignore) > 0, "Expected >0 results for 'FN' with ignore_case=True"
    
    # 3. Test iterator search
    iter_ignore = list(pyfastgrep.search_iter("FN", "src", "*.rs", True))
    assert len(iter_ignore) == len(res_ignore), "Batch and Iter search results count mismatch"

    print("All tests passed successfully!")

if __name__ == "__main__":
    try:
        main()
    except AssertionError as e:
        print(f"TEST FAILED: {e}")
        sys.exit(1)
