# NOTE — an alias restates its target's signature by hand, and nothing checks it. 16 of 52 disagree.

> **Builder, 2026-09-04:** *"do we need args, ret for aliases?... they are just delegated to their
> target?...."*

Asked of the registry rather than reasoned about. The answer is not the simple one.

## THE MECHANISM — the resolution pass copies FIVE fields, and the signature is not among them

`src/intrinsic/mod.rs:726-737`, the arc 255 Stone 2a-b pass:

```rust
alias_entry.purity = purity;
alias_entry.determinism = determinism;
alias_entry.totality = totality;
alias_entry.expand_time = expand_time;
alias_entry.category = category;
```

Five axes. **`args`, `ret` and `arity` are not copied**, and no gate compares them to the target's.
So `rete_alias.rs`'s own rule — *"an alias's axes ARE the target's… not restated here where they
could disagree"* — was applied to the axes and never to the signature sitting three lines above it.

## ⛔ THE MEASUREMENT — 16 of 52 alias rows report an arity their target does not

`wat-scripts/scratch-pad/255-alias-signature-drift.wat` (asks `(:wat::intrinsic::rows)` for every
alias row's `name|arity|alias-of`, joined against every row's arity):

```
alias rows 52  ·  ALIAS ARITY DISAGREES WITH TARGET: 16  ·  unresolvable targets: 0
```

Every one is the same shape — the target is `-1` (Variadic) and the alias declares a fixed arity.
And two of them alias the **same** target:

```
:wat::rete::core::enum::=     arity 1     ─┬─ both @alias :wat::core::=   (arity -1)
:wat::rete::core::keyword::=  arity 2     ─┘
```

The cause is visible in the source: `enum::=` writes ONE `@arg args …` line describing both
operands; `keyword::=` writes TWO. **The arity is derived from how many `@arg` lines someone
typed.** Restating the signature is not merely redundant — it is the mechanism by which the arity
gets invented.

## ★ AND THE ANSWER IS NOT "ALIASES DON'T NEED ARGS"

`:wat::rete::core::keyword::= arity 2` is probably a **true narrowing** — rete's grammar admits
exactly two operands where core `=` is variadic. `:wat::rete::core::enum::= arity 1` is almost
certainly a **typo**. **Nothing in the substrate can distinguish them.**

So the two halves of a row behave differently, and that is the finding:

```
AXES        DELEGATE.  Resolved from the target after folding; restating one is a
                       compile_error! (DocError::AliasDeclaresAxis). The rete FENCE is a
                       separate, contextual authority — see the alias-vs-RESTRICTION
                       resolution in DESIGN-STONE-the-rete-vocabulary-enters-the-registry.
SIGNATURE   MAY NARROW. And nothing declares whether a given alias narrows or inherits, so a
                       deliberate restriction and a miscount are indistinguishable — which is
                       exactly why 16 rows disagree on a green floor.
```

## THE DECISION OWED

Either an alias inherits its signature (and a narrowing needs its own vehicle — not an `@alias`
row), or a narrowing is DECLARED (`:args` present ⇒ deliberate restriction, absent ⇒ inherit) and a
gate compares the two. **What is not acceptable is the present state, where the same authoring
choice produces a true narrowing and a false arity with nothing to tell them apart.**

⚠ Related, and probably the same root: the SEAM's standing item *"19 rows lie about arity —
`#[wat_intrinsic]` derives Arity from the RUST SIGNATURE SHAPE; a `&[WatAST]` param ⇒ Variadic with
no shim check."* For a HANDLER-LESS alias row there is no Rust signature at all, so the `@arg` count
is the only source. Both are the same defect: arity inferred from a spelling rather than declared.

## CONSEQUENCE FOR `#wat.doc/Alias`

`[[DESIGN-the-tagged-edn-doc-row]]` gave `#wat.doc/Alias` an `:args`/`:ret`. **Corrected there:** an
`Alias` carries `:added`, `:alias`, and its own `:examples` (an example at the alias name genuinely
demonstrates that name). It does NOT carry `:args`/`:ret` unless the narrowing question above is
answered in favour of declaring them — in which case the key that carries them must MEAN
"deliberately narrower than my target", and a gate must hold it to that.

⛔ **Do not fix the 16 by editing `@arg` lines until that decision is made.** Half of them may be
correct.
