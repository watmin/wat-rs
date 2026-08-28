# NOTE (arc 255) — the span lints cannot see generated code, and there is a third site living in that blind spot

**Filed 2026-08-28 at the close of Stone Q-2, by the rider that struck it.** A POINTER, not a
decision. STOP-4 of Q-2's brief forbade touching this site and correctly guessed it might be wrong;
the rider looked, confirmed it, and left it. This records what it found.

## The site

`crates/wat-macros/src/wat_intrinsic.rs`, inside `emit`'s non-variadic `value_door_body`: the
**generated** ALGEBRA value door raises its `ArityMismatch` at `::wat::rust_caller_span!()` —

```rust
if vals.len() != #n {
    return Err(RuntimeError::new(::wat::rust_caller_span!(), ArityMismatch { … }));
}
```

— while **the very door it is generating carries a `_span: &::wat::span::Span` parameter it never
reads.** Stone Q gave that door a real call span; this arm did not learn.

## Why neither lint catches it

Both lints exist for exactly this defect and both are blind to it, for two independent reasons:

1. **They scan `src/`.** This code lives in `crates/wat-macros/` and is emitted as tokens, so it
   never appears in a scanned file as itself.
2. **The spelling defeats the pattern anyway.** The lints match a bare `&Span`; the macro emits the
   fully-qualified `&::wat::span::Span`, because generated code cannot rely on the caller's imports.

★ **So the lints police hand-written code and are structurally blind to the code that is written 380
times over.** That is a bigger statement than one missed site: `#[wat_intrinsic]` is the substrate's
most-multiplied author, and no span discipline reaches it.

## Why it is a DIFFERENT site from the one Q-2 fixed

Q-2 fixed `dispatch_substrate_impl`'s central guard, which fires for verbs registered with a
**hand-written** `value = <path>` twin (19 of them). This one fires for the **~38 macro-generated**
ALGEBRA value doors. Same defect class, disjoint populations, and the sweep stones (O-iv-b onward)
are steadily moving verbs from the first population into the second — **so this site's blast radius
grows with every migration while the fixed one's shrinks.**

## What a stone here would have to answer

- **Does the generated arity check even need to raise?** After Q-2, `dispatch_substrate_impl`'s
  central guard checks arity before dispatching, so the generated one may now be unreachable through
  `apply`. It is still reachable through the AST door. Measure before assuming.
- **Can the lints reach generated code at all?** Scanning `crates/wat-macros/` for span discipline
  means reading `quote!` bodies — a different instrument, not a wider glob. A cheaper alternative is
  a runtime probe: call a generated ALGEBRA door at wrong arity through both doors and assert the
  reported location is the caller's.
- ⚠ **The second option is the one this arc keeps proving right.** A behavioural probe over a
  generated artifact beats a text scan of the generator, for the same reason the compiler beat three
  span classifiers in Stone H.
  `[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

## Refs

- `crates/wat-macros/src/wat_intrinsic.rs` — `emit`, the non-variadic `value_door_body` arm.
- `tests/lint/unused_span_justified.rs` · `tests/lint/span_substitution_justified.rs` — the two
  lints, and their `src/`-only scan.
- `BRIEF-STONE-Q-2-the-threaded-span-must-be-used.md` — STOP-4, which forbade touching it and asked
  for exactly this report.
