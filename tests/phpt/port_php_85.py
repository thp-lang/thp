#!/usr/bin/env python3
"""Port the php-src 8.5 base PHPT suites into THP's fixture layout.

The input is an extracted php-src release root. The output must not exist.
Only the core ``Zend/tests`` and top-level ``tests`` suites are imported;
extension and SAPI suites are intentionally outside this corpus.
"""

from __future__ import annotations

import argparse
import re
import shutil
from pathlib import Path


SOURCE_SECTIONS = {"FILE", "FILEEOF", "SKIPIF", "CLEAN"}
EXTERNAL_EXPECTATIONS = {
    "EXPECT_EXTERNAL": "EXPECT",
    "EXPECTF_EXTERNAL": "EXPECTF",
    "EXPECTREGEX_EXTERNAL": "EXPECTREGEX",
}
SECTION = re.compile(r"(?m)^--([A-Z_]+)--\r?$")
PHP_TAG = re.compile(r"<\?php\b", re.IGNORECASE)
EXTENSION_USE = re.compile(
    r"\b(?:dl|extension_loaded|get_extension_funcs|get_loaded_extensions)\s*\("
    r"|\bReflectionExtension\s*\("
    r"|\b(?:zend_)?extension\s*=",
    re.IGNORECASE,
)
COLLECTION_TYPE = "vector<mixed>|map<int|string, mixed>"


def _skip_quoted(source: str, index: int) -> int:
    quote = source[index]
    index += 1
    while index < len(source):
        if source[index] == "\\":
            index += 2
        elif source[index] == quote:
            return index + 1
        else:
            index += 1
    return index


def _skip_comment(source: str, index: int) -> int:
    if source.startswith("//", index) or (
        source[index] == "#" and not source.startswith("#[", index)
    ):
        newline = source.find("\n", index + 1)
        return len(source) if newline < 0 else newline
    if source.startswith("/*", index):
        end = source.find("*/", index + 2)
        return len(source) if end < 0 else end + 2
    return index


def _matching(source: str, start: int, opening: str, closing: str) -> int | None:
    depth = 1
    index = start + 1
    while index < len(source):
        char = source[index]
        if char in "'\"`":
            index = _skip_quoted(source, index)
            continue
        skipped = _skip_comment(source, index)
        if skipped != index:
            index = skipped
            continue
        if char == opening:
            depth += 1
        elif char == closing:
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return None


def _has_top_level_arrow(source: str) -> bool:
    depths = {"(": 0, "[": 0, "{": 0}
    closing = {")": "(", "]": "[", "}": "{"}
    index = 0
    while index < len(source):
        char = source[index]
        if char in "'\"`":
            index = _skip_quoted(source, index)
            continue
        skipped = _skip_comment(source, index)
        if skipped != index:
            index = skipped
            continue
        if source.startswith("=>", index) and all(depth == 0 for depth in depths.values()):
            return True
        if char in depths:
            depths[char] += 1
        elif char in closing and depths[closing[char]] > 0:
            depths[closing[char]] -= 1
        index += 1
    return False


def _split_top_level(source: str) -> list[str]:
    parts: list[str] = []
    depths = {"(": 0, "[": 0, "{": 0}
    closing = {")": "(", "]": "[", "}": "{"}
    start = 0
    index = 0
    while index < len(source):
        char = source[index]
        if char in "'\"`":
            index = _skip_quoted(source, index)
            continue
        skipped = _skip_comment(source, index)
        if skipped != index:
            index = skipped
            continue
        if char == "," and all(depth == 0 for depth in depths.values()):
            parts.append(source[start:index])
            start = index + 1
        elif char in depths:
            depths[char] += 1
        elif char in closing and depths[closing[char]] > 0:
            depths[closing[char]] -= 1
        index += 1
    parts.append(source[start:])
    return parts


def _explicit_integer_key(entry: str) -> int | None:
    match = re.match(r"\s*([+-]?\d+)\s*=>", entry)
    return int(match.group(1)) if match else None


def _rewrite_map_entries(source: str) -> str:
    parts = _split_top_level(source)
    next_integer_key = 0
    rewritten: list[str] = []
    for part in parts:
        if not part.strip():
            rewritten.append(part)
            continue
        if _has_top_level_arrow(part):
            explicit_key = _explicit_integer_key(part)
            if explicit_key is not None and explicit_key >= next_integer_key:
                next_integer_key = explicit_key + 1
            rewritten.append(part)
            continue
        if part.lstrip().startswith("..."):
            rewritten.append(part)
            continue
        leading_length = len(part) - len(part.lstrip())
        leading = part[:leading_length]
        value = part[leading_length:]
        rewritten.append(f"{leading}{next_integer_key} => {value}")
        next_integer_key += 1
    return ",".join(rewritten)


def _rewrite_collections(source: str) -> str:
    output: list[str] = []
    index = 0
    while index < len(source):
        char = source[index]
        if char in "'\"`":
            end = _skip_quoted(source, index)
            output.append(source[index:end])
            index = end
            continue
        skipped = _skip_comment(source, index)
        if skipped != index:
            output.append(source[index:skipped])
            index = skipped
            continue

        if (
            source[index : index + 5].lower() == "array"
            and (
                index == 0
                or not (
                    source[index - 1].isalnum()
                    or source[index - 1] in "_$"
                )
            )
            and (index + 5 == len(source) or not (source[index + 5].isalnum() or source[index + 5] == "_"))
        ):
            opening = index + 5
            while opening < len(source) and source[opening].isspace():
                opening += 1
            if opening < len(source) and source[opening] == "(":
                end = _matching(source, opening, "(", ")")
                if end is not None:
                    inner = _rewrite_collections(source[opening + 1 : end])
                    is_map = _has_top_level_arrow(inner)
                    if is_map:
                        inner = _rewrite_map_entries(inner)
                    brackets = ("{", "}") if is_map else ("[", "]")
                    output.extend((brackets[0], inner, brackets[1]))
                    index = end + 1
                    continue
            if source[max(0, index - 2) : index] not in {"->", "::"}:
                output.append(COLLECTION_TYPE)
                index += 5
                continue

        if char == "[":
            end = _matching(source, index, "[", "]")
            if end is not None:
                inner = _rewrite_collections(source[index + 1 : end])
                is_map = _has_top_level_arrow(inner)
                if is_map:
                    inner = _rewrite_map_entries(inner)
                brackets = ("{", "}") if is_map else ("[", "]")
                output.extend((brackets[0], inner, brackets[1]))
                index = end + 1
                continue

        output.append(char)
        index += 1
    return "".join(output)


def port_source(source: str) -> str:
    # THP source is UTF-8. Preserve binary expectation sections separately,
    # but make unsupported legacy source encodings safe to discover and skip.
    source = source.encode("utf-8", errors="replace").decode("utf-8")
    source = PHP_TAG.sub("<?thp", source)
    source = source.replace("<?=", "<?thp echo ")
    source = re.sub(r"<\?(?!thp\b|xml\b)", "<?thp", source, flags=re.IGNORECASE)
    try:
        return _rewrite_collections(source)
    except RecursionError:
        # A handful of parser stress tests intentionally contain thousands of
        # nested brackets. They have no meaningful collection shape to port.
        return source


def _sections(source: str) -> list[tuple[str, str]]:
    matches = list(SECTION.finditer(source))
    if not matches:
        return []
    sections: list[tuple[str, str]] = []
    for position, match in enumerate(matches):
        content_start = match.end()
        if content_start < len(source) and source[content_start] == "\n":
            content_start += 1
        content_end = matches[position + 1].start() if position + 1 < len(matches) else len(source)
        sections.append((match.group(1), source[content_start:content_end]))
    return sections


def port_phpt(path: Path, source_root: Path) -> str:
    text = path.read_text(encoding="utf-8", errors="surrogateescape")
    sections = _sections(text)
    if not sections:
        return port_source(text)

    has_extensions = any(name == "EXTENSIONS" for name, _ in sections)
    executable = "\n".join(content for name, content in sections if name in SOURCE_SECTIONS)
    needs_extension_skip = not has_extensions and EXTENSION_USE.search(executable) is not None

    output: list[str] = []
    extension_skip_added = False
    for name, content in sections:
        if needs_extension_skip and not extension_skip_added and name in {"FILE", "FILEEOF"}:
            output.append("--EXTENSIONS--\nupstream dynamic extension dependency\n")
            extension_skip_added = True

        if name in EXTERNAL_EXPECTATIONS:
            external = (path.parent / content.strip()).resolve()
            if external.is_file() and external.is_relative_to(source_root.resolve()):
                name = EXTERNAL_EXPECTATIONS[name]
                content = external.read_text(encoding="utf-8", errors="surrogateescape")
                if content and not content.endswith("\n"):
                    content += "\n"
        elif name == "FILEEOF":
            name = "FILE"
        elif name in {"FLAKY", "XFAIL"}:
            metadata_name = name.lower()
            name = "DESCRIPTION"
            content = f"Upstream {metadata_name} metadata: {content.strip()}\n"
        elif name == "WHITESPACE_SENSITIVE":
            continue

        if name in SOURCE_SECTIONS:
            content = port_source(content)
        output.append(f"--{name}--\n{content}")

    return "".join(output)


def import_suite(source_root: Path, output_root: Path) -> None:
    if output_root.exists():
        raise SystemExit(f"output already exists: {output_root}")
    output_root.mkdir(parents=True)

    for source_name, output_name in (("Zend/tests", "zend"), ("tests", "core")):
        source_dir = source_root / source_name
        if not source_dir.is_dir():
            raise SystemExit(f"missing php-src suite: {source_dir}")
        destination = output_root / output_name
        shutil.copytree(source_dir, destination)
        for path in destination.rglob("*.phpt"):
            relative = path.relative_to(destination)
            original = source_dir / relative
            path.write_text(
                port_phpt(original, source_root),
                encoding="utf-8",
                errors="surrogateescape",
            )
        for path in destination.rglob("*.inc"):
            path.write_text(
                port_source(path.read_text(encoding="utf-8", errors="surrogateescape")),
                encoding="utf-8",
                errors="surrogateescape",
            )

    shutil.copy2(source_root / "LICENSE", output_root / "LICENSE")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("php_source", type=Path, help="extracted php-src release root")
    parser.add_argument("output", type=Path, help="new output directory")
    arguments = parser.parse_args()
    import_suite(arguments.php_source.resolve(), arguments.output.resolve())


if __name__ == "__main__":
    main()
