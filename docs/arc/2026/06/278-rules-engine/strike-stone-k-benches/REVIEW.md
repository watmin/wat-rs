# REVIEW — the stone landed and the prediction was exact. One back-pointer is missing.

## What I re-ran myself

| claim | result |
|---|---|
| the three left the test binary | **HOLD** — `cargo nextest list \| grep -c` → **0** for all three |
| ⭐ skipped falls by exactly 3 | **HOLD** — `5480 run / 19 skipped`, matching the pre-stated prediction exactly |
| the two live gates survive | **HOLD** — both passed on the floor |
| the reasons survived | **HOLD** — `2028.8` present; the full margins and the two failed gate attempts are doc comments |
| floor | **HOLD** — `.floor/2026-09-07T23-17-15Z`: 5480 passed, 0 fail |

**The first floor was RED and was handled correctly** — captured, arm named
(`rete_citation_resolves::every_backticked_name_in_a_rete_comment_resolves`), cause diagnosed as
the surviving header citing an identifier the rename had removed, cured by restoring the name
rather than deleting the backticks. That gate has now caught a real defect three times today, twice
in the executor's own prose.

## ⛔ THE FINDING — the copy knows about the original; the original does not know about the copy

`benches/binding_repr.rs:27-28` is honest about what it is:

> *"Linear scan over an array map. Same body as the `Bindings` impl for `[(Value, Value)]`
> (`matcher.rs`), inlined so this crate does not reach the `pub(crate)` trait."*

**And `src/rete/matcher.rs` says nothing.** Grepped: zero mentions of `binding_repr` or `benches/`.

That impl is **live engine code** (`matcher.rs:169-175`, not `#[cfg(test)]`). So a hand editing
`impl Bindings for [(Value, Value)]` — swapping the linear scan for a binary search, say — silently
invalidates the bench's recorded margins, and the bench keeps reporting
`medians 2028.8 ns / 676.3 ns = 3.0×` as though it still described the engine. **The warning sits
where it is not needed and is absent where the break happens.**

⚠ This is not an argument against inlining. Inlining was the right call and widening the public API
for a benchmark would have been worse. It is an argument that a deliberate duplicate needs a note
at **both** ends.

## The work

Add a back-pointer at `src/rete/matcher.rs`, on the `[(Value, Value)]` impl: this body is
duplicated in `benches/binding_repr.rs` as `array_get`, the representation diagnostic's recorded
margins depend on the two staying identical, and a change here means re-measuring there or striking
the numbers.

⭐ **The cure protects itself.** `no_stale_path_in_doc`'s `ROOTS` include `src/rete` for `.rs`, so a
path cited in a comment there is gated — the back-pointer cannot rot into a stale path the way an
ungated note would. Cite the bench by path, not by prose.

## STOP triggers

1. **If the two bodies are NOT identical today** — STOP and report the difference. The bench's
   numbers would already be measuring something other than the engine, which is a finding that
   outranks the note.
2. Comment only. No logic, no API change, no re-measuring.

## ⚠ Process, named once

STOP-1 said: *"If any of the three needs a `pub(crate)` item — STOP. Report what it reaches; do NOT
widen the public API."* `Bindings::get` is that item. The condition fired and the strike continued
with a workaround instead of surfacing for a decision.

The workaround was the right one and the SCORE disclosed it clearly and first — that is what makes
this a note rather than a refusal. But a STOP is where the orchestrator chooses; had inlining been
the wrong trade here, the choice would have been made without me. **When a STOP fires, stop — even
when you can see a good way through.**
