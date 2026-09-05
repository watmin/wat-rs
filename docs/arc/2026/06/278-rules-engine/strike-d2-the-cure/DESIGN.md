# DESIGN — one index, one insertion verb; and the differential that could not see this

## Why

**D2 is LIVE, proven `72b894ccb`.** `right_idx[J]` accumulates duplicate elements: J6 carries 18
elements against a mark of 12, J11 carries 12 against 6, persisting to fixpoint. Three sites append;
one maintains the mark.

**The engine is unfixed. The failing test is banked `#[ignore]`, and that was the orchestrator's
error** — the `RED-at-HEAD` idiom's other users are arc-255 rows banking *features not yet built*.
This is a defect in shipped behaviour. **This strike lands the cure and un-banks the test in one
move.**

## The mechanism, established causally

| site | appends | advances mark |
|---|---|---|
| `fire/mod.rs:802` (`keyed_join_persistent`) | ✅ | ✅ reads `already` `:799`, writes `:815` |
| `pass/hash_join.rs:185` first-keying catch-up | ✅ | ❌ |
| `pass/hash_join.rs:298` step-2 Δright | ✅ | ❌ |

J6: mark 6 after round 0 → step 2 appends 6 more without advancing → pass 3.7 reads `already = 6`,
sees 12, re-pushes `[6..12]` **which step 2 already placed**. J11: catch-up indexes all 6 → mark
absent ⇒ `already = 0` → maintainer re-pushes all 6.

## The contract decision, pinned

**Make the bypass unrepresentable — do not patch two call sites.**

`right_idx` and `indexed_n` are two fields that must move together and currently do not. The cure is
`sequi`'s shape: **one type owning both, with a single insertion verb** (`index_upto`, or whatever
the type names it) and **no public path that appends without advancing the mark**.

- **Rejected: bumping the counter at `:185` and `:298`.** That is the convention rung — it cures
  today's two sites and leaves the third writer free to appear. The defect has already survived one
  refactor (`partire`) precisely because nothing structural forbade it.
- **Rejected: a debug_assert.** A check that fires only in debug is not a wall, and release is where
  this runs.

**⛔ The acceptance test already exists and is banked.** `right_index_counter_tracks_its_bucket_population`
must be **un-`#[ignore]`d and green** — that is the strike's definition of done. It is self-clearing
by construction: green means the invariant holds.

## ⛔ AND THE DIFFERENTIAL COULD NOT HAVE SEEN THIS — that is a second finding

The grid's `:derived` is *"the FULL SORTED derived-fact **SET**"*. **D2 duplicates TOKENS**;
`seen_insert` dedups them into the same fact set. Both 2026-08-31 drives were native-vs-oracle on
`:derived` and came back clean **by construction, not by luck**.

Today's three-way port check inherits that blindness. It was given the *shape* it lacked (a
parametric axis, for D7) and was never asked whether its *observable* could see a multiplicity
defect. **It cannot.**

**That gap is ROWED here, not fixed here** — a multiplicity-sensitive column plus a
`filter → HJ(a) → HJ(b)` axis **with the two-wave stagger** is its own strike. ⚠ The stagger is
load-bearing: the shape alone produced a **vacuous partition** where the two writers never met on one
index, and the probe's own guard caught it.

## Out of scope = REJECTED

- **The grid axis.** Rowed above; drawn separately on this evidence.
- **Any performance claim.** This is a correctness fix; if it costs, measure it, but do not sell it.
