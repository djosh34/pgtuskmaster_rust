#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


DEFAULT_TARGET_DIR = Path("/tmp/pgtuskmaster_rust-target")
TRACKED_ROOTS = ("src", "tests", "examples", "benches")


def parse_args() -> argparse.Namespace:
    script_dir = Path(__file__).resolve().parent
    repo_root = script_dir.parent

    parser = argparse.ArgumentParser(
        description=(
            "Fail when tracked Rust source files are not present in rustc dep-info "
            "for the current target dir."
        )
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=repo_root,
        help="Repository root. Defaults to the parent of this script directory.",
    )
    parser.add_argument(
        "--target-dir",
        type=Path,
        default=DEFAULT_TARGET_DIR,
        help="Cargo target dir containing rustc .d dep-info files.",
    )
    return parser.parse_args()


def is_tracked_candidate(rel_path: Path) -> bool:
    if rel_path == Path("build.rs"):
        return True
    return rel_path.parts != () and rel_path.parts[0] in TRACKED_ROOTS


def load_tracked_rust_files(repo_root: Path) -> set[Path]:
    result = subprocess.run(
        ["git", "-C", str(repo_root), "ls-files"],
        check=True,
        capture_output=True,
        text=True,
    )

    tracked_files = {
        Path(line)
        for line in result.stdout.splitlines()
        if line.endswith(".rs") or line == "build.rs"
    }
    return {path for path in tracked_files if is_tracked_candidate(path)}


def iter_dep_info_files(target_dir: Path) -> list[Path]:
    return sorted(target_dir.rglob("*.d"))


def load_compiled_rust_files(repo_root: Path, dep_info_files: list[Path]) -> set[Path]:
    repo_root_str = str(repo_root.resolve())
    compiled_files: set[Path] = set()

    for dep_info_path in dep_info_files:
        raw_text = dep_info_path.read_text(encoding="utf-8", errors="replace")
        normalized = raw_text.replace("\\\r\n", "").replace("\\\n", "")
        for token in normalized.split():
            if token.endswith(":"):
                continue

            candidate_path = Path(token)
            if candidate_path.suffix != ".rs" and candidate_path.name != "build.rs":
                continue

            if candidate_path.is_absolute():
                if not token.startswith(repo_root_str):
                    continue
                try:
                    compiled_files.add(candidate_path.resolve().relative_to(repo_root))
                except ValueError:
                    continue
                continue

            resolved_path = (repo_root / candidate_path).resolve()
            try:
                compiled_files.add(resolved_path.relative_to(repo_root))
            except ValueError:
                continue

    return {path for path in compiled_files if is_tracked_candidate(path)}


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.resolve()
    target_dir = args.target_dir.resolve()

    if not target_dir.is_dir():
        print(
            f"target dir does not exist: {target_dir}\n"
            "run `make check` first to populate rustc dep-info",
            file=sys.stderr,
        )
        return 1

    tracked_files = load_tracked_rust_files(repo_root)
    dep_info_files = iter_dep_info_files(target_dir)

    if not dep_info_files:
        print(
            f"no rustc dep-info files found under {target_dir}\n"
            "run `make check` first to populate rustc dep-info",
            file=sys.stderr,
        )
        return 1

    compiled_files = load_compiled_rust_files(repo_root, dep_info_files)
    if not compiled_files:
        print(
            f"found {len(dep_info_files)} dep-info files under {target_dir}, "
            "but none referenced repository Rust sources",
            file=sys.stderr,
        )
        return 1

    orphaned_files = sorted(tracked_files - compiled_files)
    if orphaned_files:
        print(
            "orphan Rust files found: tracked source files not used by any compiled target",
            file=sys.stderr,
        )
        for path in orphaned_files:
            print(path.as_posix(), file=sys.stderr)
        return 1

    print(
        f"orphan Rust file check passed: {len(tracked_files)} tracked files, "
        f"{len(compiled_files)} compiled files"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
