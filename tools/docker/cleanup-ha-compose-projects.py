#!/usr/bin/env python3

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
RUNS_ROOT = (REPO_ROOT / "cucumber_tests" / "ha" / "runs").resolve()


def run_command(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    completed = subprocess.run(
        list(args),
        check=False,
        capture_output=True,
        text=True,
    )
    if check and completed.returncode != 0:
        raise RuntimeError(
            f"command `{format_command(args)}` failed with status {completed.returncode}\n"
            f"stdout:\n{completed.stdout}\n"
            f"stderr:\n{completed.stderr}"
        )
    return completed


def format_command(args: tuple[str, ...]) -> str:
    return " ".join(args)


def read_compose_projects() -> list[dict[str, object]]:
    completed = run_command("docker", "compose", "ls", "--format", "json")
    payload = completed.stdout.strip()
    if not payload:
        return []
    parsed = json.loads(payload)
    if isinstance(parsed, list):
        return [entry for entry in parsed if isinstance(entry, dict)]
    raise RuntimeError("docker compose ls --format json returned an unexpected payload")


def config_path_in_runs_root(config_path: str) -> bool:
    if not config_path:
        return False
    resolved = Path(config_path).resolve()
    try:
        resolved.relative_to(RUNS_ROOT)
        return True
    except ValueError:
        return False


def cleanup_project(name: str, config_file: str | None) -> list[str]:
    removed = []
    removed.extend(
        remove_labeled_resources(
            kind="container",
            project=name,
            list_args=("ps", "-aq"),
            remove_args=("rm", "-f"),
        )
    )
    removed.extend(
        remove_labeled_resources(
            kind="network",
            project=name,
            list_args=("network", "ls", "-q"),
            remove_args=("network", "rm"),
        )
    )
    removed.extend(
        remove_labeled_resources(
            kind="volume",
            project=name,
            list_args=("volume", "ls", "-q"),
            remove_args=("volume", "rm", "-f"),
        )
    )
    removed.extend(remove_project_images(name))
    return removed


def remove_labeled_resources(
    kind: str,
    project: str,
    list_args: tuple[str, ...],
    remove_args: tuple[str, ...],
) -> list[str]:
    completed = run_command(
        "docker",
        *list_args,
        "--filter",
        f"label=com.docker.compose.project={project}",
        check=False,
    )
    if completed.returncode != 0:
        sys.stderr.write(
            f"cleanup warning: docker {' '.join(list_args)} failed for `{project}`\n"
            f"stdout:\n{completed.stdout}\n"
            f"stderr:\n{completed.stderr}\n"
        )
        return []

    resource_ids = [line.strip() for line in completed.stdout.splitlines() if line.strip()]
    if not resource_ids:
        return []

    removed = []
    for resource_id in resource_ids:
        removal = run_command(
            "docker",
            *remove_args,
            resource_id,
            check=False,
        )
        if removal.returncode == 0:
            removed.append(f"{kind}:{resource_id}")
        else:
            sys.stderr.write(
                f"cleanup warning: failed removing {kind} `{resource_id}` for `{project}`\n"
                f"stdout:\n{removal.stdout}\n"
                f"stderr:\n{removal.stderr}\n"
            )
    return removed


def remove_project_images(project: str) -> list[str]:
    removed = []
    for image_name in (
        f"{project}-observer",
        f"{project}-node-a",
        f"{project}-node-b",
        f"{project}-node-c",
    ):
        removal = run_command("docker", "image", "rm", "-f", image_name, check=False)
        if removal.returncode == 0:
            removed.append(f"image:{image_name}")
        elif "No such image" not in removal.stderr:
            sys.stderr.write(
                f"cleanup warning: failed removing image `{image_name}` for `{project}`\n"
                f"stdout:\n{removal.stdout}\n"
                f"stderr:\n{removal.stderr}\n"
            )
    return removed


def main() -> int:
    removed = []
    try:
        projects = read_compose_projects()
    except Exception as err:  # pragma: no cover - exercised through make preflight
        sys.stderr.write(f"failed reading docker compose projects: {err}\n")
        return 1

    for project in projects:
        name = str(project.get("Name", "")).strip()
        config_file = str(project.get("ConfigFiles", "")).strip()
        if not name.startswith("ha-"):
            continue
        if not config_path_in_runs_root(config_file):
            continue
        removed.extend(cleanup_project(name, config_file))

    if removed:
        for entry in removed:
            print(entry)
    else:
        print("no stale HA compose projects found")
    return 0


if __name__ == "__main__":
    sys.exit(main())
