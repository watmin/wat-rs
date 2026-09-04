# DESIGN — STONE 1c-g: `=` and `not=` are registered `Partial`, and the placeholder dies

> **Builder, 2026-09-03:** *"= and not= are partial, not total — what other dilemma are we
> fighting here... i think these two have survived like 4 or 5 compactions now... they are
> unkillable.... it frustrates me"*
>
> And earlier, on the consequence: *"sift is making illegal calls - that's the heretic"* ·
> *"we fix it... and the blast radius reveals itself."*

**There is no remaining dilemma.** `=` and `not=` are `Partial`, proven twice by built-and-run
counterexample. This stone registers that fact, deletes the by-name placeholder, and takes the
revealed blast radius. Everything the campaign needs was already written down; nothing here is
re-derived.

## ⛔ The orchestrator's failure this stone corrects

These two rows have been held across multiple compactions. Each time, the hold was justified by a
*prerequisite* — `properties_of(name, arg_types)`, bounded generics, the alias-vs-restriction fork.
Measured this session: **none of those prerequisites changes the grade.** `=` is `Partial`
regardless, because `Value` and an unconstrained type parameter both admit `Fn`. A grade held
pending a mechanism that cannot alter it is not caution; it is a stall wearing caution's clothes.
The rows land now.

## The argued grades — LIFT, DO NOT RE-DERIVE

`[[NOTE-equality-is-argued-proven-partial-and-held]]` carries both complete doc blocks **verbatim**,
including the `#[wat_intrinsic]` wrappers, every one of the five axes with its `file:line` ground,
`@arg`/`@ret`/`@example`/`@see`, and the empirically-built `Partial` argument. That NOTE exists so
this stone is a transcription, not an argument. Copy it.

## The blast radius — MEASURED, not predicted

Method: empty `intrinsic_meta`'s `matches!` so `total?` answers `false` for both heads, build, run
the full floor, revert. Run 2026-09-03:

```
Summary [119.854s] 5129 tests run: 5124 passed, 5 failed, 17 skipped

FAIL  wat::rete      probe_arc278_foreign_pred_purity::foreign_pred_is_total
FAIL  wat::services  probe_arc278_sift_logs::sift_logs_pure_predicate_returns_only_survivors
FAIL  wat::services  probe_arc278_sift_logs::..._returns_only_survivors_on_process
FAIL  wat::services  probe_arc278_sift_arena::..._foreign_reader_counts_exact_survivors_across_a_process_fork
FAIL  wat           intrinsic::tests::the_residues_cannot_shadow_the_registry   ⬅ PROBE ARTIFACT
```

The fifth is an artifact of the probe's *shape*, not of the grade: the residue gate parses
`intrinsic_meta` for the `matches!` block by shape and panics `"the total derivation's fallback
matches! not found in intrinsic_meta — has it moved/renamed?"` (`src/intrinsic/mod.rs:3360`) when it
is gone. Retiring the placeholder retires that parser with it — see room 5.

## What the four fixtures actually compare — and why three repair and one does not

```
(= shp 0)                          i64      → :wat::i64::=               registered, @Totality Total
(= (ForeignRecord/class fr) "…")   String   → :wat::rete::string::=      RETE_OPS row, total
(= (Log/level log) Level::Error)   enum     → :wat::rete::core::enum::=  RETE_OPS row, total
(= s "high")                       Value    → ⛔ NOTHING
```

`:wat::edn::ForeignRecord/get` returns `(:wat::core::Option :- [:wat::core::Value])`
(`src/intrinsic/edn.rs:278`). `s` is a `Value`. `:wat::rete::string::=` declares
`[ParamType::String, ParamType::String]` (`src/rete/vocabulary.rs:1097`) and cannot take it, and
the `:wat::edn::` surface has no `Value`→`String` coercion (all 13 rows enumerated).

★ **So `(:wat::core::= s "high")` over a `Value` is not an illegal call — it is the only possible
call, and it is genuinely `Partial`.** `Value`'s declared domain includes `Fn`; `values_equal` has
no `Fn` arm. That the EDN reader can never actually produce a function is true and is carried by no
type in the system. `properties_of(name, arg_types)` would answer `Partial` here too — which is why
waiting for it was never going to unblock these rows.

**The honest consequence, stated as a capability loss rather than hidden:** after this stone, a sift
predicate that compares a foreign `Value` to a literal is **refused by sift's fence**. That is the
fence being correct about a predicate that is partial by type and safe only by an argument the type
system cannot hold. We ship the truth and record the loss.

## THE FOUR QUESTIONS — flat YES/NO

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **register both `Partial`, kill the placeholder** | YES | YES | YES | **NO** |

- **Obvious? YES** — the grade is proven by a committed counterexample; the doc blocks are written.
- **Simple? YES** — two registrations, three residue deletions, one gate retirement.
- **Honest? YES** — this is the entire point. A hardcoded `total: true` for a verb with a reachable
  raise is the substrate lying about itself, and it is the last such lie in this residue.
- **Good UX? NO, and it is accepted.** A legitimate sift predicate over foreign EDN stops being
  expressible. Obvious + Simple + Honest hold, so UX is the tiebreaker and not the load-bearing
  test — and the alternative (keep asserting `=` is total) buys that UX with a lie that misgrades
  **every** consumer of the axis, not just sift. The loss is named, bounded to `Value`-typed
  comparisons inside a fenced predicate, and left as the next stone's subject.

## Scope

**In:** the two `#[wat_intrinsic]` registrations · the `matches!` placeholder deleted entirely ·
`=`/`not=` removed from `intrinsic_meta`'s `pure_det` list and from `macros/eval.rs`'s expand-time
residue · the residue gate's `matches!` parser retired · the four fixtures brought to the truth.

**Out of arc 255 Stone 1c-g's scope, affirmatively:**
- **Restoring the `Value`-comparison capability.** It needs either a comparable-subset type or a
  coercion verb; both are design work with no consumer waiting but this one, and neither belongs in
  a registration stone. Tracked as the seam's next open fork, named, not deferred-in-prose.
- **`properties_of(name, arg_types)`.** Measured this stone as unable to change either grade.
- **The 8 blocked rete equality rows** (`alias vs RESTRICTION`). Still open, still Phase 1b.
