## PURGARE — Cast Report (TARGET 3 — the grid)

> Written verbatim as returned, before any synthesis. Evidence only — this file carries NO status.

## SCOPE

Swept `wat-scripts/perf/grid/` (147 files: 54 `.wat`, 43 `.clj`, 29 `.txt`, 20 `.sh`, 2 `.md`). Read every one of the 20 `.sh` in full or by targeted section; read `tests/lint/every_parity_script_is_invoked.rs` in full; grep-diffed the axis corpus independently rather than trusting the handed-down claim.

**Query kinds used to establish liveness** (per the ward's own warning that one query type misses consumers here):
1. **By name** — against `.github/workflows/ci.yml` and `tests/**/*.rs`.
2. **By discovery-walk** — read the actual glob/reconciliation code in `run-all.sh`, `check-grid-three-way.sh`, `check-where-shapes.sh`, cross-checked bidirectionally (MISSING/STALE both directions).
3. **By call-site inside sibling scripts** — built the actual invocation chain: `check-grid-speed.sh` → `run-all.sh` → `run-axis.sh` → `gen-<axis>.sh`; `check-grid-three-way.sh` → `gen-<axis>.sh` directly.
4. **By function name within a file** — every `fn_name() {` cross-checked against its call sites (all matched, none orphaned).
5. **By independent glob-diff** — `comm` between `where-*.wat` and `where-*.clj` stems (zero orphans either direction), plus a fresh per-stem check that each of the 16 non-`where-*` axes has *exactly one* of `gen-<stem>.sh` / `<stem>.clj` (zero BOTH, zero NEITHER) — re-confirming the exactly-one-of claim from scratch.
6. **Mutation-style verification** — reproduced the bug below with a standalone script under identical `set -euo pipefail` semantics before reporting it.

**Coverage sweep result:** every `.sh` outside the `check-*.sh` gate (the 11 `gen-*.sh`, `run-axis.sh`, `run-all.sh`, `compare-grids.sh`, `peragrare-census.sh`) is **alive** — transitively invoked from CI, or via direct discovery, or (for `compare-grids.sh`) a documented human-facing entry point whose own header states the gap it fills. **All 54 `.wat` axes are covered by at least one instrument.** No function, variable, env var or flag found with zero readers. No `rune:purgare` anywhere.

## FINDING — `run-all.sh:138-139`

```bash
if ! bash "$GRID_DIR/run-axis.sh" "$axis" "${rungs[@]}"; then
  echo "run-all: axis '$axis' FAILED (rc=$?) — see its stderr above" >&2
  rc=1
fi
```

**Evidence of death:** `$?` here is **not** the exit status of `run-axis.sh` — it is the status of the negated `if !` test, which bash always collapses to `0` inside the `then` branch. Reproduced under the script's own `set -euo pipefail`:

```
f() { return 3; }
if ! f; then echo "FAILED (rc=$?)"; fi     →  FAILED (rc=0)
```

always, regardless of the real failure code. The message promises the failing axis's exit code, but the value printed is effectively a constant `0` that can never vary — **a read permanently disconnected from any producer**, the same shape purgare's "parameters always passed as a constant" / "branches that always evaluate one way" categories describe, applied to a diagnostic value. ⚠ **The sweep's own `rc=1` / final `exit $rc` is unaffected and correct** — only this one diagnostic field is dead.

**Recommendation:** capture the real code before the `!` consumes it —
```bash
if bash "$GRID_DIR/run-axis.sh" "$axis" "${rungs[@]}"; then :; else
  axis_rc=$?; echo "run-all: axis '$axis' FAILED (rc=$axis_rc) …" >&2; rc=1
fi
```
**Cost:** leaf — one line, no cascade; nothing parses this message downstream.

**FINDINGS — 1.**
