# EXPECTATIONS — the rpds rebuild-loop sweep

Written **before** the strike so the result cannot move the goalposts. Every command below is the
orchestrator's own re-run; nothing here is graded on the rider's report.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | every self-reassignment site is converted | the sound detector re-run (below) | **0** remaining |
| 2 | nothing else was touched | `git diff --stat` | **6 files**, ~35 insertions / ~35 deletions, balanced |
| 3 | the nested inner call survived | `sed -n '1555,1562p' src/rete/kernel.rs` | outer `pm.insert_mut(`, inner still `pv.push_back(` |
| 4 | it compiles on the pinned toolchain | `cargo clippy --release --workspace --all-targets -- -D warnings` | exit 0, **0** warnings |
| 5 | semantics unchanged — the real gate | `cargo nextest run --release` | **4231/4231**, exit 0 |
| 6 | the differential still holds | included in row 5 (native == oracle tests are in the floor) | green |
| 7 | the fire does not regress | `fanout_per_call_alpha_census`, 3 runs | `THE FIRE` ≤ the 85.82 ms mean at HEAD |

Row 1's command:

```bash
python3 - <<'PY'
import re, pathlib
pat = re.compile(r'^\s*([A-Za-z_]\w*)\s*=\s*\1\s*\.\s*(push_back|push_front|insert|remove|set|drop_last|drop_first|push)\s*\(')
n = 0
for p in sorted(pathlib.Path('.').glob('src/**/*.rs')):
    if 'target' in p.parts: continue
    for i, line in enumerate(p.read_text().splitlines(), 1):
        if pat.match(line):
            print(f"{p}:{i}  {line.strip()}"); n += 1
print("REMAINING:", n)
PY
```

## Independent prediction

**Runtime: 8–15 minutes.** 35 single-token edits across 6 files, every one of them listed verbatim
with its current text, and a landed exemplar to copy. There is no discovery burden and no
composition to work out. Time-box at **2x the upper bound = 30 minutes**.

**Perf: row 7 is a NO-HARM row, not a win row.** Only two of the 35 sites sit on a path this
workload exercises heavily (`kernel.rs:905`, `extend_token`'s bindings fold inside
`hj:catchup:probe`; and `:244`/`:303`/`:406`, the token/element materialisers, which run at freeze
on memories that are cleared). The exemplar already took the big one. Expect the fanout fire to move
somewhere between **0 and −4 ms** — inside or barely outside noise — and do **not** grade the strike
on it. A scorecard that demanded a win here would be demanding an unfalsifiable claim; the reason to
do the sweep is that the form is wrong and the wall cannot be armed while any site remains.

The one place a real gain is plausible is `hj:catchup:probe` (18.79 ms at HEAD, 40,000
`extend_token` calls x 2 bindings each). `insert_mut` there still copies **once** — the trie is
cloned from a live `tok`, so its refcount is legitimately 2 — but the *second* key then writes in
place instead of copying again. So at most one of two copies per call disappears. Measure it; do not
promise it.

## Trap doors named in advance

- **`kernel.rs:1558`.** The only nested instance. A by-method-name rewrite corrupts it. Row 3 exists
  solely to catch that, and it is checked by eye, not by grep.
- **A site that is not rpds.** Structurally impossible — std's `insert` returns `Option`/`()`, so a
  self-reassignment would not type-check, and the tree compiles today. But if row 4 fails naming a
  missing `_mut`, that is the case, and the fix is to revert that one site, not to invent a shim.
- **A `let mut` that becomes unnecessary.** Not expected (all these bindings are still mutated), but
  if clippy raises `unused_mut` in row 4, drop the `mut` at that binding.
- **The five `#[cfg(test)]` sites in `edn_shim.rs`.** Easy to dismiss as cosmetic and skip. Row 1
  fails if they are skipped, and so would the wall that follows.
- **Load on the box.** Row 7's numbers are only comparable within a session. Re-run the HEAD
  baseline in the same batch if the box is busy rather than comparing against the figure quoted here.

## What "done" means

Rows 1–6 all pass on the orchestrator's own re-run, with their own exit codes read, before anything
is committed. Row 7 is recorded, not graded. The sweep unblocks
`tests/lint/no_rpds_rebuild_loop.rs` armed at zero (task #41), which is what makes the class
unrepresentable rather than merely swept.
