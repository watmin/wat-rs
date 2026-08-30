# DESIGN — STONE expand-T2: `@ExpandTime` parses AND reaches the entry

> T1 (`0625c6b2c`) minted `ExpandTime = Legal | RuntimeOnly | Preserving | Unreviewed` in
> `wat/runtime-meta.wat`, generated the Rust type, and broke the door to prove it.

## What T2 delivers

**A verb can DECLARE its expand-time legality where it is defined, and that declaration reaches
`IntrinsicEntry`.** Parse and carriage in one stone.

## ⛔ ONE STONE, BECAUSE SPLITTING THEM LAST TIME PRODUCED A CONTRADICTION

Totality's equivalent shipped as two: `total-T2` (parse) then `total-T2b` (carriage). That was a
mistake and it cost a rider a wasted flight — T2's brief demanded proof the value reached the
registry while its blast radius forbade `src/intrinsic/mod.rs`, **where the registry entry lives**.
The rider could not satisfy both, shipped the honest maximum, and reported the contradiction.

`[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`

So this stone's blast radius includes the entry from the start.

## The one contract decision, pinned

**`@ExpandTime` is OPTIONAL in T2 and defaults to `ExpandTime::Unreviewed`.** T3 makes it required.

Same reasoning as totality, and it earned its keep there: mandating it here forces ~433 sites to
answer in one strike, and answers produced under that pressure are guesses. **A guessed `Legal` is
a verb admitted into a macro body it may corrupt**; an honest `Unreviewed` is default-deny and
merely refuses. The T5 result is the proof this pays — 275 verdicts moved safely precisely because
the unmeasured pole refuses rather than guesses.

## The sites — named, so this stone does not rediscover them

Totality's T2 rider found two of these the hard way. They are written down now:

```
crates/wat-doc/src/lib.rs   DocComment.expand_time — the field, beside purity/determinism/totality
crates/wat-doc/src/lib.rs   DocSpecialForm — a SIBLING TYPE (lib.rs:250), not the same struct,
                            parsed by its own `parse_special_form`. BOTH need the field.
crates/wat-doc/src/lib.rs   BOTH resolution points default rather than erroring (~:678, ~:996)
crates/wat-doc/src/lib.rs   DocError::InvalidExpandTimeVariant
crates/wat-macros/…/wat_intrinsic.rs      value -> quote! match  (the `totality_token` twin)
crates/wat-macros/…/wat_special_form.rs   the same match, again
crates/wat-macros/…/wat_intrinsic.rs      ★ render_doc_error — A THIRD EXHAUSTIVE MATCH. Adding a
                            DocError variant breaks it with E0004. Totality's rider hit this
                            unannounced; it is named here.
src/intrinsic/mod.rs        IntrinsicSubmission · SpecialFormSubmission · IntrinsicEntry,
                            plus BOTH submission -> entry conversions
crates/wat-macros/…         both emit literals splice the token
```

★ **`wat_special_form.rs` is NOT a twin for error rendering** — it uses `format!("{:?}", e)`, not an
exhaustive match. So the third match exists only on the intrinsic side. Also learned the hard way.

## Out of scope = REJECTED

- **Making it required.** T3.
- **Answering `@ExpandTime` for any real verb.** T4a transcribes the 202 existing answers; this
  stone annotates only its own fixtures, plus ONE real verb as the carriage proof (below).
- **`is_expand_time_legal`.** It keeps its hand-list until T4.

## The carriage proof must be two-sided

One real verb declares a non-default value and reads back as that value; another declares nothing
and reads back `Unreviewed`. Checking only the first passes if the field is hard-wired; checking
only the second passes if carriage is broken and everything defaults. **`:wat::core::fresh-symbol`
is the right subject** — it is the axis's own witness (nondeterministic yet legal), so declaring
`@ExpandTime Legal` on it says something true and load-bearing rather than decorative.

## Calibration

Predicted 40–60 min. Larger than totality's T2 because carriage is included, smaller in surprises
because every site totality's rider discovered is named above.
