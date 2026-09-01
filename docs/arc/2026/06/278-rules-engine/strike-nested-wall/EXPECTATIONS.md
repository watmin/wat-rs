# EXPECTATIONS — an error kind nothing can produce is a promise the system does not keep

> **Every row's command was run against HEAD and its pre-value recorded.**

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,225 plus every arm you drive. Exceeding it is a PASS.**

## The scorecard, with pre-values measured at HEAD `7c28d506e`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | the nested undeclared field | **`"ACCEPTED-UNVALIDATED"`** (driven, twice) | **refused**, `UnknownField`, caret on `:nope` |
| 2 | the wall knows the shape | `grep -c kwargs-construct src/rete/validate/mod.rs` → **0** | ≥ 1 |
| 3 | `RhsMissingFields` fires | never — `x` unsupplied is accepted today | driven, with its caret |
| 4 | `RhsArityMismatch` fires | never, at this producer | driven — **or named unreachable with the reason** (STOP-2) |
| 5 | `RhsPositionalConstructionRetired` fires | never | driven — **or named unreachable with the reason** |
| 6 | the un-lowered branch | unknown | **driven** via `unreachable!`, and its fate reported |
| 7 | `aggregate-new` | driven evidence says it never arrives | **an arm only if a drive shows one arrives**; otherwise absent, and said so |
| 8 | the pin re-pointed | asserts `"ACCEPTED-UNVALIDATED"` | asserts the refusal; **anti-vacuity guard kept** |
| 9 | radius | — | `validate/mod.rs` + probes |
| 10 | lints | **116/116** (measured) | green |
| 11 | floor | **5225/5225** (measured) | ≥ 5,225 + every new arm, zero FAIL rows |
| 12 | clippy | **rc=0** (measured) | silent |

## The mutation proofs — one per kind, and they are four different refusals

Revert the head-recognition (make the wall read `items[0]` again). **All four kind-probes must
redden together** — that is the orphaning reproduced, and it is the one mutation where a shared red
set is correct rather than a leak.

Then, per kind, a mutation that disables only that arm; only its own probe reddens. **If disabling
one arm reddens another kind's probe, the two probes are not on separate kinds.**

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

60–80 minutes. The head-recognition is ten lines; four fixtures, four mutations, and trap 2's
reachability drive are the work.

## What would make this strike a failure even if every test passes

**Wiring the wall and leaving the four kinds unprobed.** Row 1 alone would go green, and the next
lowering orphans it again in silence — which is exactly how this hole was made. The probes are the
half that outlives the fix.

The second: **an `aggregate-new` arm added because `purity.rs` has one.** That is copying a sibling's
shape without its evidence, and it mints the dead code this strike exists to remove.
