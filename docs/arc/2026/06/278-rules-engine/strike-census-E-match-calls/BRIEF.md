# BRIEF — census E: make `match:calls` a call count

Read `DESIGN.md` first. **One statement moves two lines up**, two comments become true, and a probe
tells us something the tree does not currently know. No behaviour change — `census_count` is a
release no-op.

## Read in order

1. `src/rete/matcher.rs:536-560` — `alpha_match_inner_opts`. Line 544 is `let pat =
   alpha_pattern(cond)?;` and 545 is the bump. The `?` is why the name is false. Move the bump
   above the `let`. Nothing else in this function changes.
2. `src/rete/matcher.rs:528-535` — the doc claiming `compiled_cond.rs` parallels this counter.
   After the move it is TRUE; say so plainly and note what the parallel is FOR (an
   interpreter-vs-compiled call differential), so the next reader knows why the position matters.
3. `src/rete/compiled_cond.rs:946-961` — the other half of the claim. `census_count("compiled:exec")`
   is the first statement, before any early exit; that is the unit `match:calls` now matches. This
   comment was written by census B and restated the false parallel — correct it to record that
   both counters bump before their guards, deliberately.
4. `src/rete/kernel/tests/fanout_cost.rs:210-240` — the ONLY consumer. Its row is labelled
   *"match:calls (interpreter entries — expect 0)"* and it asserts nothing on this key directly;
   `prod:derivations == 40_000` is its non-vacuity guard. Read it before running the probe so you
   can say what a nonzero reading would mean.
5. `src/rete/kernel/tests/alpha_discrimination.rs:396-410` — read only to confirm `interp_calls` is
   a hand-counted local (`interp_calls += 1`), NOT this census. It must stay that way.

## The probe — run it before and after, and report both

```
cargo nextest run --release --no-capture -E 'test(fanout_rhs_allocation_census)'   # or the test's real name
```

Record `match:calls` **before** the move and **after**. Expected: 0 → 0. A nonzero after-value is
STOP-1: report the count and identify what entered `alpha_match_inner_opts` with a non-alpha
`cond`. That would be a live interpreter entry on a world documented as having none — a finding
this strike exists to surface, and it must not be re-pinned away.

## Blast radius

`src/rete/matcher.rs` (one statement moved, one doc) · `src/rete/compiled_cond.rs` (one comment) ·
`src/rete/kernel/tests/fanout_cost.rs` (its label may need a word if the reading changes).
**No new counter, no rename, no signature change, no `.wat`.**

## Mutation proof

The move itself is the proof, and it is a real one this time:

- **Before/after the move**, on the same world, report `match:calls`. If the counter's population
  genuinely widened, some world in the suite should show it — say which, or say plainly that none
  does and that the widening is therefore unobserved.
- **Delete the bump** → `fanout_cost.rs`'s printed row loses its value while
  `prod:derivations` holds. Quote it. (The consumer does not assert on this key, so this shows the
  counter's reach honestly rather than claiming a red it cannot produce — the same shape as census D.)

## STOP triggers

1. `match:calls` becomes nonzero anywhere → STOP, report the count and the entering call. Do not
   re-pin.
2. Anything other than a census count moves → STOP.
3. The move needs more than relocating one statement → STOP; the DESIGN misread the function.

## Prior result to copy for shape

`../strike-census-D-key-alloc/SCORE.md` — starred rows, raw evidence quoted, and an honest
statement where a mutation proof was not available instead of a manufactured one.
