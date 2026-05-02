# Arc 140 — Sandbox-scope leak: panic with a teaching diagnostic

**Status:** opened 2026-05-03. **Active.**

## TL;DR

When a deftest body invokes a name that exists at the OUTER test-file
scope but NOT in the deftest's prelude, the runtime currently panics
with `unknown function: :foo` — a generic message that doesn't
explain why `:foo` was unreachable. The user reaches for the wrong
fix (re-typing the name, checking the spelling, hunting through
imports) instead of the right one (moving the define into the
deftest's prelude).

This arc makes the failure self-teaching. At the moment the
sandbox's runtime cannot resolve a call head, if the name is
visible in the OUTER scope, the panic says: *"sandbox-scope leak —
`:foo` is defined in <outer-file> but deftest sandboxes don't
capture outer-scope; move `:foo` into this deftest's prelude (or
load it via the prelude)."*

## The problem — why this has burned us many times

A `(:wat::test::deftest <name> <prelude> <body>)` expands (per
`wat/test.wat`) into:

```scheme
(:wat::core::define (<name> -> :wat::test::TestResult)
  (:wat::kernel::run-sandboxed-ast
    (:wat::core::forms
      ,@<prelude>
      (:wat::core::define (:user::main ...) <body>))
    ...))
```

The `run-sandboxed-ast` primitive starts a fresh sub-program from
those AST forms. The sub-program has its OWN startup pipeline and
its OWN symbol table — it does NOT inherit the outer file's user
defines. **This is intentional** (sandbox isolation) and was reaffirmed
in user direction 2026-05-03:

> *"we intentionally do not capture stuff out of the prelude.. if
> they are reaching for something not in a prelude.. that's a
> problem"*

But the failure mode is silent. Authoring pattern:

```scheme
;; tmp-mytest.wat

;; The user puts a helper at the top of the file.
(:wat::core::define
  (:my::helper (x :wat::core::i64) -> :wat::core::i64)
  (:wat::core::i64::* x 2))

;; Then writes a deftest that invokes the helper.
(:wat::test::deftest :tmp::test
  ()                            ; ← empty prelude
  (:wat::test::assert-eq (:my::helper 21) 42))
```

At outer freeze, `:my::helper` registers at the top level. The
deftest body inside `(:wat::core::forms ...)` is *AST data* — it
doesn't get type-checked as code at the outer freeze. At runtime,
`:tmp::test` calls `run-sandboxed-ast`. The sub-program freezes
from the prelude (empty) plus auto-generated `:user::main`.
`:my::helper` is NOT registered in the sub-program. The body
runs, invokes `:my::helper`, runtime fires
`RuntimeError::UnknownFunction`. The test panics with:

```
test tmp-mytest.wat :: tmp::test
  failure: unknown function: :my::helper
```

The user sees "unknown function" and assumes the spelling is wrong,
the file isn't loaded, the dep wasn't wired, etc. They DON'T see
"sandbox scoping". The bug class has burned the project repeatedly.

## The fix

Two layers, both useful — runtime panic enrichment AND a
compile-time check rule.

### Layer 1 — runtime panic enrichment

When `apply_function` fires `RuntimeError::UnknownFunction(name)`
inside a sub-program (one started by `run-sandboxed-ast` /
`run-sandboxed-hermetic-ast` / `spawn-program-ast` /
`fork-program-ast`), the sub-program has access to the OUTER world's
`SymbolTable` (passed through as `outer_symbols` or
`parent_symbols`). The runtime checks: is `name` (or its canonical
form, stripping `<T>` per arc 139's logic) in the outer symbols?

If YES → fire a richer error WITH BOTH COORDINATES:
```
<call_site_span>: sandbox-scope leak: :my::helper invoked here
is defined at <outer_define_span> but deftest sandboxes don't
capture outer-scope. Move (:wat::core::define :my::helper ...)
into this deftest's prelude — the second argument of
(:wat::test::deftest <name> <prelude> <body>).
```

The diagnostic carries TWO spans:
- `<call_site_span>` — file:line:col of the offending invocation
  inside the deftest body. The user clicks this to land on the
  bad call.
- `<outer_define_span>` — file:line:col of the
  `(:wat::core::define :my::helper ...)` form at the outer scope.
  The user clicks this to land on the helper they need to move
  (or load from the prelude).

Without both spans the diagnostic is half-useful — the user still
has to grep. Per arc 138's principle: errors carry coordinates;
agents and humans navigate, never grep. This arc's diagnostic IS
that principle applied to a teaching error.

If NO → fire the existing `unknown function: :my::helper` (genuinely
unknown name; user typo or missing dep). That message also gains a
span via arc 138's RuntimeError sweep.

### Layer 2 — compile-time check rule

`CheckError::SandboxScopeLeak`. When the outer freeze type-checks a
`run-sandboxed-ast`-style call, walk the inner `(:wat::core::forms
...)` block — collect names DEFINED inside (prelude defines + the
auto-generated `:user::main`). Then walk each form's body — for
every call head not resolvable WITHIN the inner scope, check the
OUTER scope. If found there: fire `CheckError::SandboxScopeLeak`
with the same teaching message. If not found anywhere: fire
existing `UnresolvedReference` (genuinely typo).

This catches the bug at startup, before runtime, before the user
ever sees a "panicked at" trace. Same precedent as arc 117
(`ScopeDeadlock`), arc 126 (`ChannelPairDeadlock`), arc 130
(`MalformedVariant`) — substrate-as-teacher discipline.

## Slice plan

1. **Slice 1 — runtime panic enrichment.** Modify
   `apply_function` (or the spawn/sandbox primitives) to thread the
   outer `SymbolTable` reference through. On UnknownFunction
   inside a sandbox, check the outer table. If found, replace the
   error with a `RuntimeError::SandboxScopeLeak { name, outer_file }`
   variant. **Test**: write `wat-tests/tmp-sandbox-leak.wat` —
   define a helper at top level, deftest invokes it without
   prelude. Expect the new teaching panic.
2. **Slice 2 — compile-time check rule.** Add
   `CheckError::SandboxScopeLeak { offending_name, call_span,
   outer_define_span }` variant. Walk inner forms-blocks in
   arc-128's sandbox-aware walker. Fire the check error before
   runtime. Display arm prefixes the call_span; the message body
   embeds the outer_define_span. **Test**: same scenario; freeze
   should reject before runtime fires; both spans appear in the
   error rendering.
3. **Slice 3 — INSCRIPTION + USER-GUIDE update + 058 row.**

## Precedents

- **Arc 102 (eval-ast! polymorphic return)** — closest structural
  analog: substrate primitive's scheme claimed polymorphism the
  runtime didn't honor; fixed by aligning. Same shape: substrate
  is internally inconsistent about a contract; user trips over the
  inconsistency; substrate fixes the gap.
- **Arc 117 (ScopeDeadlock prevention)** — same discipline. A
  pattern that reliably burned users at runtime got a check-time
  rule with a teaching diagnostic.
- **Arc 124 (hermetic + alias deftest discovery)** — the deftest
  discovery silently dropped three valid deftest shapes. Same
  pattern: a substrate behavior was silently doing the wrong
  thing; lab usage exposed it; substrate was extended to handle
  the shapes properly.
- **Arc 128 (check-walker sandbox boundary)** — established that
  the outer walker stops at `run-sandboxed-ast`'s forms-block. Arc
  140 leverages the same boundary primitive but USES the outer
  scope for the teaching cross-reference.
- **Arc 116 (phenomenal cargo debugging)** — same teaching
  discipline at the failure rendering layer. Failures got
  structured + walked; this arc adds another structured failure
  type to the same surface.

## Coordinates (arc 138 dependency)

This arc layers on arc 138's spans-on-errors discipline:

- The new `CheckError::SandboxScopeLeak` variant carries
  `call_span: Span` + `outer_define_span: Span` — both arc-138-style.
- The new `RuntimeError::SandboxScopeLeak` variant carries the same.
- The runtime threading needed for slice 1 inherits arc 138
  slice 3's RuntimeError span work.

Arc 140 doesn't BLOCK on arc 138's full sweep — slice 1's runtime
enrichment can land first with span: Span::unknown() everywhere
the substrate is still mid-138-sweep. As arc 138 fills in spans,
arc 140's diagnostic gets sharper automatically. But the slice 2
check rule's two-span shape requires arc 138's CheckError span
infrastructure to be in place (which it now IS — arc 138 slice 1
shipped the CheckError span fields).

## Done when

- A deftest with an empty prelude that invokes a top-level outer
  define fires the teaching error AT FREEZE TIME (slice 2's
  check rule), with file:line:col on BOTH the call site and the
  outer-scope define.
- The same scenario without slice 2 (just slice 1) fires the
  teaching error AT RUNTIME (no longer the generic "unknown
  function"), with the same two spans.
- `wat-tests/tmp-sandbox-leak.wat` proves both layers — a SHOULD-PANIC
  deftest whose expected message includes the call-site path and
  the outer-define path.
- `cargo test --release --workspace` green.
- INSCRIPTION + USER-GUIDE row + 058 changelog.

## What this is NOT

- **Not a sandbox-removal arc.** Sandbox isolation IS the
  design — preluges define what's visible inside; outer scope is
  intentionally invisible. This arc keeps that semantic and only
  improves the failure diagnostic.
- **Not a behavior change.** Existing deftests that use prelude
  correctly continue to work. The new error fires only on the
  failure path that currently produces a generic message.

## Cross-references

- `wat/test.wat` line 304 — deftest macro definition.
- `src/test_runner.rs` line 463 — runtime test entry.
- `src/spawn.rs` line 87 — `eval_kernel_spawn_program_ast`.
- `src/freeze.rs` line 477 — sub-program freeze pipeline.
- `docs/arc/2026/05/128-check-walker-sandbox-boundary/INSCRIPTION.md`
  — sandbox-boundary precedent.
- `docs/arc/2026/04/102-eval-ast-polymorphic-return/INSCRIPTION.md`
  — polymorphism scheme/runtime alignment precedent.
- `docs/arc/2026/05/139-generic-tuple-return/DESIGN.md` — sibling
  arc; turbofish at user-define call sites doesn't strip; same
  asymmetric registration/lookup class.
