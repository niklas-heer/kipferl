"""Count a repository's files by extension, excluding generated directories."""
import argparse
import json
import os
from pathlib import Path

parser = argparse.ArgumentParser(description="Count a repository's files by extension, excluding generated directories.")
parser.add_argument("directory", help="Repository or source directory")
args = parser.parse_args()

ignored = [".git", ".hg", ".svn", ".kipferl", ".venv", "__pycache__", "node_modules", "target", "dist"]
pending = [Path(args.directory)]
visited = set()
counts = {}
while pending:
    directory = pending.pop()
    resolved = str(directory.resolve())
    if resolved in visited:
        continue
    visited.add(resolved)
    for name in sorted(os.listdir(str(directory))):
        path = directory / name
        if path.is_dir():
            if name not in ignored:
                pending.append(path)
        elif path.is_file():
            extension = path.suffix or "(no extension)"
            counts[extension] = counts.get(extension, 0) + 1
print(json.dumps(counts, sort_keys=True))
