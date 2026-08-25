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
    return measure_source(path.read_text(errors="replace"), include_tests, str(path))


def measure_source(text, include_tests, label="<source>"):
    lines = text.split("\n")
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
                "file": label,
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


# ── SELF-TEST ────────────────────────────────────────────────────────────────
#
# WHY THIS EXISTS, and why every case below is a real incident rather than an
# invented one. This tool replaced a hand-measurement that was wrong TWICE, in
# opposite directions, and then SHIPPED WITH THE SECOND BUG ITSELF:
#   - take 1 measured a function as `fn`-line -> end-of-file, swallowing the
#     `#[cfg(test)] mod tests` below it. It reported 388/451/590-line bodies that
#     are really 87/35/72, and steered the exemplar hunt at the WRONG functions
#     for three sessions.
#   - take 2 fixed the end and broke the start: it began at the `fn` line, so a
#     function whose `///` block sits ABOVE it read as 0% comment. That is how a
#     fully-documented function got filed as undocumented.
# An unverified measurer is the exact defect class this tool exists to remove, so
# it verifies itself. Each case is named for the incident it prevents.
# Run: wat-scripts/hunt/fn-census.py --selftest   (exit 0 = every case holds)

SELFTEST_CASES = [
    (
        "take-1: a #[cfg(test)] module below must NOT be swallowed",
        "fn target(x: u32) -> u32 {\n"
        "    x + 1\n"
        "}\n"
        "\n"
        "#[cfg(test)]\n"
        "mod tests {\n"
        "    use super::*;\n"
        "    #[test]\n"
        "    fn t() {\n"
        "        assert_eq!(target(1), 2);\n"
        "    }\n"
        "}\n",
        {"name": "target", "start": 1, "end": 3, "lines": 3, "nesting": 1},
    ),
    (
        "take-2: a doc block and attributes ABOVE the fn are part of the item",
        "/// One.\n"
        "/// Two.\n"
        "#[inline]\n"
        "fn target() -> u32 {\n"
        "    1\n"
        "}\n",
        {"name": "target", "start": 1, "end": 6, "lines": 6, "comment": 2, "nesting": 1},
    ),
    (
        "a brace inside a string literal must not open a block",
        "fn target() -> &'static str {\n"
        '    let _s = "{{{ not a block";\n'
        '    "}"\n'
        "}\n",
        {"name": "target", "start": 1, "end": 4, "lines": 4, "nesting": 1},
    ),
    (
        "braces inside line and block comments must not open a block",
        "fn target() -> u32 {\n"
        "    // { this brace is prose\n"
        "    /* and { these } too */\n"
        "    0\n"
        "}\n",
        {"name": "target", "start": 1, "end": 5, "lines": 5, "comment": 2, "nesting": 1},
    ),
    (
        "a char-literal brace must not open a block",
        "fn target(c: char) -> bool {\n"
        "    c == '{' || c == '}'\n"
        "}\n",
        {"name": "target", "start": 1, "end": 3, "lines": 3, "nesting": 1},
    ),
    (
        "a trait signature with no body is not a measurable item",
        "trait T {\n"
        "    fn target(&self) -> u32;\n"
        "}\n"
        "\n"
        "fn other() -> u32 {\n"
        "    1\n"
        "}\n",
        {"name": "other", "start": 5, "end": 7, "lines": 3, "nesting": 1},
    ),
    (
        "nesting is real, and a lifetime tick is not a char literal",
        "fn target<'a>(v: &'a [u32]) -> u32 {\n"
        "    for x in v {\n"
        "        if *x > 0 {\n"
        "            return *x;\n"
        "        }\n"
        "    }\n"
        "    0\n"
        "}\n",
        {"name": "target", "start": 1, "end": 8, "lines": 8, "nesting": 3},
    ),
]


def run_selftest():
    failures = []
    for title, src, want in SELFTEST_CASES:
        rows = {r["name"]: r for r in measure_source(src, include_tests=False)}
        got = rows.get(want["name"])
        if got is None:
            failures.append(title + "\n    no row for " + repr(want["name"])
                            + "; got " + repr(sorted(rows)))
            continue
        for k, v in want.items():
            if k == "name":
                continue
            if got[k] != v:
                failures.append(title + "\n    " + want["name"] + "." + k
                                + ": expected " + repr(v) + ", got " + repr(got[k]))
    # The take-1 case from the other side: with --tests the test fn MUST appear,
    # so "excluded by default" is proven to be a filter and not a parse failure.
    with_tests = {r["name"] for r in measure_source(SELFTEST_CASES[0][1], include_tests=True)}
    if "t" not in with_tests:
        failures.append("--tests must include #[cfg(test)] fns; got " + repr(sorted(with_tests)))

    for f in failures:
        print("FAIL " + f, file=sys.stderr)
    n = len(SELFTEST_CASES) + 1
    if failures:
        print("fn-census selftest: " + str(len(failures)) + " failure(s) across "
              + str(n) + " cases", file=sys.stderr)
        return 1
    print("fn-census selftest: " + str(n) + "/" + str(n) + " cases hold")
    return 0


def main():
    ap = argparse.ArgumentParser(description="Measure Rust functions for the exemplar hunt.")
    ap.add_argument("path", nargs="?", help="a .rs file or a directory to walk")
    ap.add_argument("--selftest", action="store_true",
                    help="verify the measurer against the incidents that produced it, and exit")
    ap.add_argument("--name", help="only this function")
    ap.add_argument("--top", type=int, help="show only the N largest")
    ap.add_argument("--min-lines", type=int, default=0)
    ap.add_argument("--tests", action="store_true", help="include #[cfg(test)] modules")
    ap.add_argument("--format", choices=["txt", "md"], default="txt")
    a = ap.parse_args()
    if a.selftest:
        return run_selftest()
    if not a.path:
        ap.error("a path is required unless --selftest is given")

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
