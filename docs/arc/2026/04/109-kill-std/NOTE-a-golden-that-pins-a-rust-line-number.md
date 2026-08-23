# NOTE — a golden that pins a Rust source LINE NUMBER breaks on unrelated edits

**Found 2026-08-23**, in `STONE-one-name-grammar`. Not a defect the stone introduced — a property of
the fixtures it tripped over, twice, in one flight.

## What happened

The rider's edits changed line counts in `src/check.rs` (net +6) and `src/runtime.rs` (net −10). Seven
golden `.edn` fixtures went red — not because any behaviour changed, but because they pin the
`crate::rust_caller_span!()` coordinate of the code that produced them:

```
probe_arc293_W2b_enum_purity::pure_enum_with_struct_field_rejected
  :location :line   golden 13892   actual 13898
probe_diagnostic_value_snapshot_in_errors  (5 goldens)
  src/runtime.rs:25681 → 25671
```

Zero semantic drift. The rider diagnosed it by static read rather than a blind re-run, fixed all
seven, and swept every other touched file for the same class.

## Why it is worth a NOTE

**A golden that pins a Rust line number measures the file's shape, not the program's behaviour.** It
goes red for an edit six functions away and stays green for a real change that happens not to move a
line. Both halves are wrong, and the red half is the expensive one: it trains a reader to treat
golden failures as noise to be re-baselined, which is exactly how a real regression gets waved past.

This is the day's recurring shape once more — an instrument reporting something other than what it
claims to measure. Kin: `[[feedback_a_probe_that_recalibrates_under_load_measures_nothing]]`,
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`.

## Scope

Out of `STONE-one-name-grammar`'s scope — that stone fixed the seven and moved on, which was correct.
Not tracked elsewhere; this NOTE is the record.

The fix has a known shape, and the substrate already owns half of it: a diagnostic's location is
carried structurally (`#wat.core/Span {:file :line :col}`), so a golden CAN compare the `:file` while
normalising `:line`/`:col` — asserting *that* a location was captured and *from which file*, without
pinning where in the file the emitting code currently sits. Any stone that touches these fixtures
should take that shape rather than re-baselining the numbers again.
