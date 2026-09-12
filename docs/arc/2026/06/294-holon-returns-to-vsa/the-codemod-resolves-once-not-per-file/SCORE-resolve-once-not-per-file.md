# SCORE — `positional-ctor-to-map` resolves ONCE, not per file

Branch: `merge/grok-rete` @ `a9bdbc052`. **Not committed.** Main untouched. Corpus not migrated.

```
CHECK_EXIT=0     ./target/release/wat --check wat-scripts/fixes/positional-ctor-to-map.wat
BUILD_EXIT=0     cargo build --release
floor / clippy / full corpus / wat-scripts/fixes/ as input: not run
```

`--check` was clean on the interpreter that ran the differential. `cargo build --release` then rebuilt `target/release/wat`; that new binary dies loading grok-rete `wat/gen.wat` (`:wat::core::string::interpolate` refused by the F5 defmacro purity gate). Adjacent merge FINDING, not this stone — we did not edit `wat/gen.wat`. All A/B numbers below are from the pre-rebuild binary.

## The strike

`wat-scripts/fixes/positional-ctor-to-map.wat` only.

PASS 1 walks every input path, unions unique epaths + unique-named decls. STOP-1 compares declaration **AST shape** (kinds + names), not source text — comments are not two answers. Identical demo types keep the first; different field lists halt with `STOP-1 <ep> <file-a> <file-b>`.

PASS 1 then `fill-paths` once. Union `eval-with-defs!` can go blind (neighbouring defservice/defmacro poison). A file-local enum the union cannot see is retried against **that declaring file's decls, once**, and frozen into the same map. That is not STOP-2 (no per-file cache through `rewrite-each`).

PASS 2 migrates every file against the frozen fmap + file-local `bind-kw-ctors` (unquote aliases only).

## Row 1 — differential, 20 files OLD actually changes

Worked on copies under `/tmp/pctm/gate/{orig,A,B}`. Repo files untouched. `wat-scripts/fixes/` never in the worklist (STOP-3).

| # | path | A-orig bytes |
|---|---|---|
| 1 | `wat/query.wat` | 146 |
| 2 | `wat/grep.wat` | 23 |
| 3 | `wat/bracket.wat` | 402 |
| 4–14 | `wat-scripts/perf/grid/{accum-lead-rule-cascade,retract-multiplicity,accum-lead-derived,userfn-head,min-finding,retract-accum-derived,leading-neg-consumer,accum-over-derived,userfn-accum-derived,parametric-erasure,retract-lead-accum}.wat` | 5 each (`ReadlnOutcome::Datum __datum` → `{:v __datum}`) |
| 15 | `wat-scripts/scratch-pad/probe-sift-body-direct.wat` | 85 |
| 16 | `wat-scripts/scratch-pad/census-join-scope-where.wat` | 23 |
| 17 | `wat-scripts/scratch-pad/probe-insert-all-cost.wat` | 5 |
| 18 | `wat-scripts/scratch-pad/probe-seed-insert-vs-insert-all.wat` | 5 |
| 19 | `wat-scripts/scratch-pad/arc278-fence-binder-shadow/census-fence-binders.wat` | 18 |
| 20 | `wat-tests/gen-patterns.wat` | 39 |

**A vs B: 20/20 BYTE-IDENTICAL.**

Honest scarcity: after the grok-rete merge, OLD does not rewrite `FireOutcome`/`CompileOutcome`/`InsertOutcome` (UNRESOLVED). Outside `wat-scripts/fixes/` (STOP-3), these 20 are essentially every remaining positional site OLD still touches. `accum.wat`'s `:grid::Result` has extra fields (`insert-ns` …) — a real two-answer STOP-1 if mixed with the 6-field grid records; it was kept out of the gate.

## Row 2 — timed, same 20

| version | n | unique-eps | elapsed | s/file | exit |
|---|---|---|---|---|---|
| OLD (pre-rebuild binary) | 20 mixed: 15 of the gate changers + 5 no-ops | (per file, discarded) | **1122.68s** | 56.13 | 0 |
| NEW (pre-rebuild binary) | 20 gate changers | **617 once** | **422.41s** | 21.12 | 0 |

NEW/OLD on this dense 20 ≈ **2.7×**. Not the 280× of `variant-separator-to-dot`: 617 unique epaths still each pay `eval-with-defs!`, once. The stone is the shape — O(unique eps), not O(files × eps). On 2,176 files OLD stays ~15–56 s/file (~9 h); NEW's resolve pass grows with the union of names, not with file count.

A single OLD rerun of the final 20 was started after the smoke rebuild and died in 0.02s on `wat/gen.wat` purity (see above). Additional changer batches on the pre-rebuild binary: 8-file extras 318.49s (3 changers), 2-file 42.01s (census-fence), `gen-patterns.wat` 90.81s.

## STOPs

| STOP | hit? | what |
|---|---|---|
| STOP-1 | false-positive then fixed | `:grid::Result` in `accum-lead-rule-cascade.wat` vs `retract-multiplicity.wat` — same six fields, one file has a THREE-WAY comment. Compared source text → halted. Now AST shape. Real two-answer case exists: `accum.wat` adds four timing fields. Named, not picked. |
| STOP-2 | no | one frozen fmap; file-local retry is declaring-file decls, not a rewrite-each cache |
| STOP-3 | no | tools never in the worklist |
| STOP-4 | hit then fixed | `wat-tests/gen-patterns.wat`: OLD rewrote `:wat-tests::pat::Cmd::{Put,Del}`; union eval left them UNRESOLVED (B-orig 0). Declaring-file fallback → byte-identical. |

## Session crash

WSL kernel dump 15:35:05 + `CheckConnection: getaddrinfo() failed`. First `/tmp/pctm` wiped; implementation survived uncommitted. Restarted OLD20 completed 1122.68s. No wat process left running after the gate.

## Not done (by brief)

Commit. Floor. Clippy. Full corpus. Push. Main. Migrating any repo `.wat`.
