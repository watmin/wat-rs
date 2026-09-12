# SCORE — A′: reserved type names resolve ONCE; everything else PER FILE

Branch: `merge/grok-rete` @ `f131ce6d4`. **Not committed.** Main untouched. Corpus not migrated.
Binary: `bootstrap/wat-main-a3218644d` (main `a3218644d`). `target/release/wat` not used.

```
CHECK_EXIT=0     bootstrap/wat-main-a3218644d --check wat-scripts/fixes/positional-ctor-to-map.wat
floor / clippy / full corpus / wat-scripts/fixes/ as input: not run
```

Row 0: no `wat-main` / `positional-ctor-to-map` process before or after any run. (Harness `wat --mcp` is not a codemod run.)

## The strike

`wat-scripts/fixes/positional-ctor-to-map.wat` only. B's STOP-1 assertion is gone (STOP-2).

`corpus-invariant-name?` is **not** `is_reserved_prefix`. It covers `:wat::` and `:rust::` only. Comment cites `RESERVED_PREFIXES` (`src/resolve/reserved.rs:14`) and why `:$bound::` is excluded (per-scope binder unforgeability). Honesty of the cache is `gate` at `src/resolve/registration.rs:165` (`Existing::Equivalent → NoOp` BEFORE `Reserved`; tests at `:317-319`).

```
PASS 1  unique :wat::/:rust:: epaths → fill-paths once (empty decls / stdlib)
PASS 2  every other name → fmap-for-src against THIS file's decls (OLD's local model)
```

Prints: `[positional-ctor] resolve corpus-invariant=N per-file=M` (M = sum of unique local epaths per file).

## Row 1 — differential

Preserved gate `bootstrap/pctm-gate/{orig,A,B}`. Copy orig → B, NEW, compare to A.

**20/20 BYTE-IDENTICAL.**

Collision pair in one worklist (`accum.wat` + `retract-multiplicity.wat`): NEW did **not** halt (`NEWPAIR_EXIT=0`). Both files BYTE-IDENTICAL to OLD. (`accum.wat` d=0 — already map-form readln; `retract-multiplicity.wat` d=5.)

## Row 2 — timed, same files both columns

All on `bootstrap/wat-main-a3218644d`. One run at a time.

| n | files | OLD s | NEW s | OLD s/file | NEW s/file |
|---|---|---|---|---|---|
| 1 | `retract-multiplicity.wat` | **66.37** | **44.11** | 66.37 | 44.11 |
| 5 | query+grep+bracket+2 grid | **325.38** | **164.83** | 65.08 | 32.97 |
| 20 | preserved gate changers | **1281.68** | **413.76** | 64.08 | 20.69 |

OLD is linear (~65 s/file). NEW 1→5→20 is sublinear (44 → 33 → 21 s/file): invariant names paid once.

20-file print: `corpus-invariant=331 per-file=342`. 5-file: `corpus-invariant=275 per-file=44`. 1-file: `corpus-invariant=55 per-file=15`.

NEW/OLD on the **same 20**: **3.1×**. Not 280× — locals still resolve per file, and 331 invariant names still each pay one eval.

OLD20 vs preserved A: 0 deltas (bootstrap OLD reproduces the gate oracle).

## Row 3 — split (tool's own count)

See print lines above. 20-file gate: 331 corpus-invariant unique epaths resolved once; 342 per-file local epath resolutions (sum).

## Row 4 — ReservedPrefix on the binary that runs

`/tmp/pctm-aprime/reserved-probe.wat` declares `(:wat::core::defrecord :wat::aprime::Nope …)`.

```
bootstrap/wat-main-a3218644d --check  →  #wat.macro/ReservedPrefix  name ":wat::aprime::Nope"  exit 1
```

Premise holds on `a3218644d`.

## Row 5 — `--check`

`CHECK_EXIT=0` on the A′ file, bootstrap binary.

## STOPs

| STOP | hit? |
|---|---|
| STOP-1 cache a local across files | no |
| STOP-2 halt on local shape collision | no — pair ran to completion |
| STOP-3 tools as input | no |
| STOP-4 new intrinsic | no — `string::starts-with?` is in main |
| STOP-5 output ≠ OLD | no — 20/20 and the pair |

## Not done (by brief)

Commit. Floor. Clippy. Full corpus. Push. Main. Rebuilding `target/release/wat`.
