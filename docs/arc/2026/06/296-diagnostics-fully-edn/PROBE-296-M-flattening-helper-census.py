#!/usr/bin/env python3
"""PROBE 296-M — the error-flattening-helper census. THE INSTRUMENT THAT PRODUCED THE 71.

A helper `fn f(..) -> Result<T, String>` whose body `map_err`s a typed error into a `format!`
DESTROYS THE DISCRIMINANT before any assertion can see it. Stone L's `assert_startup_error!`
cannot reach through one, so a negative test routed via such a helper has no honest fix — only
a bypass or a `rune:lint(bare-is-err)` exemption whose real cause is a signature.

Committed because a count whose instrument lives in a session tmp dir is a number nobody can
reproduce. `[[feedback_an_instrument_must_outlive_the_number_it_produced]]`

⚠ SCOPE, stated so it can be falsified: it scans `tests/**.rs` only, matches a fn whose return
type is textually `Result<_, String>`, and requires `map_err` plus `format!`/`to_string` within
the next 500 chars of its body. A helper that flattens FARTHER down its body than that, or that
returns a type alias for `Result<_, String>`, is invisible to it. Both were checked for by hand
at draw time and neither occurred.

Run from the repo root:
    python3 docs/arc/2026/06/296-diagnostics-fully-edn/PROBE-296-M-flattening-helper-census.py
"""
import os, re, sys
from collections import Counter

rows = []
for root, dirs, files in os.walk('tests'):
    dirs[:] = [d for d in dirs if d not in ('target', '.claude')]
    for f in files:
        if not f.endswith('.rs'):
            continue
        p = os.path.join(root, f)
        src = open(p, encoding='utf-8', errors='replace').read()
        for m in re.finditer(r'fn\s+(\w+)\s*\([^)]*\)\s*->\s*Result<([^,]*),\s*String\s*>', src):
            body = src[m.end():m.end() + 500]
            if 'map_err' not in body:
                continue
            if 'format!' not in body and 'to_string' not in body:
                continue
            line = src[:m.start()].count('\n') + 1
            rows.append((p, line, m.group(1), m.group(2).strip()))

want = sys.argv[1:] or None
sel = [r for r in rows if not want or any(r[0].startswith(w) for w in want)]

print(f"FLATTENING helpers: {len(sel)}" + (f"  (of {len(rows)} repo-wide)" if want else ""))
for d, n in Counter(os.path.dirname(p) for p, _, _, _ in sel).most_common():
    print(f"  {n:3}  {d}")
if sel:
    print("\nsites:")
    for p, line, name, ok in sorted(sel):
        print(f"  {p}:{line}  fn {name} -> Result<{ok}, String>")
