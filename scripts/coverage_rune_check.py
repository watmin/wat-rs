#!/usr/bin/env python3
"""coverage_rune_check.py — the rune-aware coverage gate analyzer (arc 252).

The piece cargo-llvm-cov can't do: it reports uncovered regions but knows nothing
of our runes. This reads an LCOV file + the src tree and, for each WARDED-HOME
source file, finds uncovered line-blocks NOT exempted by a
`// rune:coverage(<category>) — <reason>` on (or just above) the block.

Doctrine (docs/COVERAGE-RUNE.md): 100%-minus-argued-runes per warded file. The gate
passes iff every uncovered region in a warded home is either covered or runed.
Uncovered-AND-not-runed is a finding: test it, or rune it with a reason. (excusare
weighs each rune for legitimacy; this gate only enforces presence.)

Rune forms (both DRIFT-FREE — no line numbers, per feedback_mark_the_source_not_memory):
  // rune:coverage(<cat>) — <reason>            exempts the NEXT uncovered block
  // rune:coverage(<cat>) [cluster] — <reason>  exempts the RUN of uncovered blocks
                                                until coverage resumes (one attestation
                                                for a homogeneous arm-cluster, e.g. the
                                                is_atomizable-guaranteed unreachable! arms)

Scope: ONLY the warded homes. The flat monolith (runtime.rs / check.rs / unwarded
src/*.rs) is out of scope until migrated — it carries no vigilatum stamp to defend.

Usage:
  coverage_rune_check.py <lcov> [--homes a,b,c] [--blocks]
Exit 0 if every uncovered region in a warded home is runed; 1 otherwise.
"""
import sys
import re
import os

WARDED = "value function check types collection macros scope comms remedy argspec rust_deps".split()
RUNE_RE = re.compile(r"//\s*rune:coverage\(([a-z/]+)\)(\s*\[cluster\])?")
LOOKBACK = 3  # lines above a block within which a heading rune still counts


def parse_lcov(path):
    files, cur = {}, None
    with open(path) as fh:
        for ln in fh:
            if ln.startswith("SF:"):
                cur = ln[3:].strip()
                files[cur] = {}
            elif ln.startswith("DA:") and cur is not None:
                n, c = ln[3:].strip().split(",")[:2]
                files[cur][int(n)] = int(c)
            elif ln.startswith("end_of_record"):
                cur = None
    return files


def is_warded(path, homes):
    return any(f"/src/{h}/" in path for h in homes)


def coalesce(lines):
    """Group sorted uncovered line numbers into blocks, bridging gaps <= 2
    (a comment/blank/brace line between two uncovered statements)."""
    blocks = []
    for n in lines:
        if blocks and n - blocks[-1][1] <= 2:
            blocks[-1][1] = n
        else:
            blocks.append([n, n])
    return blocks


def runes_in(srclines):
    """Return list of (category, lineno, is_cluster)."""
    out = []
    for i, line in enumerate(srclines, 1):
        m = RUNE_RE.search(line)
        if m:
            out.append((m.group(1), i, bool(m.group(2))))
    return out


def main():
    if len(sys.argv) < 2:
        print("usage: coverage_rune_check.py <lcov> [--homes a,b,c] [--blocks]", file=sys.stderr)
        sys.exit(2)
    lcov = sys.argv[1]
    homes = WARDED
    show_blocks = "--blocks" in sys.argv
    if "--homes" in sys.argv:
        homes = sys.argv[sys.argv.index("--homes") + 1].split(",")

    files = parse_lcov(lcov)
    per_file = []
    runed_cats = {}
    grand_open_lines = grand_open_blocks = grand_runed_blocks = 0

    for path, da in sorted(files.items()):
        if not is_warded(path, homes) or not os.path.exists(path):
            continue
        srclines = open(path, encoding="utf-8", errors="replace").read().splitlines()
        runes = runes_in(srclines)
        covered = sorted(n for n, c in da.items() if c > 0)
        uncovered = sorted(n for n, c in da.items() if c == 0)
        if not uncovered:
            continue

        def next_covered_after(line):
            for c in covered:
                if c > line:
                    return c
            return float("inf")

        open_lines = open_blocks = runed_blocks = 0
        detail = []
        for s, e in coalesce(uncovered):
            cat = None
            for rc, rl, cluster in runes:
                if cluster:
                    # exempt the run of uncovered blocks below the rune until coverage resumes
                    if rl < s and e < next_covered_after(rl):
                        cat = rc
                        break
                elif s - LOOKBACK <= rl <= e:  # heading rune: on/just-above this block
                    cat = rc
                    break
            if cat:
                runed_blocks += 1
                runed_cats[cat] = runed_cats.get(cat, 0) + 1
            else:
                open_blocks += 1
                open_lines += (e - s + 1)
                detail.append((s, e))
        if open_blocks or runed_blocks:
            rel = "src/" + path.split("/src/", 1)[-1]
            per_file.append((rel, open_lines, open_blocks, runed_blocks, detail))
            grand_open_lines += open_lines
            grand_open_blocks += open_blocks
            grand_runed_blocks += runed_blocks

    per_file.sort(key=lambda r: -r[1])
    print("=== warded-home coverage gate — uncovered-not-runed (100%-or-runed) ===")
    for rel, ol, ob, rb, detail in per_file:
        flag = "OK " if ol == 0 else "GAP"
        print(f"  [{flag}] {rel:<34} uncovered-not-runed: {ol:>4} lines / {ob} block(s); runed: {rb}")
        if show_blocks and detail:
            for s, e in detail:
                print(f"          {rel}:{s}-{e}")
    print(
        f"\nTOTAL: {grand_open_lines} uncovered-not-runed lines in {grand_open_blocks} block(s) "
        f"across {sum(1 for r in per_file if r[1])} warded file(s); "
        f"{grand_runed_blocks} runed exemption(s) {runed_cats or ''}"
    )
    if grand_open_blocks:
        print("GATE: FAIL — test or rune each uncovered block (see docs/COVERAGE-RUNE.md).")
        sys.exit(1)
    print("GATE: PASS — every uncovered region in a warded home is runed.")
    sys.exit(0)


if __name__ == "__main__":
    main()
