# EXPECTATIONS — A1: the left-index latch

> Written BEFORE the strike. Graded by the orchestrator's own re-run.

| # | what | command | expected |
|---|---|---|---|
| 1 ★ | the wrong answer is GONE | the landed gate | `native == oracle`, both `OutW=2` |
| 2 ★ | non-vacuity still holds | same | `the_control_reaches_a_second_round` PASSES (`C=2`, `OutP=2`) — the fixture still reaches round 2 |
| 3 ★ | the bypass is UNREPRESENTABLE | re-introduce a key-write that skips the left index | **compile error**, not a test failure — quote it |
| 4 ★ | D2 did not reopen | `-E 'test(right_index_counter)' + test(d2)'` and the rete suite | green; `right_idx` mark still tracks its population |
| 5 | floor | `./scripts/floor.sh`, read the **Summary** line | **5427+N run, 0 failed** |
| 6 | clippy | `cargo clippy --all-targets --release` | rc=0 |
| 7 | blast radius | `git diff --stat` | `src/rete/kernel/` + one test pair only |
| 8 | perf unmoved | if a `*_cost` gate reddens, report the ratio it asserts | no cost gate reddens, or the delta is explained |

★ = load-bearing. **Row 5 is the deliverable.** A red floor ends nothing.

## Runtime prediction

45–75 min. The cure is a type change threaded through 3–4 call sites; row 3 is the fiddly part.

## Trap doors, named in advance

- **Reopening D2.** `sequi` L2-a: the catch-up's right walk pushes the WHOLE alpha memory, and what
  stops it double-counting is that `keyed_join_persistent` sets the key gating `first_keying`.
  Removing the conflation without preserving that protection is the predicted failure. STOP-1.
- **`is_keyed` is a mark, and a mark can drift from what it marks.** That is the very class being
  cured. Prefer deriving it from the buckets over storing a third field.
- **The `.wat` fixture lands in `tests/rete/`, which no in-process loader walks.** Do NOT put it
  under `docs/arc/` — that tree IS walked (`every_docs_wat_loads_or_declares_why_not`).
- **Two gates fired unexpectedly in the last strike** (`no_inlined_wat_in_tests`,
  `no_loose_string_assert`). Re-run the floor at FINAL state, not before the last edit.

## What would make me reject the result

- A green gate that greens because the fixture changed rather than the engine.
- `first_keying` patched in place.
- Row 3 unattempted, or answered with a test failure where a compile error was required.
- A red floor of any size.
