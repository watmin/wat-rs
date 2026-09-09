# EXPECTATIONS — `seen-ids` stops cloning

Written **before** the strike. `wat-scripts/queue/sqs.wat` only.

## Rows

| # | what must be true | how it is checked | expected |
|---|---|---|---|
| 1 | ★ **no cloning verb remains on the `seen-ids` path** | `grep` the file | zero `:wat::hashset::` on any `seen-ids` line; the type is `PersistentSet`, the insert is `:wat::set::conj` |
| 2 | ★ **the instrument does not change the cost of what it measures** | n=2000 `vis-ms=1000`, quiet box | **wall clock and every phase time** within run-to-run variance of the 41 s / fill≈16–17 s baseline. Not store-calls — see below |
| 3 | ★ **`redeliveries` still reports the same events** | the same run | inbox `redeliveries=0`; **at least one subscriber tier nonzero** (20 on both graded runs). A counter that reads 0 everywhere has been broken, not optimised |
| 4 | **the other four counters are untouched** | the per-tier line | `accepted`, `refused`, `acks`, `expired-waiters` behave as before; inbox `refused` still equals the publisher's `full-retries` |
| 5 | **correctness holds** | the same run | `distinct=8000`, `dup=0` |
| 6 | **blast radius** | `git diff --stat` | `wat-scripts/queue/sqs.wat` **only** |
| 7 | **the corpus loads** | `every_wat_scripts_file_loads` | PASS |
| 8 | **the floor holds** | `scripts/floor.sh` | **read the Summary line**: 5237 passed, 22 skipped, **0 FAIL, 0 TIMEOUT** |

## The rows that carry it

★★ **Row 2 is this stone's whole reason, and it is written to correct my own mistake.** On the stone that
introduced `seen-ids` I wrote the row as *"the instrument costs no store calls"* — which the executor
satisfied honestly at 9758 vs 9731 while a quadratic term walked straight through, because
clone-the-whole-set costs **allocation, not store calls.** I gated the axis I predicted and left the one
that applies wide open.

So row 2 states the **property** — *the instrument does not change the cost of what it measures* — and
checks it against **wall clock and phase times**, the axes a clone actually moves. Report the numbers
beside the baseline, not a verdict.

★ **Row 3 is the anti-vacuity guard.** The cheapest way to make an expensive counter cheap is to stop
counting. `redeliveries` must still fire where it fired before: **0 at the inbox, nonzero at a subscriber
tier.** If every tier reads 0, the counter is broken and row 3 fails even though row 2 passes.

⚠ **Row 1 is greppable on purpose.** *"Use the persistent set"* is an instruction; *"no `:wat::hashset::`
on a `seen-ids` line"* is a property I can verify without trusting a report.

## Runtime prediction

**20–35 minutes.** Three real edits (declaration, initial value, `conj`) — the other 28 `seen-ids`
mentions are State-constructor threading and do not change. One n=2000 run at ~41 s. The floor is the
long pole.

## Trap-doors

- **`:wat::set::conj` returns a NEW set.** Thread the result exactly as `seen-ids` is threaded today; do
  not reach for `insert_mut` semantics that the wat surface does not expose.
- **The constructor is `(:wat::core::PersistentSet :- [:wat::core::String])`**, mirroring the `HashSet`
  form it replaces.
- **`:wat::set::` has five verbs only** — `conj disj contains? empty? length`. The membership test is
  `:wat::set::contains?`.
- **28 threading sites must keep threading.** A dropped `:seen-ids` field is a type error, so the
  compiler catches it; a *wrong* one (threading the pre-insert set) is not caught and would silently
  under-count. Check the `conj` result is what flows forward.
- **Do not touch the other four counters, admission, visibility, or the cap.**

## What this stone does NOT claim

⚠ It does **not** make the set bounded. Memory stays O(N) — 8000 retained ids — which is honest: the
store already holds O(N) rows.
⚠ It does **not** touch `circuit.wat`'s six cloning sites or anything under `wat/` (the census lists
them; `wat/` is the builder's call).
⚠ It does **not** change `StoredRow`, `Envelope`, or the body wire shape.
