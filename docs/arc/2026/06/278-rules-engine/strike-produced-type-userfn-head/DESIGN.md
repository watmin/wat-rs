# DESIGN — `produced` diverges on a user-fn `:then` head; can it change FACTS?

## Why

Rowed by the header-audit strike (`c12a2e059`, `66a24d288`) and deliberately left unproven there.
Now driven on the oracle side.

`produced_type` (`src/rete/kernel/stratify.rs:65`) resolves a `:then` head through the
`SymbolTable`: if it names a fn whose return type is a non-`wat::core::` `Path`, it returns **that
return type**. The oracle's `rule-produces` (`wat/rete/oracle/stratify.wat:46-67`) reads the first
child's name and strips a leading colon — no symbol table, no resolution.

## MEASURED — orchestrator, 2026-09-07, oracle side, driven

Instrument: `wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat`, run through
`cargo run --release --bin wat` (**not** the installed binary — `wat/` is `include_str!`'d and
`~/.cargo/bin/wat` goes stale silently; that already cost this session one retracted measurement).

| rule | `(:wat::rete::rule-produces …)` |
|---|---|
| `:pt::plain` — ordinary fact-type head — the **ANCHOR** | `["pt::Rate"]` |
| `:pt::via-userfn` — `:then [(:pt::first-rate ?rates)]`, fn returns `:pt::Rate` | `["pt::first-rate"]` |

The oracle names the **function**. Native, by reading `produced_type`, names `pt::Rate`. The anchor
proves the extractor is sound, so the second row is a divergence and not a broken read.

**A user-fn `:then` head is a shipped form**, not a hypothetical:
`tests/rete/probe_arc278_then_user_forms_userfn.wat` drives exactly it.

## Why this could matter — and the reason it might not

`produced` is where the sweep **assigns** a stratum:

```rust
for p in &view.produced { if required > cur { type_strata.insert(p.clone(), required); } }
```

If the oracle assigns to `pt::first-rate`, then `pt::Rate` is **never raised**, and a downstream
rule consuming `pt::Rate` computes `req-pos` from 0. Stratified firing runs each group to fixpoint
and threads facts forward **without re-firing lower strata** — so a consumer stratified below its
own producer never sees the fact. That is the shape of a dropped-row defect, and it is the same
class as `16f504e14`, where the oracle was the wrong one.

**⛔ BUT THE BITING CASE MAY BE UNREACHABLE, AND THAT IS THE ACTUAL QUESTION.** The shipped probe's
own header records why its fn *extracts* rather than *constructs*: every surface a user fn could
build a NEW record through macro-expands to `:wat::core::kwargs-construct` or
`:wat::core::aggregate-new`, both of which `purity.rs`'s `KNOWN_UNREVIEWED` ratchet refuses in this
position. If a `:then` user fn can only ever hand back a fact that **already exists**, then whatever
rule created that fact already raised its type's stratum, and the divergence may never change a
single row.

So there are exactly two outcomes and both are worth having:

1. **A rule set exists where the facts differ.** Then the oracle mis-stratifies, Clara is the
   referee, and this is a real defect.
2. **The purity fence makes every such rule set unconstructible.** Then the divergence is
   latent-but-unreachable — which is a finding with a name, and the honest cure is a guard or a
   comment at `produced_type`, not a change to either stratifier.

## The one contract decision

**Arm 2 compares FACTS, and a failure to construct the fixture is a RESULT, not a failed strike.**
Outcome 2 must be reported with the exact refusal text that blocked each attempt, not summarised as
"couldn't reproduce" — `[[feedback_not_reproducible_is_not_a_disposition]]`. A count of attempts is
not evidence; the compiler's own words are.

## Files

- `src/rete/kernel/tests/` — a new in-crate test for arm 1 (the extractor is `pub(crate)`).
- `wat-scripts/scratch-pad/arc278-produced-type-userfn-head.wat` — already landed, the oracle half.
- Arm 2's fixture, wherever it must live to be driven.

## Out of scope = REJECTED

- **Any cure to either stratifier.** Not until arm 2 answers.
- `purity.rs` and its `KNOWN_UNREVIEWED` ratchet — explicitly out of scope in the probe that
  documented it, and out of scope here. If it blocks arm 2, that IS outcome 2.
- The six `mirror` claims verified in `c12a2e059`.
