# EXPECTATIONS — a refusal that names a Rust file is a refusal aimed at the wrong person

> Written **before** the strike. **Every row's command was run against HEAD and its pre-value
> recorded.**

## ⛔ NO PINNED TEST COUNT

**The floor must be ≥ 5,219 plus every arm you drive. Exceeding it is a PASS.**

## The scorecard, with pre-values measured at HEAD `30f78ebdf`

| # | what | pre-value AT HEAD | expected after |
|---|---|---|---|
| 1 | the refusal's span | `rust_caller_span!()` at both sites — a `src/rete/kernel/fire/*.rs` line | the caller's `list_span`; a probe asserts the span names a **`.wat`** file |
| 2 | ★ the site is now GATED | the lint cannot see either fn (**no span param**) | re-introduce `rust_caller_span!()` → `span_substitution_justified` **REDDENS** |
| 3 | every real caller had one | `eval_fire_rules_native`, `eval_fire_once_native`, `eval_fire_rules_explain` all take `list_span: &Span` (measured) | all three pass it |
| 4 | call sites updated | `fire_rules_on_session` has **9** callers; `fire_once_session` **1** (measured) | all of them, and **report the count you find** |
| 5 | tests need no rune | — | synthetic spans in the 8 test callers, unruned — they are leaves |
| 6 | the lint is NOT widened | — | `tests/lint/span_substitution_justified.rs` gains only a **doc** note recording the proxy + the 534/71 numbers |
| 7 | blast radius | — | 3 fire files + 5 test modules + the lint's doc. **`validate/typing.rs` absent** (E1 is the next strike) |
| 8 | lints | **116/116** (measured) | green |
| 9 | floor | **5219/5219** (measured) | ≥ 5,219 + every new arm, zero FAIL rows |
| 10 | clippy | **rc=0** (measured) | silent |

## The mutation proofs — two arms, two mechanisms

1. **★ The gate now sees it.** Re-introduce `rust_caller_span!()` in one threaded body; the
   **existing** lint must redden and name the site. *This is the strike's structural claim; row 1
   alone would pass on a fix that merely swapped a value.*
2. **The span actually reaches the user.** Replace the threaded `span` with
   `rust_caller_span!()` at the `refuse_export_without_arm` call only; row 1's probe must redden with
   a `src/…rs` path.

Per arm: **proven** / **reachable but not driven** / **not reachable, and why**.

## Runtime prediction

40–55 minutes. The threading is mechanical; the nine call sites and mutation 1 are the work.

## What would make this strike a failure even if every test passes

**A fix that swaps the value without adding the parameter** — e.g. passing the span through a
thread-local, or re-spanning the error at the boundary. Row 1 would go green and the site would stay
invisible to the lint, so the next `rust_caller_span!()` written there is unguarded. **The parameter
is the point**; row 2 exists for exactly this and row 1 cannot see it.

The second: **widening the lint.** 534 sites tree-wide are span-less fns stamping
`rust_caller_span!()`, and the exclusion protecting them is principled. Row 6 checks the lint gains
a doc note and nothing else.
