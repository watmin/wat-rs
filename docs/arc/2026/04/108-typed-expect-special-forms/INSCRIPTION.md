# Arc 108 — typed `expect` as `:wat::core::*` special forms — INSCRIPTION

## Status

Shipped 2026-04-29 on the same day arc 107 landed.

## What this arc adds

Two substrate special forms in the `:wat::core::*` namespace, each
with explicit `-> :T` arm-result annotation matching the rest of
the branching family (`match`, `if`, `try`):

```scheme
(:wat::core::option::expect -> :T <opt-expr> <msg-expr>)
;; opt-expr  : :Option<T>
;; T         : declared arm-result type (must match Some-inner)
;; msg-expr  : :String
;; result    : T (Some-inner) or panic with msg

(:wat::core::result::expect -> :T <res-expr> <msg-expr>)
;; res-expr  : :Result<T, E>
;; T         : declared arm-result type (must match Ok-inner)
;; msg-expr  : :String
;; result    : T (Ok-inner) or panic with msg
```

## Why

Arc 107 shipped these as wat-level helpers in `:wat::std::*` —
the fastest path to closing proof_004's silent-disconnect-cascade
hang. Two follow-ups landed on review:

1. The user named the placement wrong. `expect` is a
   sum-type-discriminating branching construct (Some-arm vs
   None-arm; Ok-arm vs Err-arm). The branching family lives in
   `:wat::core::*` (match, if, try); `:wat::std::*` is for
   wat-level conveniences over substrate primitives. `expect` is a
   substrate concept.

2. `match`, `if`, and `try` all carry the explicit `-> :T`
   arm-result annotation. The wat-level helpers in arc 107
   relied on `let*`-binding inference — a function-call pattern
   that doesn't match the syntactic family.

Arc 108 corrects both. The wat-level helpers retire; the special
forms take their place.

## Slice 1 — substrate dispatch

- `src/check.rs::infer_option_expect` and `infer_result_expect`:
  arity-4 validation, `->` marker check, type-keyword parse,
  unification of `args[0]` against `:Option<T>` (or
  `:Result<T, fresh_E>`), unification of `args[3]` against
  `:String`, return type is the declared `T`. Mirrors `infer_if`
  for the `-> :T` parsing and `infer_try` for the unification
  shape. Dispatch arms wired next to `:wat::core::try` at
  `src/check.rs:462`.

- `src/runtime.rs::eval_option_expect` / `eval_result_expect`:
  evaluate the value expression; on success return the inner; on
  failure call the new helper `expect_panic` which evaluates the
  msg, snapshots the call stack, builds an `AssertionPayload` with
  the form's value-expression span as `location`, and
  `panic_any`s. Dispatch arms wired next to `:wat::core::try` at
  `src/runtime.rs:2189`.

- `src/runtime.rs::expect_panic` shared helper between the two
  forms — same panic discipline as
  `:wat::kernel::assertion-failed!` (which arc 107 made
  polymorphic for the Type T arm).

Verification by direct `wat <file>` runs:

| Path | Result |
|---|---|
| `Some 42 → expect -> :i64 "...should..."` | prints `42`, exits 0 |
| `:None → expect -> :i64 "broker disconnected"` | panic at line:col with that message, exits 2 |
| `Ok 99 → expect -> :i64 "...should..."` | prints `99`, exits 0 |
| `Err "rundb crashed" → expect -> :i64 "expected Ok value"` | panic at line:col with that message, exits 2 |

Panic location points at the value-expression's column inside the
form (e.g., `:12:35` for `(... opt -> :i64 "msg")` where `opt`
appears at column 35). Adequate — the form's outer paren is one
column to the left, and stack backtraces capture the surrounding
function frame.

## Slice 2 — retire the wat-level helpers

- `wat/std/option.wat` deleted.
- `wat/std/result.wat` deleted.
- `src/stdlib.rs` drops the two `WatSource` entries.

proof_004's drive-requests migrates to the new syntax (three
sites):

```scheme
;; before (arc 107)
((_ :())
 (:wat::std::option::expect
   (:wat::kernel::send cache-req-tx (Put k v))
   "drive-requests Put: cache-req-tx disconnected — cache thread died?"))

;; after (arc 108) — `-> :T` at HEAD position
((_ :())
 (:wat::core::option::expect -> :()
   (:wat::kernel::send cache-req-tx (Put k v))
   "drive-requests Put: cache-req-tx disconnected — cache thread died?"))
```

The `-> :()` declares the unwrap-target arm-result type at HEAD
position — declared BEFORE the value-producer (the
`:wat::kernel::send` call). For the recv-reply site the
annotation reads `-> :Option<wat::holon::HolonAST>`.

Tests at `wat-tests/core/option-expect.wat` (4 deftests:
Some-i64, Some-string, Some-nested-Option, None-panics-with-message)
and `wat-tests/core/result-expect.wat` (3 deftests: Ok-i64,
Ok-string, Err-panics-with-message). 7 deftests green under
`cargo test --release --test test`. The panic-path tests run
inside `:wat::test::run-ast` so the surrounding catch_unwind
captures the AssertionPayload as a `Failure` on the inner
RunResult; the outer deftest matches on `Failure/message` to
verify the supplied msg surfaced.

Re-run after migration:

```
running 6 tests
test 004-cache-telemetry.wat                  ... ok (52ms)
test 004-step-A-rundb-alone.wat               ... ok (45ms)
test 004-step-B-cache-alone.wat               ... ok (14ms)
test 004-step-C-both-null-reporter.wat        ... ok (43ms)
test 004-step-D-reporter-never-fires.wat      ... ok (57ms)
test 004-step-E-reporter-fires-once.wat       ... ok (52ms)
test result: ok. 6 passed; 0 failed; finished in 350ms
```

## Slice 3 — INSCRIPTION + USER-GUIDE + 058 row

- This file.
- `wat-rs/docs/USER-GUIDE.md` updates the expect entry: new
  syntax (`-> :T`), new namespace (`:wat::core::*`).
- 058 FOUNDATION-CHANGELOG row.

## What this arc does NOT do

- Does NOT change `:wat::kernel::assertion-failed!`'s polymorphic
  scheme (still `∀T. ... -> :T` from arc 107).
- Does NOT add a generic `:wat::core::expect` polymorphic over
  Option-vs-Result. Two siblings is the explicit shape; runtime
  dispatch is per-variant for clear panic-message semantics.
- Does NOT migrate other call sites that COULD use expect (e.g.,
  `Service/batch-log`, `Stream`'s ack loops). Each call site is a
  separate decision per author.

## The four questions (final)

**Obvious?** Yes — symmetric with `match`, `if`, `try`; reads as
"unwrap-or-panic with this message into this type." The `-> :T`
declares the unwrap target right at the call site.

**Simple?** Yes — two `infer_*` + two `eval_*` + one shared
`expect_panic` helper, each mirroring the established
`infer_try` / `eval_try` shapes. ~250 LOC including comments.
Two stdlib files retire (~20 LOC); net ~230 LOC.

**Honest?** Yes — special forms get explicit type annotations like
the rest of the branching family; `:wat::core::*` is the right
namespace for substrate-provided branching constructs.

**Good UX?** Yes — `-> :T` makes the unwrap target visible at
every call site without depending on the surrounding `let*`
binding to declare it. Error messages name the parameter (`opt`
or `res`, and `msg`) and the declared type directly.

## Cross-references

- `wat-rs/docs/arc/2026/04/107-option-result-expect/` — interim
  `:wat::std::*` shape replaced by this arc.
- `wat-rs/docs/arc/2026/04/028-*` (or wherever try lives) — the
  precedent special form that `option::expect` mirrors.
- `holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/` —
  migration target; all 6 deftests green post-migration.
- `holon-lab-trading/docs/proposals/2026/04/059-the-trader-on-substrate/059-001-l1-l2-caches/DEADLOCK-DIAGNOSIS-2026-04-29.md`
  — the diagnosis that drove arcs 107 + 108.
