# NOTE — is a primitive its own constructor? `(:wat::core::keyword "x")` vs `keyword/from-string`

**Filed 2026-08-23**, at the builder's request, watching `STONE-defservice-emits-the-binder` retire
`keyword/of`:

> *"i noticed it uses `:wat::core::keyword/of` … aren't primitives their own constructors?
> `(:wat::core::keyword "something")` => `:something` ? … i think a NOTE can serve as a reminder to
> see if we want this or not"*

**A reminder, not a ruling.** Nothing here is decided.

## ★ THE POINT, stated plainly — `keyword/from-string` IS the constructor

The builder's clarification, and it is the whole NOTE:

> *"keyword/from-string is literally what i'm asking for — ctor takes a string and returns a keyword
> of it?"*

**Yes. It already has exactly the constructor's signature.** From the substrate's own registration
(`src/check.rs:17401`), comment and scheme both:

```rust
// keyword/from-string: String → keyword (inverse; rejects ':'-prefixed input).
params: vec![string_ty()],   ret: keyword_ty()
```

`String → keyword`. That is a constructor for `:wat::core::keyword`. Nothing about it needs designing,
discovering, or building — **only the spelling is wrong.** The proposal is therefore not "add a
constructor"; it is:

```clojure
(:wat::core::keyword/from-string "something")     ;; today
(:wat::core::keyword "something")                 ;; the type IS its own constructor
```

A verb hangs off a type that does not need one. The type already owns the arrow.

## The principle, and it is already on the board

`294/SEAM.md` carries it as an open item:

> *"**`List/of` + `char/of`** retire into `List`/`char` (**verb-equals-type**). 63 sites, all
> tests/probes."*

And the collections already demonstrate it — the type name IS the constructor, in value position,
with the param-spec inline:

```clojure
(:wat::core::Vector :- [:wat::core::i64] 1 2 3)   →  [1 2 3]
```

Same name, two positions: a TYPE in an annotation slot, a CONSTRUCTOR in value position. That is
verb-equals-type working today.

## The observation this NOTE exists to record

**`/of` is not the only suffix wearing this shape. `/from-string` is the same thing.**

```
:wat::core::List/of              queued for retirement (seam)
:wat::core::char/of              queued for retirement (seam)
:wat::core::keyword/of           RETIRED by STONE-defservice-emits-the-binder
:wat::core::keyword/from-string  ← 9 references. NOT examined by anyone yet.
```

`keyword/from-string` is the clearest case of the four, because its declared signature is ALREADY
`String → keyword` — the others need a judgement about what their constructor should take, and this
one does not.

The `/of` retirement was scoped from the spelling `/of`. **A census scoped from a SPELLING misses
every sibling spelled differently** — `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`. The
rule is *"a type does not need a verb to construct it"*; `/of` is one phrasing of the violation and
`/from-string` is another.

## What is NOT settled, and must be measured before anyone acts

- **Is `:wat::core::keyword` dispatchable as a call head today?** A static read found only an EDN-tag
  match (`src/edn_shim.rs:2175`), not a call-dispatch arm — so probably not, but that is a READ and
  owes a measurement. This NOTE was written with a rider in the field; running the binary would have
  measured that rider's uncommitted work, not main.
- **What happens to the AST-node twin?** `keyword-node` builds a `WatAST::Keyword`, not a `Value`.
  Verb-equals-type says nothing about which of the two `(:wat::core::keyword "x")` should return, and
  a constructor that silently picks one is the kind of ambiguity this arc keeps deleting.
- **Does the principle run the other way?** `keyword/to-string`, `keyword/to-symbol`,
  `keyword/to-type-form` are CONVERSIONS OUT, not constructors. Verb-equals-type plausibly does not
  touch them — but nobody has said so, and an unstated boundary is how a rule over-applies.
- **`char/of` and `List/of` are 63 sites of evidence** already gathered. Whatever is decided here
  should be decided WITH them, not separately — three retirements of one shape done three times is
  the disease this arc exists to end.

## Why it is a NOTE and not a stone

The angle-bracket campaign is mid-flight and this is orthogonal to it. `keyword/of` had to go because
its purpose was minting the retired spelling; `keyword/from-string` has no such urgency — it mints
ordinary names, and the minting WALL (next stone) constrains what it may build without touching
whether it should exist.

Kin: `294/SEAM.md`'s `List/of` + `char/of` line, `[[feedback_scope_the_check_from_the_rule_not_the_diff]]`.
