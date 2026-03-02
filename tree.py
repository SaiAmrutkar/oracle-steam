import os

# Folders we don't want in the tree
IGNORE_DIRS = {
    "node_modules",
    "target",
    ".git",
    ".idea",
    ".vscode",
    "__pycache__"
}

# Files we usually don’t care about
IGNORE_FILES = {
    ".DS_Store",
}

def print_tree(path=".", prefix=""):
    entries = []
    for e in os.listdir(path):
        if e in IGNORE_DIRS:
            continue
        if e in IGNORE_FILES:
            continue
        entries.append(e)

    entries.sort()

    for i, entry in enumerate(entries):
        full_path = os.path.join(path, entry)
        is_last = i == len(entries) - 1
        connector = "└── " if is_last else "├── "
        print(prefix + connector + entry)

        if os.path.isdir(full_path):
            extension = "    " if is_last else "│   "
            print_tree(full_path, prefix + extension)


if __name__ == "__main__":
    print(os.path.basename(os.getcwd()) + "/")
    print_tree()
