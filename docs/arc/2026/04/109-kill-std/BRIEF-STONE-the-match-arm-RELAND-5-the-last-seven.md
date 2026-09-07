# BRIEF — RELAND 5: the last seven, three causes, one of them not ours

> Floor **5219 passed / 7 failed** (108 → 38 → 16 → 7), clippy 0. All four probe rows PASS,
> `#[ignore]` 0. Each of the seven is diagnosed below; **none shares a root with another.**

## CAUSE 1 — GOLDEN SPAN DRIFT (`peers_bijection` ×4)

```
golden   :file "wat/service.wat" :line 881
actual   :file "wat/service.wat" :line 910
```

Everything else is byte-identical — same `ProgramBodyEvalFailed`, same `MalformedTemplate`, same
`:reason` text. The arm migration rewrote `wat/service.wat`'s clauses and moved its lines by ~29.

★ **The goldens pin a STDLIB SOURCE LINE**, which arc 109 already has a NOTE about in the Rust case:
`NOTE-a-golden-that-pins-a-rust-line-number.md`. This is the same defect in wat clothing — any edit
to `service.wat`, however semantically inert, invalidates four goldens.

**Work:** recapture the four (`UPDATE_EDN=1`), then verify the diff is the line number and nothing
else. **And record the class**: a golden that pins a stdlib line is a golden that will break again.
Whether to strip stdlib spans from these goldens, or accept the recapture, is worth naming — but do
not silently recapture and move on.

## CAUSE 2 — THE RETE COMPILER STILL READS THE OLD ARM (`grid_axes` ×2) ⛔ THE SUBSTANTIVE ONE

```
malformed :wat::rete::lower form: malformed match arm
src/rete/expr_ir.rs:525   LowerError::unsupported(…, "malformed match arm")
src/rete/purity.rs:1306   "<malformed match arm>"
```

`Expr::Match` exists in the IR, but its **arm reader is on the retired grammar**. `lower()` correctly
refuses rather than guessing — the "total or it refuses" contract doing its job — so a rule body
containing a `match` no longer compiles natively and the `where-control` / `where-record` grid axes
go dead.

★ This is the one that matters beyond the migration: **the rete compiler is wat's future, and it
cannot lower the language's current match form.** It should move with the grammar, not behind it.

### ⛔ AMENDED — this is an IR CHANGE, not an arm reader fix. I understated it.

Reading the code rather than the error message: rete's refusal is a **stated scope boundary**, not
drift. `lower_pat` says so twice, in as many words:

```rust
"match map-destructure is not lowered in v1"      // at the List-with-Map head, and at a bare Map
Pat::Variant { name, payload: Option<Box<Pat>> }  // payload is ONE POSITIONAL nested pattern
```

So `lower()` is not failing to keep up — it is correctly refusing something it was never given. The
work is three layers, and only the first is what I originally wrote:

```
1  lower_match gains a WatAST::Vector arm (the bracket clause)             small
2  Pat::Variant's payload becomes NAMED — Option<Box<Pat>> cannot express
   {:k v}; it needs field-name -> sub-pattern                              AN IR CHANGE
3  the matcher consuming Pat binds BY NAME, not by position
```

★ **It is the identical position→name move the surface just made, one layer down.** The arms went
from `(Variant a b)` to `[Variant {:a a :b b}]`; `Pat::Variant` is still `(name, one positional
payload)`. And once the payload is named, **rete needs the declared field names too** — the same
fact the codemod needed and could not get until 296 L. `type-of` pays off a second time, here.

**Work:** items 1-3 above, plus `purity.rs:1306`'s classifier. Include the nested `[Variant {:k v}]`
(no body) form.

⚠ **If this is too large for this reland, SAY SO and split it** — causes 1 and 3 are independent and
can land without it. A grid axis staying dead with a NAMED reason is honest; a widened refusal is
not (STOP-3).

## CAUSE 3 — NOT THIS STONE'S (`wat_scripts` loader ×1)

```
wat-scripts/scratch-pad/probe-arc278-surface-registers-service-reads.wat
  unresolved reference :probe::chan-svc::State/seen
```

**This file was NOT rewritten by the arm sweep** — its last two commits are `ab52b7188` (angle-
bracket parametrics) and `037ef43ef`, neither of them the match-arm campaign. The failing reference
is a `Type/member` accessor on a `defservice`-generated `State` record.

The floor was 5220/5220 green immediately before the arm strike, so it regressed **during today's
campaign but not via the codemod's text rewrite of this file**. Candidates are the Rust-side changes
in the arm strike, or one of 296 H-2c / J / K / L, all of which touched type registration.

**Work: BISECT IT.** Do not fold it into cause 1 or 2 and do not fix it blind. If it turns out to
predate the campaign, say so — that is a finding, not a failure.

## STOP TRIGGERS

- **STOP-1 — the three causes are fixed as one.** They share a floor, not a root.
- **STOP-2 — cause 1 is recaptured without recording the class.** A golden pinning a stdlib line
  will break on the next stdlib edit; the recapture is the fix, the NOTE is the stone.
- **STOP-3 — cause 2 is made to "pass" by widening `lower()`'s refusal.** `lower()` is total or it
  REFUSES; that contract is the reason rete is trustworthy. Teach it the new arm; never soften the
  refusal.
- **STOP-4 — cause 3 is fixed without a bisect.** Its origin is unknown and guessing which stone
  broke it is how a real regression gets buried under a plausible patch.

## EXPECTATIONS

| # | what | expected |
|---|---|---|
| 1 | floor | `0 failed`. State the delta per cause, not a total |
| 2 | cause 1 | 4 goldens recaptured; diff is the line number ONLY; the class recorded |
| 3 | ⛔ cause 2 | `expr_ir.rs` lowers the bracket/map-pattern arm INCLUDING the nested no-body form; `lower()`'s refusal contract unchanged (STOP-3) |
| 4 | cause 2, proven live | both grid axes run non-vacuously; `spec_equals_native_on_every_where_family` PASS |
| 5 | ⛔ cause 3 | the bisect names the commit; if it predates the campaign, say so |
| 6 | the 4 probe rows | still PASS, `#[ignore]` 0 |
| 7 | clippy | 0 |
