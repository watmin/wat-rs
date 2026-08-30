# DESIGN — STONE total-T3: declaring nothing becomes ILLEGAL

> **Builder's ruling, 2026-08-30:** *"i think declaring nothing needs to be illegal — we do not
> tolerate optional here…"*

T2 minted `@Total` as OPTIONAL, defaulting to `Unreviewed`. That default is now struck. **Every
registration must declare its totality or fail to compile.**

## What changes

`@Total <Variant>` joins `@Purity`, `@Determinism`, `@Category` and `@added` as a REQUIRED
directive. Absent → `DocError::MissingTotality` → the `#[wat_intrinsic]` / `#[wat_special_form]`
macro refuses to expand, naming the offending verb and the four legal values.

```
438 registrations must answer     429 #[wat_intrinsic] + 9 #[wat_special_form]
  1 already does                  :wat::i64::/  @Total Partial  (T2b)
437 do not                        every one becomes a compile error
```

## ★ `:Unreviewed` STAYS, and the ruling makes it MORE load-bearing, not less

This is the one thing that could be misread as contradicting the ruling, so it is pinned here.

*Optional* meant **the author could say nothing and the substrate would invent an answer.** That is
what dies. `:Unreviewed` is the opposite: an author **must type it**, deliberately, and a reader can
`grep` it. Silence becomes impossible; ignorance becomes *declared*.

Four questions, on keeping it:

- **Obvious? YES** — `@Total Unreviewed` reads as "nobody has measured this verb," which is exactly
  what it means.
- **Simple? YES** — one directive, always present, four legal values, no absence case anywhere.
- **Honest? YES**, and this is the load-bearing one. Forcing 437 verbs to a `Total`/`Partial` answer
  in a single strike produces **guesses**, and a guessed `:Total` is a lie in a fence that ADMITS
  code into a `where`. `:Unreviewed` is default-deny — it refuses. An honest refusal beats a
  confident lie in the only direction that can hurt us.
- **Good UX? YES** — `grep '@Total *Unreviewed'` IS the totality work list, per verb, at the verb,
  and it shrinks monotonically.

## The one contract decision, pinned: the initial value is `Unreviewed` for all 437

**No existing hand-list is migrated into `@Total`. Measured, and here is why.**

The two surviving totality lists are not two censuses of one property — they answer **different
questions over different populations**:

```
rete/purity.rs  `total` sub-list   38 verbs   "exactly the verbs the 9-file / 98-row where-corpus
                                               uses inside a `where`, each verified total by
                                               READING its own implementation" — DEFAULT-DENY,
                                               deliberately narrow, SILENT about everything else
macros/eval.rs  is_pure_total     202 verbs   expand-time legality — 255's own DESIGN already
                                               plans to rename it `:expand-time-legal`
```

Set-differencing them yields **171** verbs the macro list calls total and the rete list does not —
but most of those are the rete list being *silent*, not *contradicting*. The genuinely load-bearing
disagreement is visible at `:wat::i64::/`: macro says total, rete says not, and the verb's own doc
says it raises on two distinct inputs.

**Migrating either list would import an answer to a question `@Total` is not asking**, into 437 doc
blocks, where finding it again would be far harder than it is today. The lists stay exactly where
they are, untouched, as T4's raw material.

★ Note what this makes T3: **entirely mechanical.** No verb's totality is adjudicated. Every site
gets the same line. That is a property worth having in a 437-site sweep — there is no judgement for
a sweep to get wrong.

## Out of scope = REJECTED

- **No consumer derives.** `intrinsic_meta`, `is_pure_total`, `RETE_OPS` are untouched and keep
  working. That is T4.
- **No verb is adjudicated `Total` or `Partial`.** Exactly one verb carries a real answer and it
  landed in T2b by transcription from its own doc.
- **The `i64::/` dispute is not resolved.** T4's first case.

## Files

```
crates/wat-doc/src/lib.rs                  DocError::MissingTotality; BOTH resolution points
                                           (parse ~:678, parse_special_form ~:996) stop defaulting
crates/wat-macros/src/wat_intrinsic.rs     render_doc_error gains the MissingTotality arm
                                           (the THIRD exhaustive match — T2's rider found it
                                           the hard way, via E0004; it is named here so this
                                           stone does not rediscover it)
src/**/*.rs  +  crates/**/*.rs             437 doc blocks gain one line
```

## Method — the sweep is an ephemeral Rust tool, and the placement is uniform

437 sites is well past the threshold where a hand-edit or a shell one-liner is the wrong instrument.
Build a surgical Cargo binary under repo-local `tools/`: read file → insert one line immediately
after the `@Determinism` line of each doc block that lacks `@Total` → write. Every other byte
untouched. Delete the tool before the commit.

**Placement is `@Total` immediately after `@Determinism`, before `@Category`** — matching
`:wat::i64::/`'s existing block, so all 438 read identically.

★ **Content-integrity is a separate axis from tests-green:** the tool must confirm each file's
non-ASCII character count is unchanged, and the orchestrator re-checks it independently at scoring.
A whole-file round-trip has silently dropped 5,720 non-ASCII characters in this repo before, while
the suite stayed green.

## Calibration

The compiler names every site, so the sweep is diagnostic-driven rather than census-driven — which
is the entire reason this route was chosen. Predicted 40–70 min.
