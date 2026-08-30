# NOTE — a CALLABLE keyword in value position has four kinds and three answers

**Filed 2026-08-30.** Surfaced by an executor mid-strike on excursus 001 (*"`mapv` won't take a
record accessor as a function here"*); builder's read — *"that feels like a bug"*. Confirmed by
measurement, then **scoped from the rule rather than the instance**, per
`NOTE-a-function-type-prints-in-a-spelling-you-cannot-write.md`'s own lesson.

## The rule

> **A callable keyword in value position must resolve to a function carrying the types it
> actually has.**

wat has four kinds of callable. Measured against that rule, they give three different answers.

| kind | value-position type | usable as a value? |
|---|---|---|
| user `defn` | `[:probe::R :-> :wat::core::String]` — concrete | ✅ yes |
| **record accessor** `:probe::R/sk` | `[:wat::core::Record :-> :wat::core::String]` — **abstract receiver** | ⚠️ passable, but only where the ABSTRACT type is expected |
| **constructor** `:probe::R` | `:wat::core::keyword` — not a function | ❌ no |
| **intrinsic** `:wat::core::str` | `:wat::core::keyword` — not a function | ❌ no |

Identical syntax, identical position. The answer depends only on how the callee was minted.

## The accessor case, and why it is the strange one

The constructor and intrinsic rows are one known defect —
`255/NOTE-an-intrinsic-cannot-be-passed-as-a-value.md`: no `Function` entry exists, so the
checker reports what the token literally is. Honest, if unhelpful.

The accessor **does** resolve. Its type is simply wrong, and the failure inverts the usual
intuition:

```wat
(:wat::core::defrecord :probe::R [sk <- :wat::core::String])

(:probe::R/sk r)                                                          ;; GREEN — direct call
(:wat::core::mapv :probe::R/sk (:wat::core::Vector :- [:probe::R] …))     ;; RED
(:wat::core::mapv :probe::R/sk (:wat::core::Vector :- [:wat::core::Record] …))  ;; ★ GREEN
```

```
parameter #1 expects [:probe::R :-> :?N]; got [:wat::core::Record :-> :wat::core::String]
```

★ **Being MORE specific about your data makes the program stop compiling.** The only difference
between the RED line and the GREEN one is that the green one throws the element type away.

It is not about `mapv` or collections — a bare higher-order fn whose parameter is
`[:probe::R :-> :wat::core::String]` reproduces it with nothing else in scope.

## Why the direct call survives

`src/check.rs:5602–5620` narrows at the call site, and says so itself:

> *"Arc 258 cascade — accept any subtype of `:wat::core::Record` (includes specifically-typed
> records like `:myapp::Pt` …). `is_subtype` is reflexive so `:wat::core::Record` itself still
> matches."*

So a check at one use site compensates for an over-general declared receiver. Value position
gets no such compensation, and wat has no parameter contravariance, so `Record -> String`
never unifies with `R -> ?`.

★ **The compensation is the tell.** Where a check props up a type that is wider than the thing
it describes, the type is what is wrong — the check is scar tissue. Same shape as
`gen-tests/FINDINGS.md` F3's three "declared optimistically" sites, moved from the return
position to the parameter position.

**Honest limit:** I measured the behaviour and found the compensating call-site path. I did
**not** find the line that constructs the accessor's scheme with a `Record` receiver. Start
there, not from this paragraph.

## What is owed

- **The accessor's scheme carries the concrete receiver** — `R -> String`, not
  `Record -> String`. Then `check.rs:5602`'s subtype narrowing becomes redundant rather than
  load-bearing, and value position works without help.
- **Constructors and intrinsics resolve to functions** — the arc-255 row, unchanged.
- ⚠ **And a census scoped from the RULE, not from these four.** The instrument is a
  round-trip: for every kind of callable the substrate can mint, bind it in value position and
  pass it where its own declared type says it belongs. Anything that fails is the same defect.
  Four kinds are what I could enumerate today; the rule does not promise there are four.

## Not blocking

The workaround is one line, and an executor reached for it unprompted:

```wat
(:wat::core::defn :probe::get-sk [r <- :probe::R] -> :wat::core::String (:probe::R/sk r))
```

A user `defn` lands in `sym.functions` with a scheme naming the concrete receiver, so it
unifies. No runtime cost.

## Kin

- `2026/06/255-builtin-registry/NOTE-an-intrinsic-cannot-be-passed-as-a-value.md` — rows 3 and 4;
  same rule, different mechanism (no entry at all, rather than a wrong type).
- `excursus/2026/08/001-sns-sqs/NOTE-a-record-accessor-in-value-position-loses-its-receiver-type.md` —
  the discovery record, with the full three-probe measurement.
- `NOTE-a-function-type-prints-in-a-spelling-you-cannot-write.md` — the census-from-the-rule
  discipline this NOTE followed, and the reason the table above has four rows instead of one.
