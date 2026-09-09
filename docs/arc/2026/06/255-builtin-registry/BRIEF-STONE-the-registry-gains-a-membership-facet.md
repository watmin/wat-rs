# BRIEF — STONE ①′: the registry gains a MEMBERSHIP facet

## Why the first attempt correctly refused

STOP-1 fired on `BRIEF-STONE-the-rete-vocabulary-becomes-rows.md` and **zero rows were registered.**
My brief asserted *"the table already carries everything a row needs."* It does not — measured
field-by-field against `IntrinsicEntry`:

```
category: wat_doc::Category   16 variants, every one a positive claim about what a verb DOES,
                              and NO `Unreviewed` pole (unlike Purity/Determinism/Totality/
                              ExpandTime). `ReteOp.class` answers HOW IT DISPATCHES — an
                              orthogonal axis, by the enum's own doc. Nothing on the row answers
                              this, and choosing one would misdescribe most of the 74.

ret: ParamType                the field's OWN doc: "Alias/Fallback only — unused for
                              Form/Redispatch." A placeholder for 19 of 74 rows. Registering from
                              it would publish PersistentVector's return type as `bool` — a
                              documented filler transcribed as truth.
```

★ I designed from the producer and never opened the consumer.
`[[feedback_a_design_is_unfalsifiable_until_something_consumes_it]]`

## ⛔ The ruling — the registry is the SOLE authority, so the facet goes IN it

**Builder, 2026-09-09:** *"255 demands that the registry becomes the sole authority for these
questions… so we pave another road we need."*

The rejected alternative was resolve consulting `RETE_OPS` directly at the `where` boundary. That
would make the vocabulary table a **second authority**, which is the exact defect this arc exists to
end. A membership facet keeps one door.

> **The registry answers TWO questions, not one: *"what is this verb's full contract?"* and
> *"does this name exist?"* A population whose contract is not fully known can still be a MEMBER.**

★ The type side already reads this way and is the precedent to mirror:

```
TypeEnv   types: HashMap<String, TypeDef>   full structure
          builtin_names: HashSet<String>    membership WITHOUT structure
          contains()  = types ∪ builtin_names
          get()       deliberately does NOT consult builtin_names — a leaf has no TypeDef to return
```

`src/types.rs:597` says why, verbatim: *"`get` intentionally does NOT gain the same `||`: a builtin
leaf's whole point is that it has no structure to return."*

## The work

1. `IntrinsicRegistry` gains a membership-only set beside `entries`, mirroring `builtin_names`.
2. A membership query — the question `resolve` will ask — consulting **both**. `lookup_entry` keeps
   consulting `entries` only, for the same reason `TypeEnv::get` does.
3. Fold `RETE_OPS`' 74 `rete_name`s into the membership set. **Names only** — no fabricated
   category, no placeholder return type.

⛔ **Do NOT add `Category::Unreviewed` and do NOT use `ReteOp.ret`.** Both were measured as
dishonest above. If the design seems to need either, the facet is being built wrong.

## Acceptance

```
cargo clippy --release --all-targets -- -D warnings   EXIT 0
-E 'test(p1_annotation)' 10 · -E 'test(a1_one_rule)' 4 · -E 'test(a2_a_variant)' 15
```

State in the SCORE: how many names entered the membership set, and that `lookup_entry`'s behaviour
is unchanged for every existing caller.

## STOP triggers — each is a REJECTION

**STOP-1.** If a full `IntrinsicEntry` must be fabricated for any rete row — STOP. That is the wall
the first attempt hit and it was right.

**STOP-2.** If `lookup_entry` starts answering for membership-only names — STOP. A caller asking for
a contract must not receive a name that has none; that is the `TypeEnv::get` asymmetry, deliberately.

**STOP-3.** If any existing registry caller changes behaviour — STOP with the verbatim block. This
stone ADDS a facet; it alters nothing that exists.

**STOP-4.** If a second membership set appears anywhere outside the registry — STOP. One authority
is the entire point.

## Tier

You edit and report. Run clippy and the three targeted filters. **Do NOT run `scripts/floor.sh`, an
unfiltered `nextest`, or any corpus sweep** — I measure. **Do NOT background a command and end your
turn.** **Do NOT contact any peer.**
