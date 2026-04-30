# Arc 108 — typed `expect` as `:wat::core::*` special forms

**Status:** in flight (2026-04-29)
**Predecessor:** arc 107 (interim wat-level helpers in `:wat::std::*`).

## The finding driving this arc

Arc 107 shipped `:wat::std::option::expect` / `:wat::std::result::expect`
as wat-level functions. The cure works — proof_004 closes the
silent-disconnect-cascade-hang class. But the user named the
namespace placement wrong and the syntax incomplete:

> "expect looks like a match expr.. it should be typed?.. like
> match and if?..."
> "this is :wat::core::* / this is not part of std"

`expect` performs sum-type discrimination — Some-arm vs None-arm,
Ok-arm vs Err-arm — same shape as `match` (which takes `-> :T`)
and `if` (`-> :T`) and `try` (already a special form). The other
branching constructs all live in `:wat::core::*` and all carry
explicit arm-result type annotations. Arc 107's wat-level helpers
break both invariants.

## The cure

Promote the helpers to **special forms** in `:wat::core::*`
namespace, with explicit `-> :T` arm-result annotation matching the
rest of the branching family.

### Syntax

```scheme
(:wat::core::option::expect -> :T <opt-expr> <msg-expr>)
;; opt-expr  : :Option<T>
;; T         : declared arm-result type (must match the value carried by Some)
;; msg-expr  : :String

(:wat::core::result::expect -> :T <res-expr> <msg-expr>)
;; res-expr  : :Result<T, E>
;; T         : declared arm-result type (must match the Ok-inner)
;; msg-expr  : :String
```

Both panic on the failure variant (`:None` / `Err`) with the
caller-supplied message; both return the Some/Ok inner on success.

### Why head-position `-> :T`

`if` and `match` put `-> :T` AFTER the first arg (cond / scrutinee)
because the first arg is a dispatch-determiner that does NOT
itself produce the result — `then`/`else` and arm bodies do. The
type lands between the determiner and the producers.

In `expect`, the value expression IS a producer (the Some-/Ok-arm
yields its inner). The honest position for `-> :T` is HEAD —
declared before any value producer — so the form reads: "declare
result T; derive it from this value or panic with this message."
Putting the type after the value-expression would mirror `if`'s
shape but lose the semantic invariant that `-> :T` precedes
producers.

### Why special forms

- **Symmetry with the family.** `match`, `if`, `try` are special
  forms in `:wat::core::*` and all carry `-> :T` for the
  arm-result type. `expect` is the same kind of construct.
- **Namespace honesty.** `:wat::core::*` is the substrate's own
  namespace; substrate-provided branching constructs live there.
  `:wat::std::*` is for wat-level conveniences over substrate
  primitives. `expect` is a substrate concept.
- **Better error messages.** The checker can report
  `expected :T (declared) | got <Some-inner-type>` at the right
  parameter rather than relying on inference from the `let*`
  binding context.
- **Source-location fidelity.** The runtime's panic carries the
  span of the `expect` form itself (via `WatAST::span` on the
  outer paren); the panic message points at the call site of
  expect rather than depending on the user-function frame stack.

## Stepping stones

### Slice 1 — substrate dispatch

- `src/check.rs`:
  - `infer_option_expect(args)` mirrors `infer_if`: validates
    arity 4, validates `args[1]` is `->`, parses `args[2]` as a
    type keyword `:T`, infers `args[0]` and unifies with
    `:Option<T>`, infers `args[3]` and unifies with `:String`,
    returns `T`.
  - `infer_result_expect(args)` similar, with `:Result<T, fresh_E>`
    on `args[0]`.
  - Dispatch arms in `infer` next to `:wat::core::try`.

- `src/runtime.rs`:
  - `eval_option_expect(args, env, sym)`: eval `args[0]` to a
    `Value::Option`; on `Some(v)` return v; on `None` eval
    `args[3]` to a String, snapshot the call stack, build an
    `AssertionPayload` with the form's span, `panic_any(payload)`.
  - `eval_result_expect(args, env, sym)` similar.
  - Dispatch arms in `eval_call`'s match next to `:wat::core::try`.

### Slice 2 — retire the wat-level helpers

- `wat/std/option.wat` deleted.
- `wat/std/result.wat` deleted.
- `src/stdlib.rs` drops the two `WatSource` entries.
- proof_004's drive-requests migrates to the new syntax:

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
  position. For the recv-reply site the annotation reads
  `-> :Option<wat::holon::HolonAST>`.

### Slice 3 — INSCRIPTION + USER-GUIDE + 058 row

- `wat-rs/docs/arc/2026/04/108-*/INSCRIPTION.md`.
- `wat-rs/docs/USER-GUIDE.md` updates the expect entry with the
  new syntax.
- 058 FOUNDATION-CHANGELOG row.

## What this arc does NOT do

- Does NOT change `assertion-failed!`'s polymorphic scheme — that's
  still `∀T. ... -> :T` per arc 107.
- Does NOT add a generic `:wat::core::expect` that polymorphically
  dispatches over Option vs Result. Two siblings (one per type)
  is the explicit shape; runtime dispatch is per-variant.
- Does NOT migrate other call sites that COULD use expect (e.g.,
  Service/batch-log, Stream's ack loops). Each call site is a
  separate decision per author.

## The four questions (final)

**Obvious?** Yes — symmetric with match / if / try; reads as
"unwrap-or-panic with this message into this type."

**Simple?** Yes — two `infer_*` + two `eval_*` Rust functions, each
mirroring `infer_try` + `eval_try`'s shape and roughly the same
length.

**Honest?** Yes — special forms get explicit type annotations like
the rest of the branching family; `:wat::core::*` is the right
namespace for substrate-provided branching.

**Good UX?** Yes — `-> :T` makes the unwrap target visible at the
call site without relying on the surrounding let* binding to declare
it; error messages name the parameter and the declared type
directly.

## Cross-references

- `wat-rs/docs/arc/2026/04/107-option-result-expect/` — the interim
  `:wat::std::*` shape this arc replaces.
- `wat-rs/docs/arc/2026/04/028-*` (or wherever try lives) — the
  precedent special form that `option::expect` mirrors.
- `holon-lab-trading/wat-tests-integ/proof/004-cache-telemetry/` —
  the migration target.
