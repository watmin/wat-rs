# DESIGN — STONE total-T4b: the fence's `total` axis DERIVES; the hand-list becomes a backlog

> Builder: *"we continue to make the registry the sole source of truth for these properties."*

T4a (`89711f133`) moved 27 verified totality rulings to their registration sites. This stone makes
the fence read them, and collapses the duplicate T4a deliberately opened.

## The mechanism today

`intrinsic_meta` (`src/rete/purity.rs`) ends:

```rust
let total = matches!(head, ":wat::core::=" | ":wat::i64::>" | … );  // 38 names
Some(OpMeta { pure: true, deterministic: true, total })
```

Twenty-seven of those 38 names now carry the answer at their own registration site. The `matches!`
is a second copy of a fact the registry holds — the shape 255.1c retired as *"a gate reading a copy
of the truth"* (`intrinsic/mod.rs:988`, `[[feedback_a_gate_over_two_hand_lists_is_a_hand_list]]`).

## The derivation

```rust
let total = match registry().lookup_entry(head).map(|e| e.totality) {
    Some(Totality::Total)      => true,
    Some(Totality::Preserving) => true,    // the HEAD contributes no partiality
    Some(Totality::Partial)    => false,
    Some(Totality::Unreviewed) | None => matches!(head, /* the 11 unhomed */),
};
```

★ **`Preserving => true` is not a convenience — it is the convention the codebase already runs on.**
`matches!(entry.purity, Pure | Preserving)` appears at `intrinsic/mod.rs:1038` and
`intrinsic/reflect.rs:83`; `Preserving` satisfies an axis rather than failing it, because the form
contributes nothing of its own and `classify_expr` recurses into the arguments separately.

## ⛔ AND A CORRECTION THIS STONE MUST MAKE FIRST — the orchestrator's error

T4a's brief told the rider to transcribe all 27 as `@Total Total`. **Two of them were wrong**, and
their own doc blocks say so:

```
:wat::core::if     @Purity Preserving   @Determinism Preserving   @Total Total   ← inconsistent
:wat::core::let    @Purity Preserving   @Determinism Preserving   @Total Total   ← inconsistent
```

Two axes say *"I preserve my sub-forms' property"*; the third claims an intrinsic one. `if` has no
totality of its own — **it is total exactly when its branches are**, which is the sentence
`Totality::Preserving` was minted with in stone total-T1.

The rete sub-list agrees and points at the fix in its own words: *"`if`/`let` — value/control-flow
ops with no domain restriction … **exactly the convention `pure`/`deterministic` already use**."*
For these two verbs that convention IS `Preserving`.

**So T4b corrects `if` and `let` to `@Total Preserving` before deriving.** The fence's verdict is
unchanged either way (`Preserving => true`), which is what makes the correction safe to land here
rather than needing its own stone.

## The one contract decision, pinned: THE VERDICTS DO NOT MOVE

**For all 38 verbs, `intrinsic_meta(head).total` must return exactly what it returns today.** The
derivation changes *where the answer comes from*, never the answer. Any verb whose verdict flips is
a defect in the transcription or the mapping, not an improvement, and it is STOP-1.

The `where`-corpus (`wat-scripts/perf/grid/where-*.wat`, 9 files / 98 rows) is the live consumer
and must stay green.

## What the residue becomes

After derivation the fall-back `matches!` holds **exactly 11 names**:

```
map · mapv · filter · foldl · reduce            W7 HOF family — parked on effectful_by_prefix
= · not= · and · or · not · bool::to-string     remaining P6-c dispatch population
```

Every one is unhomed, and that is the *only* reason its ruling cannot live at its registration
site. **The list stops being a hand-list and becomes a named homing backlog** — `38 → 11`, each row
pointing at the wave that will retire it.

★ It should also carry a comment saying so, because a reader who finds an 11-name `matches!` with
no explanation will reasonably assume it is the same hand-list it replaced.

## Out of scope = REJECTED

- **Answering `@Total` for any further verb.** The other ~403 keep `Unreviewed` honestly.
- **`is_pure_total` (`macros/eval.rs`) and `RETE_OPS`.** Two more totality consumers; neither is
  this stone, and `is_pure_total` measures expand-time legality rather than totality at all.
- **Homing any of the 11.**

## Calibration

Predicted 30–45 min. Small in lines, load-bearing in behaviour: it is the first stone where the
registry ANSWERS a question the runtime asks.
