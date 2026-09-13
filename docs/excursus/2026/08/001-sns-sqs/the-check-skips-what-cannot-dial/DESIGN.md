# DESIGN — the check skips what cannot dial

Builder: *"you've named two things to work on... so... let's work on them."* First of the two, deliberately:
**the guard BUYS floor time and the injector gate SPENDS it**, so do the buying first.

**Drawn 2026-09-13. NOT STRUCK.**

## Why

`the-dial-declares-its-peer` (`49bdf2b41`) added a third bijection check to `wat/service.wat`, and it cost
floor time — **measured twice, mine and grok's:**

```
before (four of my runs)   523.1 · 523.6 · 527.5 · 529.3 s   5239 tests
grok, after                570.2 s                            5241
mine, after                556.5 s                            5241
```

**+27 to +47 s, ~+5 % to +9 %**, uniform across the heaviest tests (`wat::lint` +46.7 s, `wat::cli` +12.4 s) —
macroexpansion costing more everywhere, as a per-`defservice` walk should.

⭑ **And the walk is unconditional.** `wat/service.wat:1029`:

```
impls-dialed-surfaces  (collect-impls-dialed collect-impls-dialed ops
                         (:wat::core::Vector :- [:wat::core::String]))
```

A service with **no `Address`-typed `:durable`/`:ephemeral` field at all** still walks its entire `:impls` tree
looking for connects that could never resolve to one.

## ⭑⭑ The guard is BEHAVIOUR-PRESERVING BY CONSTRUCTION — and that is the contract

```
impls-dialed-surfaces  (:wat::core::if (:wat::core::empty? addr-fields)
                         (:wat::core::Vector :- [:wat::core::String])
                         (collect-impls-dialed …))
```

> **If `addr-fields` is empty, no `connect` in `:impls` can resolve to an `Address`-typed field, so
> `collect-impls-dialed` can only return empty.** The guard returns empty directly.

★ That is a **proof, not a measurement** — and it is why this stone's correctness row is a *read*, while only
its *value* row is a run. `addr-fields` is built at `:971` from `:durable` ++ `:ephemeral`; the collector's only
way to add a surface is to match a connect argument against one of those fields.

## The one contract decision

> **The guard ships only if its saving EXCEEDS THE NOISE BAND.** My floor band is **~6 s** across four runs.
> If the guard recovers less than that, it is unmeasurable, and an unmeasurable optimisation in the
> `defservice` macro is complexity with no evidence — **revert it and record the null.**

⭑ **That null is a permitted, expected outcome**, not a failure of the stone. The +27–47 s might be dominated
by services that *do* have Address fields (the queue, the topic, the workers), in which case the guard saves
almost nothing and the honest result is "measured, not worth it."

## The four questions

**Obvious?** YES — don't search for something that cannot be there. **Simple?** YES — one `if` around one
expression. **Honest?** YES: the correctness claim is a construction proof and the value claim is a
measurement, and the stone commits in advance to reverting if the value is not there. **Good UX?** YES for the
floor and for anyone waiting on it; invisible otherwise.

## Scope

**IN:** the guard at `:1029` · **≥3 floor runs with it and ≥3 without**, all quoted · all 14 rows of
`the-dial-declares-its-peer` re-verified unchanged · a ruling: keep or revert, on the measurement.

**OUT = REJECTED:**
- ⛔ **Any change to the check's semantics.** This is a fast path, not a rule change. The refusal must still
  fire and the three corpus controls must still pass.
- ⛔ **Guarding on anything but `addr-fields` being empty.** A cleverer predicate (e.g. "no `connect` token in
  the file") would not be behaviour-preserving by construction, and the construction proof is the whole reason
  this is cheap to trust.
- ⛔ **Quoting a count of affected services.** ⚠ I tried: *"files mentioning an `Address` type"* is **97 of 235**
  files containing a `defservice` — and it **over-counts**, because helper signatures like `sqs.wat:1945`
  mention `Address` outside any service. **The floor delta is the deliverable; the service count is not
  measurable with a cheap instrument and must not be guessed.**
- **Optimising the collector itself.** If the guard is insufficient, that is a different stone with a different
  proof obligation.

## Trap-doors

1. ⛔ **≥3 runs each side, or the number means nothing.** The band is ~6 s and a single pair could show anything.
   The previous stone's own cost was only credible because two of us measured it.
2. ⛔ **The box must be quiet.** ⭑ And note the inversion this campaign learned: a quiet box is right for
   *timing* and wrong for *races*. **This is timing** — quiet box, and say so.
3. **Re-verify the refusal fires.** A fast path that skips too much would silently disable the check; the two
   floor tests from `49bdf2b41` are the guard against that and must stay green.
4. **`wat/service.wat` is stdlib**, frozen at build time — but no Rust change, so no stash-dance.
5. **The floor count is 5241**, not 5237 or 5239. Two stones have moved it.
