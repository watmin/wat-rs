# The seq-container narrow waist's own header says its capabilities are "stubbed" — all three are live and load-bearing

**Filed 2026-08-05, during arc 278 #57 1b. Small, not blocking, and worth a note precisely because
of WHERE it is.**

## The stale claim

`src/collection/seq_container.rs:27-31`:

```
//! # Scope (strike 1 — positional accessors only)
//!
//! This strike migrates `first`/`second`/`third` (the `Indexable` capability).
//! Other capabilities (`Tail`/`Append`/`Mappable`) are stubbed as methods for
//! later strikes; they do not change observable behavior today.
```

**All three are implemented and consulted across four files.** Measured 2026-08-05:

| capability | method | live call sites |
|---|---|---|
| Tail | `has_tail()` (`:174`) | `check.rs:4316`, `eval.rs:1725` |
| Append | `has_append()` (`:193`) | `infer.rs:188`, `runtime.rs:14869` |
| Mappable | `mappable()` (`:214`) | `infer.rs:854`, `:934` (foldl / foldr), via `extract_seq_elem` |

*"They do not change observable behavior today"* is false: `has_append` gates `conj`'s accepted
container set in the runtime, and `mappable` gates the fold family's in the checker. Both decide
whether real programs compile and run.

## Why a stale header HERE is worth writing down

This file is R14's **narrow waist** — the thing built specifically so that container knowledge stops
being hand-rolled per-op in `check.rs` and `runtime.rs` independently. Its own doc states the
purpose: *"Adding a new container type required touching every op on both sides by hand — the
O(ops)-per-container cost that caused the drift bugs (arc 220/249/278-0b)."*

A header that under-reports what the waist already carries invites exactly the behaviour the waist
exists to prevent: a reader who believes `Mappable` is a stub has no reason to route a new fold-like
op through the registry, and hand-rolls a per-container match instead. **The artifact most able to
re-breed the drift class is a stale note on the cure.**

## The smaller thing under it — an inconsistent naming shape

Two capabilities are `has_*` predicates called directly (`container.has_tail()`); the third is bare
(`StreamContainer::mappable`) and passed as a function reference into `extract_seq_elem`. Both call
shapes are legitimate — `extract_seq_elem` is a generic helper parameterised by a capability
predicate, which is *better* factoring, not worse. But the naming does not say so: `mappable` reads
as a different KIND of thing from `has_append` when it is the same kind used more cleverly.

Worth one of: rename to `has_mappable` for uniformity, or rename all three to bare adjectives and
let the call shape vary. **Not worth a strike of its own** — it wants to ride along the next time
this file is opened.

## What this does NOT change

**Nothing about #57's 1b ruling.** The five HOFs re-dispatch to core's inference either way; that
decision rests on `foldl` being polymorphic over the container CONSTRUCTOR (a rank-1 `TypeScheme`
cannot say *"C ranges over four constructors"*), which is true whether the constructor set is
derived from a capability method or listed by hand. Re-dispatch inherits whatever the waist does
today and whatever it does after any tidy-up — which is the property that made it the right call.

## Kept visible: the orchestrator corrected itself twice here, and the second one was wrong

Told the builder `foldl`'s container set is *"derived, not listed."* Then read the header's
"stubbed" line plus a `grep` for `has_mappable` that returned nothing, and **retracted a correct
statement** — telling him the claim was overstated. It was not. `mappable()` exists; it simply does
not carry the `has_` prefix the grep assumed, and the header that prompted the doubt is the stale
thing.

So: a grep shaped by an assumed naming convention, checked against a doc comment that was itself out
of date, produced a retraction of a true claim. Two unreliable instruments agreeing is not
corroboration. `[[feedback_validate_a_search_pattern_before_trusting_its_count]]` —
and the sharper form for next time: **when a grep and a comment agree against a claim, neither one
is the code.** Read the implementation.
