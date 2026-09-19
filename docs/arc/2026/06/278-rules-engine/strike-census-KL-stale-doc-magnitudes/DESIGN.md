# DESIGN — census K/L: two doc claims in `census.rs` that overstate what the code does

Census audit sections K and L (`../vigilia-2026-09-05/recon/census-name-audit.md:125-129`),
re-grounded at HEAD 2026-09-06. **Both are documentation-only. No number is wrong, and no cure
touches an instrument.** Say so in the SCORE; the value here is that two comments stop teaching
something false, not that anything was at risk.

## ★ And the audit itself overstates L — that correction is part of the strike

The audit's L reads: *"The maintainer site is inside `if already < right_elements.len()` … and
catch-up inside `if first_keying` …, so **neither can ever emit a 0 row**."*

**Half of that is false.** Driven at HEAD:

| site | key | `n` argument | can it emit 0? |
|---|---|---|---|
| `hash_join.rs:350` | `STEP2` | `dr.iter().count()` | **yes** — outside the loop, its own comment says so |
| `hash_join.rs:216` | `CATCHUP` | `n_all.saturating_sub(already)` | **YES** — zero whenever `n_all == already` |
| `fire/mod.rs:928` | `MAINTAINER` | over `&right_elements[already..]` | **no** — the enclosing `if already < right_elements.len()` guarantees ≥ 1 |

So the property holds at **two of three** sites, not one. Only the maintainer cannot express
"ran and appended nothing" — and for it that state is not reachable: the guard *is* "there is
something to append."

**An inherited work-list row is a claim** (`[[an-inherited-work-list-is-a-claim]]`). This one was
checked and came back partly wrong; the audit gets amended rather than quietly worked around.

## L — what is actually wrong

`census.rs:842-845`, above `right_idx_appended`:

> *"Called with `n == 0` too: 'the block ran and appended nothing' and 'the block never ran' are
> different facts, and a census that cannot tell them apart is the blind spot the first D2 probe
> shipped."*

Stated of the function, it is true at STEP2 and CATCHUP and **false at MAINTAINER**. A reader who
takes it at face value would believe `site_ran(MAINTAINER)` distinguishes "the maintainer ran and
had nothing to do" from "the maintainer was never invoked." It does not — both produce no row.

**No live consequence:** the consumer never asks it. `right_index_counter_invariant.rs` calls
`site_ran` for `CATCHUP` (`:382`) and `STEP2` (`:388`) only, and reads the maintainer through
`n > 0` filters (`:269`, `:300`).

## K — what is actually wrong

`census.rs:623-626`, repeated at `:886-887`:

> *"the `alpha:*` marks fire PER FACT: at 40,200 facts that is ~3.2ms of pure clock-reading per row
> … THREE of alpha's five children (candidates/element/fieldnames) were individually SMALLER than
> their own instrument."*

Today **two** `alpha:*` marks exist — `alpha:seed` (`alpha.rs:312`) and `alpha:delta` (`:421`) —
both `phase_end` at the end of a pass function, i.e. **once per pass**. The five children and the
three named ones are gone.

★ **And the reason they are gone is this very measurement.** Dated 2026-08-01 against a
no-sub-marks control, it found 26% of the reading was the instrument — and the children were then
removed. **The justification succeeded, and now reads as a live description of the world it
eliminated.** The pair-count column it argues for is still correct (`census.rs:338`) and the
argument still holds in general; only the magnitude rotted.

## THE ONE CONTRACT DECISION

**Amend both docs to say what is true, and keep the reasoning.**

- **K**: mark the measurement as historical — dated, against five per-fact children, and the cause
  of their removal — then state today's shape (two marks, once per pass). Do **not** delete the
  measurement: it is why the column exists, and a column whose justification is deleted is the next
  thing someone removes.
- **L**: scope the claim to the sites that deliver it, and say why MAINTAINER differs (its guard
  makes "ran and appended nothing" unreachable, and no consumer asks).

## Out of scope = REJECTED

- **Moving the MAINTAINER call outside its guard** so the property becomes uniform. Nothing wants
  that row, it is an engine edit for an instrument's benefit (C10), and census G settled that an
  unread signal is not an improvement.
- Deleting the pair-count column or the 2026-08-01 measurement.
- Census M — a bench replica writing production keys is a hazard, not a doc claim, and gets its own
  strike.

## Verification — there is no mutation, and that is not a gap

Both changes are comments. The evidence is the two greps that establish the facts:

1. `grep 'phase_end("[^"]*alpha' src/rete/kernel/fire/pass/alpha.rs` → **two** marks, both per pass.
2. each `right_idx_appended` call's `n` argument, quoted, showing which can be zero.

Plus a green floor. Do not manufacture a red for a comment.

## STOP triggers

1. Any `alpha:*` mark turns out to fire per fact → STOP; K's premise is wrong.
2. `MAINTAINER` turns out to be reachable with `n == 0` → STOP; L's premise is wrong and the audit
   was right after all.
3. Any code outside a comment changes → STOP. This strike edits documentation.
