import os

ROOT_DIR = os.getcwd()   # or r"D:\CG"
OUTPUT_FILE = "code.txt"

# Folders to ignore
IGNORE_DIRS = {".git"}

tree_lines = []
file_contents = []

def scan_tree(path, prefix=""):
    try:
        items = sorted(os.listdir(path))
    except PermissionError:
        return

    # Filter out ignored directories
    items = [item for item in items if item not in IGNORE_DIRS]

    for index, name in enumerate(items):
        full_path = os.path.join(path, name)
        is_last = index == len(items) - 1

        connector = "└── " if is_last else "├── "
        tree_lines.append(prefix + connector + name)

        if os.path.isdir(full_path):
            new_prefix = prefix + ("    " if is_last else "│   ")
            scan_tree(full_path, new_prefix)
        else:
            read_file(full_path)

def read_file(path):
    try:
        with open(path, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read()
    except Exception:
        content = "[Could not read file]"

    file_contents.append("\n")
    file_contents.append("=" * 30)
    file_contents.append(f"FILE: {os.path.relpath(path, ROOT_DIR)}")
    file_contents.append("=" * 30)
    file_contents.append(content)

# Root folder
tree_lines.append(os.path.basename(ROOT_DIR) + "/")
scan_tree(ROOT_DIR)

with open(OUTPUT_FILE, "w", encoding="utf-8") as out:
    out.write("\n".join(tree_lines))
    out.write("\n\n")
    out.write("\n".join(file_contents))

print("✔ Folder tree + file contents written to code.txt (excluding .git folder)")
