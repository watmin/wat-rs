# NOTE — ⛔ REFUTED: the re-cast's `holon::outcome` cut. All twelve are BINDING, not algebra.

> Found 2026-09-01 in pre-flight, before a stone was drawn.
> Refutes item 1 of `[[NOTE-partire-RECAST-on-the-current-runtime]]`. **Nothing was moved.**

## What was proposed

The re-cast's first module: extend `src/holon/outcome.rs` with twelve items from `runtime.rs` —
`PairedVectors` · `pair_values_to_vectors` · `cosine_outcome_from_values` · `presence_q_from_values` ·
`coincident_q_from_values` · `run_ast_arg_for_eval_coincident` · `coincident_of_two_values` ·
`eval_form_digest_coincident_shared` · `eval_form_signed_coincident_shared` · `FallbackVerdict` ·
`classify_fallback_outcome` · `dot_outcome_from_values`. 409 lines, rated Level 1, with the argument
that `src/holon/outcome.rs`'s own header calls itself *"Pure functions lifted out of `runtime.rs` per
Stone HOME-8"* — i.e. an unfinished migration, the same shape the numeric stone had just closed.

## ⛔ The refutation, from the home's own doctrine

`src/holon/mod.rs:8–52`, **The two-layer doctrine (Stone HOME-8)** — a *mechanical* signature test:

> *"a function taking `env: &Environment` and/or `sym: &SymbolTable` is **binding**: it reaches into
> the running program (lookups, calling user functions, provenance). **It lives in `runtime.rs`
> today** (a future strike turns it into a `#[wat_intrinsic]` shim under `src/intrinsic/holon/`)."*
>
> *"a function taking **neither** is **algebra**: pure computation over already-evaluated values. It
> lives here, in `src/holon/`."*
>
> *"nothing here reaches back into `runtime.rs`'s evaluator (`eval_inner`, `Environment`,
> `SymbolTable`) — **that boundary is the whole point**."*

Measured against the twelve:

```
pair_values_to_vectors               EvalBreak · SymbolTable · Value
cosine_outcome_from_values           EvalBreak · SymbolTable · Value
run_ast_arg_for_eval_coincident      Environment · EvalBreak · SymbolTable · Value · WatAST
eval_form_digest_coincident_shared   Environment · EvalBreak · SymbolTable · Value · WatAST
classify_fallback_outcome            EvalBreak · Value
dot_outcome_from_values              EvalBreak · SymbolTable · Value
```

**Every one carries `SymbolTable` or `Environment`.** They are binding by the doctrine's own test.
Moving them into `src/holon/` would violate the exact boundary that home exists to draw — and the
header sentence the cast quoted (*"pure functions lifted out"*) is the reason: what was lifted was
the **algebra half**. The binding half staying behind is not an unfinished migration. **It is the
migration's result.**

## ★ How the cast reached a wrong answer with correct evidence

It cited this doctrine — **in another module** — to refuse a different cut: `program_dim` /
`require_encoding_ctx` into `src/holon/`, *"blocked by `src/holon/codec.rs`'s documented stricter
two-layer contract."* So it read the rule, applied it once, and did not apply it to the twelve.

★★ The slip is a real one and worth naming: `codec.rs` is **stricter still** (no `WatAST`/`Value`/
`RuntimeError`/`Span` either). Reading "codec is stricter" as "the siblings are unconstrained"
inverts the doctrine — the `env`/`sym` line binds **all** of `src/holon/`; codec merely adds more.
`[[feedback_a_claims_support_does_not_travel_with_the_claim]]`

## What the twelve actually need — and it is a different campaign

The doctrine names their future explicitly: **a `#[wat_intrinsic]` shim under `src/intrinsic/holon/`**.
That is arc 255's homing shape (register the verb at an edge, body stays or moves behind it), not
arc 109's decomposition shape (relocate an impl into a domain home). They are not a megafile cut.

⬜ **Not drawn.** Whoever takes it should read it as a HOMING stone against `src/intrinsic/holon/`,
and should expect the bodies to stay in `runtime.rs` behind the shims — which buys correctness and
registry coverage, **not lines**.

## The standing correction to the map

`[[NOTE-partire-RECAST-on-the-current-runtime]]` item 1 is **struck**. Its remaining items —
`record` (shipped), the kernel family, the died-error cluster, `option`/`result`, the purity
classifier — are unaffected; each rests on its own evidence.

⚠ **And the general lesson for the rest of them:** a home may carry a *contract* that its line count
and its header sentence do not advertise. Before proposing a move INTO a home, read that home's
`mod.rs` for the rule it enforces — three of this campaign's homes now have one
(`src/holon/`'s two-layer split, `src/holon/codec.rs`'s stricter bar, and every `src/<domain>/`'s
"must not reference `crate::intrinsic`").
