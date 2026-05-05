import pyfastgrep

# batch
print(pyfastgrep.search("fn", "src", "*.rs", 10))

# streaming
for r in pyfastgrep.search_iter("fn", "src", "*.rs"):
    print(r)