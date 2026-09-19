# DESIGN — `conferre` L2-1: the last cell of the leading-condition matrix, as a grid axis

## Why

L2-1's own report forbids calling it a defect *"until someone fires a leading accumulate into a
HashJoin across ≥2 rounds and counts rows."* This strike is that drive — **and it is a grid axis,
not a scratch probe**, because the builder's standing rule is that every flaw earns a grid that
proves it is gone, and here the axis can be the instrument of discovery *and* the standing proof in
one artifact.

## The class is already PROVEN — only this cell is unmeasured

`where-accum-lead-cascade.wat`'s header states the matrix and its history:

```
where-accum-lead   leading accumulate, in a RULE,  no cascade   — covered, agrees
leading-exists     leading :exists,    in a RULE,  WITH cascade — covered, agrees
that axis          leading accumulate, in a QUERY, WITH cascade — covered, agrees
```

The defect it was built for was **found by the rete differential fuzzer (2026-08-25, family A) and
is real**: *"the row count tracked the FIXPOINT ROUND COUNT exactly — a 1-rule inert chain gave 2
rows and a 2-rule chain gave 3, where both references give 1."* And the header states the reason
each cell needs its own axis: **"a fix for one did not reach the other."**

**Measured, anchored, 2026-09-07: 2 grid axes have a leading accumulate in a RULE
(`where-accum-lead`, `where-accum-group`); 0 of them have a cascade.** The fourth cell — leading
accumulate, in a RULE, WITH a cascade — has never been driven.

## And the guard exists on the sibling path only

- `filter.rs` threads `leading_emitted` through **five** sites (`:107`, `:135`, `:172`, `:278`, and
  the parameter at `:28`).
- `delta.rs:365` builds that map and hands it to **filter only** (`:623`).
- `accumulate.rs:134-145` re-seeds an empty token whenever `new_tokens.is_empty() && pids.is_empty()`
  — **every round, with no guard**, because it never receives the map.
- `rules.rs:538`: *"multiplicity defect was already closed on 2026-08-24 (`leading_emitted`,
  `71d0e700e`)."*

**The arc's signature shape for the twelfth time**, and this one is sharper than most: the cure was
not merely written down elsewhere — it was *driven, proven and shipped* on the sibling path, and the
sibling was never given it.

## The shape

Mirrors `where-accum-lead-cascade`'s design, moved from a QUERY to a RULE and given a join:

```
  Reading(v)      for v in [0, items)        [input — the accumulate's source]
  Anchor(k)       for k in [0, anchors)      [input — the join's right side]
  S1(i) :- Seed(i);  S2(i) :- S1(i);  S3(i) :- S2(i)      INERT cascade, read by nothing
  Busy(k,n) :- [?n <- (acc::count) :from Reading] AND Anchor(k)      ★ leading acc → join
```

⛔ **The cascade is inert and that is the whole design.** It derives facts the rule never reads; its
only job is to make the fixpoint iterate more rounds. So the answer must be **identical** at every
cascade depth, and any spread is the engine leaking its own round count into a result — the exact
signature the fuzzer caught in the sibling cells.

## The one contract decision

**The dial is the cascade depth, and the expected `:derived` does NOT move with it.** That inverts
the usual grid contract, where the count is a function of the size — here the whole assertion is
that it is *not*. The anti-vacuity count is therefore `anchors` (one `Busy` per anchor), constant,
and non-vacuity comes from `anchors > 0` plus the native/oracle/Clara three-way, not from the count
tracking a dial.

## Two outcomes, both worth the axis

1. **Rows scale with cascade depth on some engine** — L2-1 is a real defect, the cure is threading
   `leading_emitted` into `accumulate.rs`, and this axis is already the regression test.
2. **All three agree at every depth** — L2-1 is refuted; `accumulate.rs`'s seed is unreachable in a
   way the guard on `filter.rs` is not, and the row closes with a driven negative. The axis stays,
   because the fourth cell of a matrix whose own header says *"a fix for one did not reach the
   other"* should not be the one cell nobody watches.

## Out of scope = REJECTED

- **Any cure to `accumulate.rs`.** Outcome 1 earns its own strike; this one measures.
- The other three cells — covered, agreeing, untouched.
- `conferre` L2-3 (closed today), census G's gate, Stone K's relocation.
