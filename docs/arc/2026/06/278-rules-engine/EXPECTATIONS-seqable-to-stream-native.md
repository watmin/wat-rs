# EXPECTATIONS — Strike 1: `seqable->stream` goes native

Written BEFORE the strike so the result cannot move the goalposts.
Brief: `BRIEF-seqable-to-stream-native.md`. Design: `DESIGN-STONE-seq-traversal-one-door.md`.

## Scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | builds | `cargo build --release --all-targets` | exit 0, zero warnings |
| 2 | the gate is REAL | the new gate, run **before** the fix | **FAILS** (~12s vs a 1s wall) |
| 3 | **the load-bearing row** — the quadratic is gone | the new gate, after | PASSES (~10ms, 100× margin) |
| 4 | the door is SHARED | `keep` / `dedupe` / `distinct` go linear **with zero edits of their own** | linear |
| 5 | laziness survives | early-exit still short-circuits (`take` over a long/lazy source) | no full realisation |
| 6 | results unchanged | every delegating verb returns the same elements, same order | identical |
| 7 | stdlib load order | `(:wat::deporder::verify-stdlib)` | `[]` |
| 8 | the floor | `cargo nextest run --release` (orchestrator, central) | no test's result changes but the new gate |

**Row 2 is not a formality.** A gate that never went red is a gate that proves nothing — R59's
lesson, learned on this arc's own floor. If it passes before the fix, the strike stops there.

**Row 5 is the trap-door.** The whole point of `seqable->stream` is laziness. A "fix" that
materialises the sequence eagerly would make rows 3, 4, 6 and 8 all green while silently destroying
the property the function exists for. It must stay lazy.

Row 8 is mine, not the rider's — riders get build-only plus one narrow filtered gate.

## Independent prediction

**Runtime: 30–50 minutes.** Longer than the edge fix: this is fresh Rust against the `Stream`
enum, not a three-line guard with an exemplar two hundred lines away. Predicted mode: one-shot
green, with STOP-1 the realistic risk if `NativeThunk` cannot hold the container handle cheaply.

Confidence is moderate, not high. The mechanism is measured and the dispatch shape has a working
reference (`eval_vec_foldl`), but the lazy-cell construction is the part I have **not** read in
detail, and that is exactly where I have been wrong repeatedly on this thread.

## Trap-doors named in advance

- **The `List` arm silently staying quadratic.** `Arc<LinkedList>` has no indexed access. If the
  rider indexes it per step, everything looks fixed and that arm is still O(n²) — the exact silent
  divergence this stone exists to kill, reproduced inside the fix. The design's four questions turn
  on this; the brief calls it out; row 4 should be checked on a `List` too, not only a Vector.
- **Eager materialisation dressed as a fix** — see row 5.
- **A gate that measures the wrong verb.** It must exercise a verb that *delegates* through the
  normaliser and that the rider is **not** editing (`keep` is the clean choice). Testing `filter`
  here would prove nothing: `filter` hand-rolls its own walk and is Strike 2's problem.
- **Clippy under the deny wall.** New Rust in `src/` must be warning-clean; the workspace lints are
  `deny`, so a warning is a build failure, not a note.

## What this does NOT claim

Strike 1 fixes the **converter**, so the six delegating verbs go linear. The seven hand-rollers —
`filter`, `remove`, `take-while`, `drop-while`, `interpose`, `reductions` — **stay quadratic** until
Strike 2 migrates them onto the door. In particular `query-by-type-string`, the function that
started this whole chase, calls `filter`, so **the A8 derive quadratic is still there after this
strike**. That is expected and correct sequencing: prove the door works, then move everyone through
it. Do not read a still-slow node-share axis as a failed Strike 1.
