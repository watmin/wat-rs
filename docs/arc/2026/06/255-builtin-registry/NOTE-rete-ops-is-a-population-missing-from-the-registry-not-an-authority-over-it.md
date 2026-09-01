# NOTE — `RETE_OPS` is a POPULATION MISSING from the registry, not an authority OVER it

> Measured 2026-08-31 while sizing wave 3. **No row, nothing drawn.** This CORRECTS a framing my own
> wave-2 DESIGN shipped, so the campaign that takes `rete_op_for` inherits the measurement instead of
> the error.

## What wave 2's DESIGN said, and why it was wrong

> *"`rete_op_for` (line 251) — a whole vocabulary table with its own consumers and its own `OpMeta`
> per row. **It is the largest remaining authority** and it needs its own design."*

Filed alongside the prefix guesses, as though it were the same kind of thing. **It is not.**

## The measurement

```
RETE_OPS rows ....................... 74   (Alias 35 · Fallback 20 · Redispatch 10 · Form 9)
rows carrying an OpMeta literal ..... 74
rete_name REGISTERED as an intrinsic .. 0 / 74
core_name REGISTERED as an intrinsic .. 46 / 74
row meta vs. the CORE verb's declaration ... 17 "disagreements"
```

⛔ **The 17 are not disagreements.** A rete row is a **DIFFERENT VERB** from its core twin, usually a
*totalized* one — which is the whole point of a vocabulary Law A can admit. Two worked examples,
both verified against the corpus:

```
:wat::rete::i64::/          class Fallback, params [I64 I64 Keyword I64]
  core :wat::i64::/ is PARTIAL (zero divisor — runtime-meta.wat's canonical example)
  the row is TOTAL, and honestly so: (:wat::rete::i64::/ 1 0 :undefined -1) returns -1
  its own comment says it: "Total BY CONSTRUCTION: the caller's `:undefined` value covers
  the undefined point"

:wat::rete::string::concat  fixed arity 2
  core :wat::string::concat is PARTIAL (variadic; check.rs:14944 admits arity 0)
  the row's narrower domain is genuinely total
```

★ **I nearly filed `:wat::rete::i64::/` as a live defect** — "a row claims total for a division that
can divide by zero." The comment two lines above the row is its refutation, and the corpus proves it.
Reading the row's own neighbourhood answered what the row alone could not.

## The correct framing

`rete_op_for` does not OVERRULE the registry the way the prefix guesses did. It answers about
**74 verbs the registry has never heard of** — `rete_name` is registered **zero** times.

So the remaining work is not a RETIREMENT, it is a **HOMING campaign**:

> Home the 74 rete-surface verbs. Their `OpMeta` then becomes a copy of the truth, and
> `rete_op_for`'s early return retires by itself — the same motion every other guard in this campaign
> has made, arrived at from the opposite direction.

⚠ **And it is not one wave.** The four `OpClass`es differ in what a registration would even mean:
`Alias`/`Fallback` carry `params`/`ret` that `check.rs` registers a `TypeScheme` from; `Form` and
`Redispatch` carry **no scheme at all** and are checked by dedicated inference arms. A design that
treats the 74 as one population will be wrong for 19 of them.

★★ **Builder, 2026-08-31, naming the destination:** *"the purity measurements rete is doing… they
will be satisfied by the registry when we're done… not there yet."* This NOTE is what "not there
yet" measures to: 74 verbs, zero homed, four classes.
