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

---

## ⛔ RETRACTED, same day — the prescription above RE-PROPOSES AN EXPLICITLY REJECTED OPTION

The closing paragraph recommends comparing `:file` while **normalising `:line`/`:col`**. That is wrong,
and it was ruled on in arc 296 before I wrote it. From `tests/types/probe_arc293_W2b_enum_purity.rs`,
in the fixture's own comment:

> *"a pinned line that gets updated when it moves is in a constant state of correctness, while a
> DROPPED field is permanently blind … the span **DISCRIMINATES THE EMITTER** —
> `ImpureVariantFieldInPureEnum` can be raised from more than one call site in check.rs, and
> `rust_caller_span!()` says which. Drop it and this test goes green the moment a *different* code
> path starts raising the same error kind — that silent pass is exactly the coverage this pin buys.
> **KEEP PINNING THE SPAN. Do not re-propose dropping it.**"*

And `296/BRIEF-296-WaveB1-complete-the-26.md`: *"BUILDER RULED: staleness. Recapture, keep pinning the
span."* — with the cost measured rather than assumed: exactly **one** `.edn` golden in the tree pinned
a `src/*.rs` span at the time, so the churn surface was trivial.

**What I got wrong, and it is the whole error:** I read the churn as the cost and never asked what the
pin BUYS. It buys emitter discrimination — the one thing normalising the line would destroy — and my
"fix" would have converted a test that catches a wrong emitter into one that cannot. The red I called
noise is the field doing its job.

★ `[[feedback_a_rejected_option_returns_in_new_clothes]]`. A reject list is stored as PHRASINGS and a
proposal arrives in its own: arc 296 rejected *"drop the span"*, and I proposed *"normalise `:line`
while keeping `:file`"* — the same operation wearing different words, in a NOTE offering it to future
stones as the fix. **The check I skipped is one command:** before proposing that a carried field be
dropped or loosened, grep the tree for a prior ruling on that field.

**The standing rule, restated so this NOTE stops arguing against it:** an internal `src/*.rs` span that
moved is STALENESS. Recapture it and KEEP PINNING IT. What survives from the observation above is only
this — the recapture is mechanical and must be VERIFIED, not rebaselined blind: confirm the new line is
the same `rust_caller_span!()` call site in the same file, and that only its position moved.
