# EXPECTATIONS — STONE D · `join` widens to `Seqable`

> Written BEFORE the strike so the result cannot move the goalposts. Every row is re-run by the
> orchestrator independently; the rider's report is a hypothesis until a command of mine agrees.

## Baseline, taken this session before the rider left

| fact | measured |
|---|---|
| floor | `4924 tests run: 4924 passed, 19 skipped`, 81.9s, ARM.txt empty |
| clippy | 0 under `-D warnings` |
| HEAD | `8c14bb4a0`, tree clean |
| `join` over Vector | `"1-2-3"` — works, renders i64s (279.3) |
| `join` over Stream | **REFUSED AT CHECK TIME**: `parameter #2 expects (:wat::core::Vector :- [:?2046]); got (:wat::stream::Stream :- [:wat::core::i64])` |

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | Vector unchanged | `probe_stone_D` row 1 | `"1-2-3"` |
| 2 | **Stream accepted** | `probe_stone_D` row 2 | `"2-3-4"` — the load-bearing row |
| 3 | List accepted | `probe_stone_D` row 3 | joins; proves the whole surface set |
| 4 | **rendering survived** | `probe_stone_D` row 4 | non-string via Stream renders identically to Vector |
| 5 | the gap is really closed | `target/release/wat` on the design's own repro | prints, does not raise |
| 6 | eager path untouched | read the diff | `Value::Vec` arm byte-identical; no normaliser call on it |
| 7 | floor | `scripts/floor.sh` | **4924/4924** — a widening adds capability, so any DROP is a finding and any RISE must be explained test-by-test |
| 8 | clippy | `cargo clippy --release --all-targets --workspace -- -D warnings` | 0 |
| 9 | blast radius held | `git diff --stat` | `check.rs` + `string_ops.rs` + probe + fixture. **Nothing else.** |

Row 6 is scored by reading, not by a test. No test in this repo can fail because a Vector took a
slower path — which is exactly why it needs a human-read row.

## Independent prediction

**12–25 minutes.** Two edits in two files plus a four-row probe, with the walk shape available to
copy verbatim from `transform.rs:709-768`. The runtime half is ~15 lines. Wake-up at 2× = 50 min.

Confidence is high on the runtime half and **lower on the type half**: `params[1]` is consumed by
whatever generic-unification path `join` shares with its neighbours, and I have not traced what else
reads that scheme. Row 7 is where that surfaces.

## Trap doors, named before the strike

1. **A `Seqable` param may not unify the way a concrete `Vector` did.** `Seqable` is a *surface*, and
   satisfaction-based unification took three stones in arc 118 (118.B2c, B2d, B6) to work in a
   `defclause`. This is a `TypeScheme`, not a clause — a different path. If the checker accepts a
   Stream but rejects a Vector, this is the mechanism.
2. **The eager path silently going lazy.** Cheapest failure to ship and the hardest to see: green
   tests, correct output, quiet cost. Row 6 exists only for this.
3. **`render_str_total` skipped on the new arm.** Would produce a plausible `"a-b-c"` for strings and
   garbage or a raise for anything else. Row 4 exists only for this.
4. **The scheme's reach.** If `join`'s scheme is shared or pattern-matched elsewhere, widening it
   could move unrelated diagnostics. Row 7 catches it; STOP-3 in the brief governs it.
5. **A `HashSet` join is now expressible and ORDER-UNDEFINED.** `Seqable` is extended to Vector,
   PersistentVector, List, Stream — **not** HashSet (`wat/seq.wat:81-90`). If the widening
   accidentally admits HashSet, `join` acquires a nondeterministic output and that is a defect, not a
   feature. Not a scorecard row because it should be impossible; named so it is not a surprise.

## What would make me reject a green report

- Row 2 green and row 4 absent or hand-waved.
- Row 7 at anything but 4924 without a test-by-test account.
- A container `match` in `string_ops.rs` instead of a call to `seqable_value_to_stream`. That passes
  every test above and re-derives a classification with a documented quadratic trap in the List arm.
