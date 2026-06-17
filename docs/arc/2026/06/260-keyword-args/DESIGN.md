# Arc 260 — keyword / named arguments (the form should say what it means)

> **STATUS: STUB — queued, non-blocking.** Surfaced 2026-06-12 by the builder during the
> `deftest'` build (arc 259 S3.5a). Not started. This file is the context breadcrumb so the
> arc can be picked up cold.

## The trigger

Writing a probe, the form `(:wat::kernel::assertion-failed! "msg" :wat::core::None :wat::core::None)`
stopped the builder cold:

> *"i've been staring at the none/none and some/some without any sigils as to wtf they are…
> can we make those :kwargs or something? :reason :expected :actual — or whatever, i legit
> don't know the ordering."*

That last clause is the whole bug: **a competent reader cannot tell, from the form, what the
trailing positional `:None :None` mean — nor in which order.** The form refuses to carry its own
meaning. That is the exact failure the substrate exists to abolish (legibility-by-design; the
form and the prose on one channel — see the recenter interlude / #93).

## Grounded facts

`assertion.rs:54-57`:

```rust
pub struct AssertionPayload {
    pub message:  String,
    pub actual:   Option<String>,
    pub expected: Option<String>,
    // … location, frames, chain (internal)
}
```

So `(:wat::kernel::assertion-failed! <message> <arg2> <arg3>)` is **`(message, actual, expected)`**
— and `actual`-before-`expected` is precisely the order a writer flips by accident. Every
`… :None :None` / `… (:Some a) (:Some b)` call site is illegible and order-fragile.

Call sites today: `assertion-failed!` is called from `wat/test.wat` (the assert helpers) and is
the panic primitive every `assert-*` lands on. The opacity is widespread but low-stakes
(non-blocking).

## The reach, and the deeper question

The builder reached for **keyword args** — `:actual …`, `:expected …`. The load-bearing
question this arc must answer first:

> **Does wat have keyword / named arguments? If not, that is the real reach-stumble** — the
> substrate is missing a tool an LLM (and a human) instinctively reaches for, and the fix is to
> make it, not to nest a workaround.

Grep `keyword`/`kwarg`/named-arg support in the parser + checker before designing; do NOT assume.

## Design space (to weigh with the four questions when the arc opens)

1. **Keyword arguments** — `(assertion-failed! :message "…" :actual … :expected …)`. The general
   capability; clojure-idiomatic (`& {:keys […]}` / maps-as-kwargs). Biggest surface, widest payoff
   (labels every call site in the language, not just this one). If wat lacks them, this is a real
   language feature with its own type story (typed kwargs are non-trivial).
2. **A record argument** — `(assertion-failed! "msg" (AssertionDetail :actual … :expected …))`.
   Reuses the EDN-native record surface (arc 257); no new call-arg machinery. The labels live in the
   record's field names. Narrower; doesn't generalize to other verbs.
3. **Named-positional via a labeled enum / tuple** — weakest; still positional under the hood.

Lean (un-grounded, for the future self to re-decide): if wat genuinely lacks keyword args, the
record-arg (option 2) is the cheap honest fix for `assertion-failed!` *now*, and "keyword args in
wat" is its own larger arc to weigh on its own merits. Don't conflate the two unless grounding
shows kwargs are cheap.

## Scope / discipline

- **Non-blocking.** It breaks nothing and fixes nothing functional; it is a legibility/ergonomics
  debt. Do it when the velocity allows, not as a dependency.
- When opened: ground the kwargs question FIRST (parser/checker), then four-questions the design
  space, then a disconfirming probe, then build through a shadowdancer + weigh (the arc-259 rhythm).
- Pairs the prose-and-form thesis: a call form that needs a comment to decode its own arguments is
  the thing the comm channel exists to kill.

## RELATED — the macro-time sibling (the kwargs pattern already exists, inlined)

This arc is RUNTIME call-site keyword args. Its sibling is **macro-time options parsing**, and that
pattern already shipped inlined in `defservice` (2026-06-16, arc-272 rs-1, `wat/service.wat` ~line 67):
a variadic defmacro takes trailing `[:key val :key val …]`, folds them into a HashMap in one pass
against a `known-opts` set (rejecting any unknown key with a named macro-error), then reads each option
as `(HashMap/get opts-map "<key>")` with a default. Adding an option = one `known-opts` entry + one
`get`. This IS the canonical "macro with optional keyword-options" shape (Clojure `& {:keys}`, CL
`&key`, Python `**kwargs`).

**Why it's inlined, not extracted (yet):** the macro-eval fence CANNOT call user-defined wat fns
(`feedback_does_a_macro_need_it_intrinsic_boundary`), so a shared `parse-macro-opts` helper would have to
be a **Rust intrinsic** on the macro-eval allow-list (the "does a macro need it → intrinsic" boundary),
or this arc's facility generalized to expand-time. Per rule-of-three: defservice is the ONLY consumer
today; extract when a SECOND option-taking macro appears (generalizing from N=1 risks baking in
defservice's specifics). When this arc is built, weigh folding the macro-time pattern in (or minting the
intrinsic) so the two layers share one kwargs story.
