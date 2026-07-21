# NOTE (arc 109 cleanup) — a scope-qualified name reaches `Identifier::bare` in closure re-emit (debug-only assert)

**Filed 2026-07-20 (surfaced mid arc-278 #16, running the floor in DEBUG by mistake).**
Queued, NOT built — it is **release-correct** (see below), so it does not dent the
zero-in-release floor; a low-priority cleanup, arc-109-style. Named-not-lost.

## The finding (grounded)

Running `cargo nextest run` **in debug** (the floor is normally `--release` — the arc
convention, `DESIGN-no-hidden-failures.md:318`), `probe_arc170_gapj_each_kwargs`
(`tests/services/`, the `each`+kwargs tail proof) panics:

```
crates/wat-reader/src/identifier.rs:101:9:
Identifier name must not contain U+0001 (env-key separator); got "kwargs\u{1}713"
```

Backtrace: `Identifier::bare("kwargs\u{1}713")` ← `function_to_define_form_with_body`
(`src/closure_extract.rs:2746`) ← `extract_closure` (`:413`) ← `eval_kernel_fn_forms`
(`:530`).

`closure_extract.rs:2746` re-emits each `Function` param name as a binder symbol via
`Identifier::bare(param.clone())`. For the `each`/kwargs path the param name is *already
scope-qualified* — `kwargs` + `\u{1}` (the env-key **scope separator**) + `713` (the
scope id) — because the kwargs-provisioning layer scoped the `kwargs` binding.
`Identifier::bare` is the constructor for **un-scoped** names; its arc-249 debug assert
(`identifier.rs:101`, added `447a0590`, 2026-06-05) exists precisely to catch a
scope-qualified/env-key-encoded name arriving via a non-lexer route — and it is doing
its job here.

## Why it is not a floor failure (release-correct)

The assert is `debug_assert!` — compiled OUT in release. In `--release`, `each_kwargs`
does not merely "pass because the assert is gone": it **asserts real behavior** (every
one of 5 items incremented exactly once, final durable count == 5) and that passes. So
the scope-qualified name round-trips correctly at runtime; the encoding is used as
designed. This is a **debug-strictness inconsistency, not a runtime bug**.

Dates (none new): construction site `closure_extract.rs:2746` — arc 241, `7244cf43`
(2026-05-29); the assert — arc 249, `447a0590` (2026-06-05); the test that trips it —
arc 294 item 9a, `0181901a` (2026-07-12). So it has been catchable-in-debug for ~1 week,
invisible in the release-only workflow.

## The fix (when it comes)

At `closure_extract.rs:2746` (and the sibling `:2754` rest-param), a param that may be
scope-qualified should be reconstructed with the **scope-aware** `Identifier` path (split
the env-key back into `name` + `scopes`, or carry the `Identifier` through instead of
`param: String`), rather than `Identifier::bare`, which by contract takes a bare name.
Alternatively, ensure `Function.params` holds bare names at this point and the scope
rides separately. Either closes the debug assert honestly without changing release
behavior. Small, localized; no floor risk.
