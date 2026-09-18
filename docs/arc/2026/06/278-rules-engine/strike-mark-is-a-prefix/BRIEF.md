# BRIEF — D1: make the mark mean what `already` needs it to mean

**Floor GREEN when you are done.**

## Read in order

1. **`DESIGN.md`** — and note it records a correction to my own board row. The file under audit is
   not vacuous; the defect is narrower.
2. **`src/rete/kernel/session.rs`** — `JoinRightIndex`, `already()` (*"how many elements have been
   pushed"*), `writer()`, `RightIndexWriter::push`.
3. **`src/rete/kernel/fire/mod.rs:829-833`** — `already` used as a **slice offset** into
   `right_elements`. This is the mismatch.
4. **`src/rete/kernel/fire/pass/hash_join.rs`** — the first-keying catch-up. It pushes the whole
   `all_right`; `sequi` L2-a is the row.
5. **`src/rete/kernel/tests/right_index_counter_invariant.rs`** — the existing test. **Read its
   header in full**; it explains the reach guards and why `maintained_joins()` reads the census row.
   Do not weaken any of it.

## The work, in order

**1. Try to break it first.** Construct a fire where `already` ≠ the number of leading
`right_elements` actually indexed. The shape to aim at: the catch-up running on a join whose right
index the maintainer has already written. If you get it, you have a driven defect — report the
reading before curing.

**2. Cure the shape.** The catch-up indexes `right[already..]` rather than all of `all_right`, so
every writer respects the mark. Then the prefix property is structural, not a consequence of call
order.

**3. Extend the check.** Assert the elements indexed are exactly `right_elements[0..mark]` — the
property `already` relies on — alongside the existing count reading. Keep every existing guard.

## Blast radius

`src/rete/kernel/` only.

## STOP triggers

1. **If the tail-only catch-up changes ANY census number, STOP and report.** The catch-up currently
   pushes the whole memory; indexing only the tail is expected to change what it appends **when
   `already > 0`** — and if that happens today, you have found the live defect. Do not adjust a gate
   to match; surface the reading.
2. **If you cannot construct a violation, say so explicitly** and cure it structurally anyway. "I
   could not build it" is not "it cannot happen" — but it must be stated as the former.
3. **If the prefix assertion cannot be written without exposing bucket internals, STOP.** Do not add
   a `pub` accessor that re-opens what D2's cure closed; a `#[cfg(test)]` reader is acceptable only
   if no shipping path can reach it (D3's `push_ref` is the precedent, and I verified it never
   intersects a census assertion).
4. **On any RED: DO NOT RE-RUN.** Capture whole, name the arm, surface it.

## Prior result to copy for shape

`../strike-beta-write-door/` — private state, one door, and the proof is a compiler error.
