#!/usr/bin/env python3
"""Coomi Lite complexity budget.

The lite line is defined by budgets, not by intent. Run this before every
release (and in CI) so a feature cannot quietly re-inflate the app.

Exit code is non-zero only when a *hard* limit is exceeded. The target column
is the direction of travel; the hard column stops regressions.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

RUST_DIR = ROOT / "apps" / "coomi-rs"
WEB_SRC = ROOT / "apps" / "web" / "src"
ROUTER = WEB_SRC / "router" / "index.ts"
TOOLS_SRC = RUST_DIR / "tools" / "src" / "lib.rs"
WEB_RS = RUST_DIR / "ui" / "src" / "web.rs"

BUDGETS = {
    "rust_loc": {"target": 20_000, "hard": 21_000},
    "web_loc": {"target": 8_000, "hard": 11_000},
    "pages": {"target": 14, "hard": 16},
    "tools": {"target": 24, "hard": 36},
    "api_routes": {"target": 70, "hard": 80},
}

# Product nouns that were deliberately removed. They may still appear in
# historical docs, but must not creep back into runtime/UI code.
FORBIDDEN_NOUNS = [
    "工作室",
    "工作台",
    "数字生命",
    "超载",
    "提示词库",
    "群聊",
    "协同工作台",
    "工单看板",
]

SKIP_PARTS = {"target", "node_modules", "dist", "build", ".git"}


def walk(root: Path, suffixes: tuple[str, ...]):
    for path in root.rglob("*"):
        if not path.is_file() or path.suffix not in suffixes:
            continue
        if any(part in SKIP_PARTS for part in path.parts):
            continue
        yield path


def count_lines(paths) -> int:
    total = 0
    for path in paths:
        try:
            with path.open("r", encoding="utf-8", errors="ignore") as handle:
                total += sum(1 for _ in handle)
        except OSError:
            pass
    return total


def count_matches(path: Path, pattern: str) -> int:
    if not path.is_file():
        return 0
    text = path.read_text(encoding="utf-8", errors="ignore")
    return len(re.findall(pattern, text))


def count_forbidden_nouns() -> list[tuple[str, int]]:
    hits: dict[str, int] = {noun: 0 for noun in FORBIDDEN_NOUNS}
    roots = [WEB_SRC, RUST_DIR / "ui" / "src", RUST_DIR / "services" / "src"]
    for root in roots:
        for path in walk(root, (".rs", ".ts", ".vue")):
            text = path.read_text(encoding="utf-8", errors="ignore")
            for noun in FORBIDDEN_NOUNS:
                hits[noun] += text.count(noun)
    return [(noun, hits[noun]) for noun in FORBIDDEN_NOUNS if hits[noun]]


def main() -> int:
    rust_loc = count_lines(walk(RUST_DIR, (".rs",)))
    web_loc = count_lines(walk(WEB_SRC, (".ts", ".vue", ".css")))
    pages = count_matches(ROUTER, r"path:\s*'")
    tools = count_matches(TOOLS_SRC, r"ToolSpec\s*\{")
    api_routes = count_matches(WEB_RS, r"\.route\(")

    values = {
        "rust_loc": rust_loc,
        "web_loc": web_loc,
        "pages": pages,
        "tools": tools,
        "api_routes": api_routes,
    }

    print("Coomi Lite budget")
    print(f"{'metric':<14}{'current':>9}{'target':>9}{'hard':>9}  status")
    failed = False
    for name, value in values.items():
        budget = BUDGETS[name]
        if value > budget["hard"]:
            status = "FAIL"
            failed = True
        elif value > budget["target"]:
            status = "warn"
        else:
            status = "ok"
        print(
            f"{name:<14}{value:>9}{budget['target']:>9}"
            f"{budget['hard']:>9}  {status}"
        )

    forbidden = count_forbidden_nouns()
    if forbidden:
        detail = ", ".join(f"{noun} x{count}" for noun, count in forbidden)
        print(f"\nforbidden product nouns: {detail}")
        print("  (warning only; keep runtime and UI code on existing nouns)")
    else:
        print("\nforbidden product nouns: none")

    if failed:
        print("\nBudget exceeded. Remove something before adding something.")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
