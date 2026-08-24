#!/usr/bin/env python3
"""fn-census — measure Rust functions honestly, for the exemplar hunt.

WHY THIS EXISTS. Twice this metric was taken by hand and twice it was wrong in
the same way: a function's size was read as the distance from its `fn` line to
the end of the file, which silently swallowed the `#[cfg(test)] mod tests`
below it. That produced a target table claiming 388 / 451 / 590-line bodies
that are really 87 / 35 / 72 — and it steered the campaign for three sessions.
The second attempt fixed the end and broke the start, reading a documented
function as 0% comment because its `///` block sits ABOVE the `fn` line.

Both are the same class: a function's extent was guessed from one anchor.
This tool takes both boundaries from the source, so neither guess is available
to make. It is the instrument; do not re-derive these numbers by eye.

WHAT IT MEASURES, per function:
  lines     — the whole item: doc comments and attributes THROUGH the closing brace
  comment   — `//`-family lines inside that extent (doc comments included)
  nesting   — maximum brace depth inside the body, braces in strings/chars/
              comments excluded
Test modules are excluded by default (`--tests` includes them).

USAGE
  wat-scripts/hunt/fn-census.py src/rete                # every fn, largest first
  wat-scripts/hunt/fn-census.py src/rete --top 15
  wat-scripts/hunt/fn-census.py src/rete/purity.rs --name intrinsic_meta
  wat-scripts/hunt/fn-census.py src/rete --min-lines 200 --format md
No dependencies; plain python3, no venv needed.
"""
import argparse
import pathlib
import re
import sys

FN_RE = re.compile(
    r"^(?P<indent>\s*)"
    r"(?:pub(?:\s*\([^)]*\))?\s+)?"
    r"(?:default\s+)?(?:const\s+)?(?:async\s+)?(?:unsafe\s+)?"
    r"(?:extern\s+\"[^\"]*\"\s+)?"
    r"fn\s+(?P<name>[A-Za-z_][A-Za-z0-9_]*)"
)
LEAD_RE = re.compile(r"^\s*(///|//!|//|#\[|#!\[)")
ATTR_OPEN_RE = re.compile(r"^\s*#!?\[")


def strip_noise(line, in_block):
    """Return (code_without_strings_chars_comments, still_in_block_comment)."""
    out = []
    i = 0
    n = len(line)
    while i < n:
        c = line[i]
        if in_block:
            if c == "*" and i + 1 < n and line[i + 1] == "/":
                in_block = False
                i += 2
                continue
            i += 1
            continue
        if c == "/" and i + 1 < n and line[i + 1] == "/":
            break  # line comment: rest is noise
        if c == "/" and i + 1 < n and line[i + 1] == "*":
            in_block = True
            i += 2
            continue
        if c == '"':
            i += 1
            while i < n:
                if line[i] == "\\":
                    i += 2
                    continue
                if line[i] == '"':
                    i += 1
                    break
                i += 1
            continue
        if c == "'":
            # char literal or a lifetime; lifetimes carry no brace, so a cheap
            # skip of a 1-2 char literal is enough and a lifetime falls through.
            m = re.match(r"'(?:\\.|[^\\'])'", line[i:])
            if m:
                i += m.end()
                continue
            i += 1
            continue
        out.append(c)
        i += 1
    return "".join(out), in_block


def test_mod_ranges(lines):
    """Line ranges (0-based, inclusive) of `#[cfg(test)] mod ... { }` blocks."""
    ranges = []
    in_block = False
    for idx, line in enumerate(lines):
        if "#[cfg(test)]" not in line:
            continue
        j = idx + 1
        while j < len(lines) and not re.match(r"\s*(pub\s+)?mod\s", lines[j]):
            if lines[j].strip() and not LEAD_RE.match(lines[j]):
                j = None
                break
            j += 1
        if j is None or j >= len(lines):
            continue
        depth = 0
        started = False
        k = j
        while k < len(lines):
            code, in_block = strip_noise(lines[k], in_block)
            depth += code.count("{") - code.count("}")
            if "{" in code:
                started = True
            if started and depth <= 0:
                ranges.append((idx, k))
                break
            k += 1
    return ranges


def measure_file(path, include_tests):
    lines = path.read_text(errors="replace").split("\n")
    skip = [] if include_tests else test_mod_ranges(lines)

    def in_skip(i):
        return any(a <= i <= b for a, b in skip)

    results = []
    i = 0
    while i < len(lines):
        if in_skip(i):
            i += 1
            continue
        m = FN_RE.match(lines[i])
        if not m:
            i += 1
            continue
        # Walk BACK over the item's doc comments and attributes — the boundary
        # the second hand-measurement missed.
        start = i
        j = i - 1
        while j >= 0 and LEAD_RE.match(lines[j]) and not in_skip(j):
            start = j
            j -= 1
        # Walk FORWARD to the real closing brace, braces-in-noise excluded.
        depth = 0
        started = False
        end = None
        in_block = False
        k = i
        while k < len(lines):
            code, in_block = strip_noise(lines[k], in_block)
            if not started and ";" in code and "{" not in code:
                break  # a trait signature / extern decl: no body
            depth += code.count("{") - code.count("}")
            if "{" in code:
                started = True
            if started and depth <= 0:
                end = k
                break
            k += 1
        if end is None:
            i += 1
            continue
        body = lines[start : end + 1]
        comment = sum(1 for l in body if re.match(r"\s*(//|/\*)", l))
        depth = maxd = 0
        in_block = False
        for l in body:
            code, in_block = strip_noise(l, in_block)
            depth += code.count("{") - code.count("}")
            maxd = max(maxd, depth)
        results.append(
            {
                "file": str(path),
                "name": m.group("name"),
                "start": start + 1,
                "end": end + 1,
                "lines": len(body),
                "comment": comment,
                "pct": round(comment * 100 / len(body)),
                "nesting": maxd,
            }
        )
        i = end + 1
    return results


def main():
    ap = argparse.ArgumentParser(description="Measure Rust functions for the exemplar hunt.")
    ap.add_argument("path", help="a .rs file or a directory to walk")
    ap.add_argument("--name", help="only this function")
    ap.add_argument("--top", type=int, help="show only the N largest")
    ap.add_argument("--min-lines", type=int, default=0)
    ap.add_argument("--tests", action="store_true", help="include #[cfg(test)] modules")
    ap.add_argument("--format", choices=["txt", "md"], default="txt")
    a = ap.parse_args()

    root = pathlib.Path(a.path)
    files = sorted(root.rglob("*.rs")) if root.is_dir() else [root]
    if not files:
        print(f"no .rs files under {root}", file=sys.stderr)
        return 1

    rows = []
    for f in files:
        rows.extend(measure_file(f, a.tests))
    if a.name:
        rows = [r for r in rows if r["name"] == a.name]
    rows = [r for r in rows if r["lines"] >= a.min_lines]
    rows.sort(key=lambda r: -r["lines"])
    if a.top:
        rows = rows[: a.top]

    if not rows:
        print("no functions matched")
        return 0
    if a.format == "md":
        print("| function | file:lines | lines | comment | nesting |")
        print("|---|---|---:|---:|---:|")
        for r in rows:
            print(
                f"| `{r['name']}` | `{r['file']}:{r['start']}-{r['end']}` | "
                f"{r['lines']} | {r['pct']}% | {r['nesting']} |"
            )
    else:
        w = max(len(r["name"]) for r in rows)
        for r in rows:
            print(
                f"{r['name']:<{w}}  {r['file']}:{r['start']}-{r['end']}  "
                f"lines={r['lines']:<5} comment={r['comment']:<4}({r['pct']}%)  "
                f"nesting={r['nesting']}"
            )
    print(f"\n{len(rows)} function(s); {len(files)} file(s) scanned"
          f"{'' if a.tests else '; #[cfg(test)] modules excluded'}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
