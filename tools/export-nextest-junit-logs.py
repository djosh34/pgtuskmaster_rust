#!/usr/bin/env python3
import shutil
import sys
import xml.etree.ElementTree as ET
from pathlib import Path


def slugify(value: str) -> str:
    chars = []
    for ch in value:
        if ch.isalnum() or ch in "._-":
            chars.append(ch)
        else:
            chars.append("_")
    slug = "".join(chars).strip("_")
    return slug or "test"


def text_of(node: ET.Element | None) -> str:
    if node is None:
        return ""
    return node.text or ""


def write_test_log(case: ET.Element, output_dir: Path) -> None:
    classname = case.attrib.get("classname", "")
    name = case.attrib.get("name", "")
    time_value = case.attrib.get("time", "")
    full_name = f"{classname}::{name}" if classname else name
    status = "passed"

    skipped = case.find("skipped")
    failure = case.find("failure")
    error = case.find("error")
    if skipped is not None:
        status = "skipped"
    elif failure is not None:
        status = "failed"
    elif error is not None:
        status = "error"

    sections = [
        f"name: {full_name}",
        f"status: {status}",
        f"time_seconds: {time_value}",
    ]

    if failure is not None:
        failure_message = failure.attrib.get("message", "")
        if failure_message:
            sections.append(f"failure_message: {failure_message}")
        failure_text = text_of(failure).strip()
        if failure_text:
            sections.append("failure_text:\n" + failure_text)

    if error is not None:
        error_message = error.attrib.get("message", "")
        if error_message:
            sections.append(f"error_message: {error_message}")
        error_text = text_of(error).strip()
        if error_text:
            sections.append("error_text:\n" + error_text)

    system_out = text_of(case.find("system-out")).rstrip()
    system_err = text_of(case.find("system-err")).rstrip()
    if system_out:
        sections.append("stdout:\n" + system_out)
    if system_err:
        sections.append("stderr:\n" + system_err)

    body = "\n\n".join(sections).rstrip() + "\n"
    file_name = f"{slugify(full_name)}.log"
    (output_dir / file_name).write_text(body, encoding="utf-8")


def main() -> int:
    if len(sys.argv) != 3:
        print(
            "usage: export-nextest-junit-logs.py <junit-xml-path> <output-dir>",
            file=sys.stderr,
        )
        return 1

    junit_path = Path(sys.argv[1])
    output_dir = Path(sys.argv[2])
    if not junit_path.is_file():
        print(f"junit file not found: {junit_path}", file=sys.stderr)
        return 1

    root = ET.parse(junit_path).getroot()
    if output_dir.exists():
        shutil.rmtree(output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    cases = list(root.iter("testcase"))
    for case in cases:
        write_test_log(case, output_dir)

    summary_path = output_dir / "SUMMARY.txt"
    summary_path.write_text(
        f"exported {len(cases)} test logs from {junit_path}\n", encoding="utf-8"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
