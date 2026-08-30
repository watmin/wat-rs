# DESIGN — STONE total-T2: the Totality axis becomes DECLARABLE at the registration site

> **Builder's ruling, 2026-08-29:** *"we have been dragging our feet on building a totality
> measurement — it sounds like now is the time to do it…. (in the near future we will only support
> totality….. but … this effort will identify who doesn't … and its a future work list)….. it
> sounds like we must… for two discrete reasons at minimum….."*
>
> **T1 is struck** — `525dbdb5b`, the FM 2-bis probe: the `Totality` defenum exists in
> `wat/runtime-meta.wat`, the Rust type generates from it, and the door was broken (`E0599`) to
> prove the generation is real.

## Why — and there are FOUR reasons, not two

| # | consumer | what it does today |
|---|---|---|
| 1 | arc 278's `where`-fence | `(and (pure? f) (deterministic? f) (total? f) (primitive? f))` — three axes have a Layer-1 home; `total` never did |
| 2 | the completeness gate's 25-verb hole | those verbs already declare `@Purity`/`@Determinism`; they cannot be DERIVED-and-ruled while one axis has nowhere to live |
| 3 | `macros::is_pure_total` (`eval.rs:353`) | 411 lines, 202 verbs, hand-written |
| 4 | `rete/vocabulary.rs`'s `RETE_OPS` | its own `op.meta.total` |

★ **And the lists do not agree.** `intrinsic_meta` names 177 verbs, `is_pure_total` names 202, and
only **102** are shared — 100 macro-only, 75 rete-only. They give **opposite answers for
`:wat::i64::/`**: `is_pure_total` includes it (*"div-by-zero is a deterministic located abort …
never a panic"*) while `intrinsic_meta`'s `total` sub-list explicitly excludes it (*"`i64::/` is
both, and undefined at a zero divisor"*). Nothing compares them. That is not drift — it is **two
different properties wearing one name**, which 255's own DESIGN already suspected when it planned
`is_pure_total`'s replacement as `:expand-time-legal`.

## What T2 delivers

**The `@Total <Variant>` directive, parsed and carried on `DocComment`.** After this stone a verb
can DECLARE its totality where it is defined, and every consumer can read it.

## The one contract decision, pinned

**`@Total` is OPTIONAL in T2 and defaults to `Totality::Unreviewed`.**

Mandating it here would force 429 sites to answer in a single strike, and answers produced under
that pressure are guesses. **A guessed `:Total` is a lie in a fence that ADMITS CODE into a
`where`** — strictly worse than an honest `:Unreviewed`, which is default-deny and refuses.

Making it mandatory is **T3**, and it is deliberately a separate stone: Layer 1's *no `Default` →
struct-literal completeness is a compile error* rule turns that flip into compiler-generated
worklist, which is the whole reason this route was chosen over a census.

★ **Why the split is the right decomposition (proactive slicing).** T2 makes the axis *readable*,
which lets us MEASURE — before committing to the grind — how many of the 429 already have an answer
recoverable from the three existing hand-lists, and how many are genuinely unmeasured. **That
measurement sizes T3.** Bundling them means starting the grind without knowing its size.

## Out of scope = REJECTED (affirmative cuts, not deferrals)

- **No consumer changes.** `intrinsic_meta`, `is_pure_total`, `RETE_OPS` are untouched. They keep
  their hand-lists and keep working. Making them derive is T4.
- **No Layer-1 baseline field.** The LOCKED RECORD MODEL is not edited by this stone.
- **No verb is annotated.** Zero `@Total` directives are added to real verbs here; the mechanism
  ships unused except by its own fixtures. Annotating is T3's grind.
- **The `i64::/` contradiction is NOT resolved here.** It is named, and it is T4's first test case —
  because the resolution is a ruling about what the two lists MEAN, not an edit.

## Files, and the mirror set

`wat/runtime-meta.wat`'s own correction paragraph (2026-08-19) is the map — it was written *because*
adding five `Category` variants broke the build in three places, after a stale-to-stale gate stayed
green. `Determinism` is the exact structural twin to copy; every site below is where it already
appears.

```
crates/wat-doc/src/lib.rs         DocComment.totality field · parse `@Total` · DocError::
                                  InvalidTotalityVariant · the round-trip test's own list
crates/wat-macros/src/wat_intrinsic.rs      value -> quote! token, one arm per variant  (~:787)
crates/wat-macros/src/wat_special_form.rs   the same match, again                        (~:75)
```

★ **The property that holds** (and the one that does not): a generated TYPE cannot drift from its
generator, but **every mirror that turns a value back into something else is hand-written.** They
cannot drift *silently* — each is an exhaustive `match`, so a missing arm is `E0004`, a hard error.
Build green + test-build green = every match-shaped mirror reached.

## The algorithm

1. `DocComment` gains `totality: Totality`, defaulting to `Totality::Unreviewed` when no `@Total`
   directive is present. Same for the special-form doc struct.
2. `@Total <Variant>` parses with the same shape as `@Determinism` — singleton (a second occurrence
   is `DuplicateSingleton`), unknown variant is `InvalidTotalityVariant { got }` naming all four.
3. Both proc-macros gain the value→token match arm, exhaustive, no wildcard.
4. The registry entry carries it through, exactly as `determinism` is carried.

## Calibration

Predicted 25–45 min. Comparable: the `@yields` subject stone (P5-b) and the `Category` five-variant
addition, both of which touched this same mirror set.

## Probe contracts (T1, already committed at `525dbdb5b`)

`crates/wat-doc/src/lib.rs`'s `probe_totality_axis` — four variants named, match exhaustive. Proven
non-vacuous by renaming `:Partial` in the `.wat` and observing `E0599` ×4, `EXIT=101`.
