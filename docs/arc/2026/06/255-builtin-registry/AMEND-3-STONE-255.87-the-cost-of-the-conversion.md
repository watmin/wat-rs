# AMEND 3 — STONE 255.87: the conversion must not cost time

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `995dcda3f`
(group B cured; `.floor/2026-10-03T09-00-08Z`: 6404 passed, 1 failed, 3 timed out, all time limits). Commit locally on
`main`; **do not push**.

## Where it stands

Amendment 2 is accepted as measured. The only reds are time limits: `rete::reachability` shards (30 s), the rete fuzz
deftest (90 s), `retirement_table_is_fully_reachable` (240 s). They are the cost the converted stdlib added, measured in
amendment 1: **1.47×** (reachability shard 2, 9.707 → 14.312 s) and **1.48×** (`keyed_gather`, 7.647 → 11.330 s), six
runs each; the floor clock went 394 → 580 s. The builder's goal for this cutover is a **performance upgrade**: a
conversion that slows the system down is not finished. **No time limit is raised** (T1, and the 255.86 ruling).

## The work

1. **The comparison you did not have:** the same per-phase startup table (amendment 1's temporary timers) for the
   **unconverted** stdlib (`wat/` at `e08fe7349`, the same Rust), beside the converted one, on the same workload, so
   the delta is attributed by phase, not inferred. Amendment 1 put **6.9 s of 13.9 s** in the span from
   `record("4-register-defmacros")` (`src/freeze/env.rs:423`) to `record("4-expand-all")`: `register_defmacros`,
   `register_aggregate_kwargs_companions`, `seed_declared_type_names` (twice), `preregister_acronyms`, `expand_all`.
   Report which calls grew, by how much.
2. **Find the mechanism** inside the phase that grew: a symbol head or name canonicalized again on every lookup (string
   allocation per `canonical_identity` call), a match that falls back to a slow path for a symbol, a walk repeated per
   macro, a cache keyed by spelling that misses. Prove it with a counter (calls, allocations), not only a clock.
3. **Cure it at the door** (canonicalize once where a name enters, as K1 did for types; or key the lookup by canonical
   identity once), so the converted stdlib's startup is **no slower than the unconverted one**: the same six-run table
   after the cure, both columns. If the converted stdlib can be made faster, say by how much and why.
4. **Then the floor**, with no limit changed: the time-limit rows must pass on the default limits. Clippy; `git status`
   clean; then census `--diff` and `delta.sh` on the 179-file sample (the brief's item 5, which has not run yet).

## STOPs

- **STOP-1:** the slowdown is in a phase whose cure changes what a program means (e.g. dropping a check). Describe it and
  STOP.
- **STOP-2:** the converted stdlib cannot reach the unconverted startup time without a design change (e.g. sharing a
  frozen stdlib across worlds). Report the measurement and the design, and STOP: that is the builder's call.
- A STOP means STOP. Append to the SCORE, commit, **do not push**.
