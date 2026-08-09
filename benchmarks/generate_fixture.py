#!/usr/bin/env python3
"""Generate a deterministic, tool-neutral file-search benchmark fixture."""

from __future__ import annotations

import argparse
from pathlib import Path


EXTENSIONS = ("rs", "txt", "md", "json", "toml", "yaml", "py", "js", "ts", "css")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    parser.add_argument("--directories", type=int, default=250)
    parser.add_argument("--files-per-directory", type=int, default=200)
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    args.root.mkdir(parents=True, exist_ok=True)
    if any(args.root.iterdir()):
        raise SystemExit(f"refusing to populate non-empty directory: {args.root}")

    for directory_index in range(args.directories):
        directory = args.root / f"dir_{directory_index:04d}"
        directory.mkdir()
        for file_index in range(args.files_per_directory):
            extension = EXTENSIONS[file_index % len(EXTENSIONS)]
            path = directory / f"file_{file_index:04d}.{extension}"
            path.write_bytes(b"benchmark fixture\n")

    total = args.directories * args.files_per_directory
    print(f"generated {total} files below {args.root}")


if __name__ == "__main__":
    main()
