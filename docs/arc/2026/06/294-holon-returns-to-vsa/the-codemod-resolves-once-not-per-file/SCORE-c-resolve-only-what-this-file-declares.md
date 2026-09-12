# SCORE — (c): resolve a LOCAL enum path only if THIS FILE could declare it

Branch: `merge/grok-rete` @ `f35b00431`. **Not committed.** Main untouched. A′ (`bbcb70e38`) is the base.
Binary: `bootstrap/wat-main-a3218644d`. Corpus not migrated.

```
CHECK_EXIT=0     bootstrap/wat-main-a3218644d --check wat-scripts/fixes/positional-ctor-to-map.wat
floor / clippy / full corpus / wat-scripts/fixes/ as input: not run
```

Row 0: no `wat-main` / `positional-ctor-to-map` process before or after any run.

## The strike

`wat-scripts/fixes/positional-ctor-to-map.wat` only. Between `local-eps-of` and `fill-paths`:

- file has a macro-shaped decl (`defservice` / `sift-rules-defsvc` / `defmacro`) → keep ALL local paths (STOP-1)
- otherwise keep a path iff it equals, or is prefixed by on a `::` or `.` boundary, the name (child 1) of a plain type decl (`defenum` `defrecord` `defstruct` `defsurface` `newtype` `typealias` `typeunion`)

Filter in place (STOP-2). `ctor-edits` still the only candidate-site door. No syntactic kwargs classifier.

Prints: `[positional-ctor] resolve corpus-invariant=N per-file=M kept-local=K skipped-local=S`

`--audit` in the path vector: for every skipped path, `try-type-of` against this file's decls; Some → `STOP-6 skipped-that-resolved-to-Some <ep> <path>`.

## Row 1 — differential

| set | oracle | result |
|---|---|---|
| 20 gate changers | `bootstrap/pctm-gate/A` | **20/20 BYTE-IDENTICAL** |
| 5 sampled no-change files | HEAD content | **5/5 BYTE-IDENTICAL** (d=0) |
| `accum.wat` + `retract-multiplicity.wat` | OLD | **IDENTICAL, no halt** (`CPAIR_EXIT=0`) |

## Row 2 — audit (exactness)

`--audit` on the 20 gate files + 5 samples. `AUDIT25_EXIT=0` (no STOP-6).

```
local paths total = 434
kept              = 92
skipped           = 342
skipped-that-resolved-to-Some = 0
```

Print: `corpus-invariant=360 per-file=434 kept-local=92 skipped-local=342`

## Row 3 — counts (tool's own)

See print lines. Gate-only n=20: `corpus-invariant=331 per-file=342 kept-local=77 skipped-local=265`.

## Row 4 — timed, same files, A′ vs (c)

All `bootstrap/wat-main-a3218644d`. One run at a time. n=1 is `experiri-acc-head.wat`. n=5 is the five samples. n=20 is the gate.

| n | A′ s | (c) s | A′ s/file | (c) s/file |
|---|---|---|---|---|
| 1 | **26.39** | **22.35** | 26.39 | 22.35 |
| 5 | **143.77** | **57.75** | 28.75 | 11.55 |
| 20 | **420.20** | **197.75** | 21.01 | 9.89 |

n=5 samples: `kept-local=15 skipped-local=77`. A′/c ≈ **2.5×** on the files that change nothing. n=20 gate ≈ **2.1×** (query.wat keeps all locals because it has macro-shaped decls).

A′ n=20 vs (c) n=20 vs gate A: 0 deltas.

## Row 5 — `--check`

`CHECK_EXIT=0`.

## STOPs

| STOP | hit? |
|---|---|
| STOP-1 skip locals in a macro-decl file | no |
| STOP-2 reorder | no — foldl conj |
| STOP-3 tools as input | no |
| STOP-4 new intrinsic | no |
| STOP-5 output ≠ oracle | no |
| STOP-6 skipped path resolved Some | no — 0 |

## Not done (by brief)

Commit. Floor. Clippy. Full corpus. Push. Main.
