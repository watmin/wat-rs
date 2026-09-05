# DESIGN — STONE: `Span` equality becomes honest. Census run: the blast radius is TWO TESTS, and both are bugs it was hiding.

> **Builder, 2026-09-05:** *"fix the span eq before wat-fmt writes position assertions"*

Follows `[[NOTE-span-eq-is-vacuous-and-its-safety-claim-is-false]]`, which said the blast radius was
**not measured** and that the census was the first move of a stone rather than a step inside one.
**The census has now been run.** This document carries its result, so the stone is drawn against a
number instead of a fear.

## THE CENSUS — imposed, read, reverted

Made `Span::eq` compare `file`/`line`/`col`/`end`, added `PartialEq` to `Pos` (it had none either —
positions have never been comparable at all), gave `WatAST` a manual `PartialEq` that compares
structure and skips the span, built clean, ran the floor, reverted.

```
Summary  5170 tests run: 5168 passed, 2 FAILED, 17 skipped
```

**Two.** Both in `tests/diagnostics/`:

```
probe_arc243_stone3_typeerror_pattern_a::typeerror_span_access_is_single_path
probe_arc243_stone6_checkerror_pattern_a::checkerror_span_access_is_single_path
```

★ **And `src/runtime.rs:19751` PASSED** — `assert_eq!(span, outer_span, "outer-form span should
survive a rewrite")`. That assertion was vacuous, and its claim happens to be TRUE. Worth recording:
vacuity does not imply falsity, and the fix makes it a real gate for free.

## ⛔ THE TWO FAILURES ARE TEST BUGS THE VACUOUS EQUALITY WAS MASKING

```rust
// tests/diagnostics/probe_arc243_stone3_typeerror_pattern_a.rs:80-99
(
    TypeError::new(
        wat::rust_caller_span!(),      // line 83 — the span STORED in the error
        TypeErrorKind::CyclicSubtype { … },
    ),
    wat::rust_caller_span!(),          // line 89 — a DIFFERENT span, used as "expected"
),
```

**Two separate `rust_caller_span!()` invocations, at different lines.** They were never the same
span; the arm proves it:

```
left:  Span { …probe_arc243_stone3_typeerror_pattern_a.rs, line: 83, col: 17, end: None }
right: Span { …probe_arc243_stone3_typeerror_pattern_a.rs, line: 89, col: 13, end: None }
```

The test's stated point is *"Universal single-path access — works for EVERY TypeError regardless of
which kind variant. The whole point of Pattern A."* Under vacuous equality it proved only that
`err.span()` does not panic. **The fix makes it prove what it says**: capture ONE span into a
binding and use it on both sides.

## THE CHANGE

```
crates/wat-reader/src/span.rs   Span::eq compares file/line/col/end. Hash STAYS a no-op.
                                Pos gains PartialEq/Eq (it had neither).
crates/wat-reader/src/ast.rs    WatAST loses `#[derive(PartialEq)]` and gains a manual impl
                                over its 14 variants: compare structure, SKIP the span.
tests/diagnostics/ (×2)         bind one span, use it on both sides.
```

⭐ **The requirement the old blindness served is PRESERVED, and moved to where it always belonged.**
The module doc's reasoning — *"a parsed-at-runtime AST and a synthetic AST with the same structure
should compare equal regardless of where they came from"* — is about **`WatAST`**, and the manual
impl states it exactly. `Span` stops lying on behalf of a requirement that was never its.

⚠ **`Hash` stays a no-op, deliberately.** Rust's contract is `a == b ⟹ hash(a) == hash(b)`; the
converse is not required. A no-op span hash with an honest span eq satisfies it (unequal values may
collide). And `WatAST`'s hash must stay position-independent for `canonical_edn_wat`.

## THE FOUR QUESTIONS

| | Obvious? | Simple? | Honest? | Good UX? |
|---|:---:|:---:|:---:|:---:|
| **honest `Span::eq`, span-skipping `WatAST::eq`** | YES | YES | YES | YES |

- **Obvious? YES** — `Span::eq → true` surprises every reader; the module doc needs three paragraphs
  to defend it, and the defence contains a false claim.
- **Simple? YES** — one manual impl over 14 variants, mechanical, and it deletes a derive.
- **Honest? YES**, and it is the whole stone: *"it never compares Span values for equality"* was
  false at three sites, and two of them were asserting something untrue.
- **Good UX? YES** — wat-fmt is entirely about positions. Every `assert_eq!` on a span it writes
  starts meaning what it says, instead of silently passing.

## Scope

**In:** `Span::eq` · `Pos: PartialEq` · `WatAST`'s manual `PartialEq` · the two arc-243 tests ·
the `span.rs` module doc, whose *"never compares Span values"* claim must go.

**Out, affirmatively:** `Hash` (stays a no-op, and the reason is written above) ·
`src/runtime.rs:19751`, which passes as-is and becomes a real gate for free ·
`tests/value/probe_runtime_error_one_door.rs:38`'s `Debug` workaround — it is correct and can now be
simplified, but its stale comment (*"`Span` doesn't derive `PartialEq`"* — it does) is the tenth
comment-caused defect of this campaign and fixing the comment is IN, changing the code is OUT.
