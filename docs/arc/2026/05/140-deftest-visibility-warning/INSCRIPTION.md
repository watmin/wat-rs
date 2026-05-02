# Arc 140 — INSCRIPTION

## Status

**Shipped + closed 2026-05-03.** Three slices over the same day:

- **Slice 2** — compile-time check rule. `CheckError::SandboxScopeLeak`
  variant + `validate_sandbox_scope_leak` walker in `src/check.rs`.
  2 unit tests. Commit `4943805`.
- **Slice 1** — runtime panic enrichment.
  `RuntimeError::SandboxScopeLeak` variant +
  `outer_symbols: Option<Arc<SymbolTable>>` field on `SymbolTable`,
  set by `eval_kernel_spawn_program_ast` in `src/spawn.rs`. 3 unit
  tests. Commit `d422ef5`.
- **Slice 3** — this INSCRIPTION + USER-GUIDE addition + 058 row.

Slice ordering is intentional: slice 2 (compile-time) shipped before
slice 1 (runtime) because the compile-time path is the user-facing
load-bearing fix. Slice 1 is defense-in-depth for runtime / dynamic-
forms paths the static walker can't see. Both layers are on; both
fire with the same two-span teaching diagnostic.

The arc surfaced the discipline `feedback_no_known_defect_left_unfixed`
(memory): when we know how to surface a failure RIGHT NOW, do it
now. Slice 1's "covers most cases — defer slice 1" framing was
deferral-by-rationalization and was corrected mid-arc.

## What this arc adds

Two failure-class diagnostics that catch the same bug at two
different points in the pipeline:

| Layer | Variant | Fires |
|---|---|---|
| Compile-time | `CheckError::SandboxScopeLeak` | At outer freeze; the walker descends into every `(:wat::kernel::run-sandboxed-ast ...)` / sibling sandbox primitive's forms-block, builds the inner-scope name set (defines stripped of `<T,...>`), and walks each inner form's body. Fires when a call head misses inner scope but resolves in the outer `SymbolTable`. |
| Runtime | `RuntimeError::SandboxScopeLeak` | At sub-program runtime; when `apply` hits an `UnknownFunction` AND the canonical name (sans `<T,...>`) resolves in the outer `SymbolTable` reachable via `sym.outer_symbols`. |

Both carry two spans:
- `call_span` — file:line:col of the offending invocation inside the
  sandboxed body.
- `outer_define_span` — file:line:col of the outer-scope define
  (best-effort: the function's body span).

The diagnostic message is identical in shape across layers:

```
<call_span>: sandbox-scope leak: ':my::helper' invoked here is
defined at <outer_define_span> but deftest sandboxes do NOT capture
outer-scope. Move (:wat::core::define :my::helper ...) into this
deftest's prelude (the second argument of `(:wat::test::deftest
<name> <prelude> <body>)`), or load it into the prelude via
`(:wat::core::load! "path/to/file.wat")`. Sandbox isolation is
intentional — see wat/test.wat's deftest macro.
```

## Why

The deftest macro (defined in `wat/test.wat`) expands its body into
a `:user::main` wrapped in `:wat::kernel::run-sandboxed-ast`. The
sub-program's symbol table contains ONLY the prelude's defines plus
stdlib — outer-file user defines are NOT captured. **This is
intentional** (sandbox isolation; preserved across this arc).

The bug class that has burned the project repeatedly: a user puts a
helper at the top of their test file, references it from a deftest
body. Outer freeze type-checks against the OUTER scope (the helper
is visible there); pass. Sub-program freeze runs with the restricted
scope; resolve / runtime fires `unknown function: :my::helper`.
Generic message; no scoping explanation; no coordinates. The user
hunts for spelling, dep wiring, missing imports — none of which are
the actual issue.

User direction (2026-05-03):

> *"this has burned us many times — how can we make users /in
> deftest/ be told this happening.... new arc — make it and prove
> it works"*
>
> *"panic when they invoke a form that's not invokable... it's a
> scoping problem. we intentionally do not capture stuff out of the
> prelude.. if they are reaching for something not in a prelude..
> that's a problem"*

The arc preserves sandbox isolation while making the failure
self-teaching at both possible discovery points (compile and
runtime).

## Implementation

### Slice 2 — compile-time check rule

`src/check.rs::validate_sandbox_scope_leak`:

1. Recurse every form looking for sandbox-primitive call sites.
2. At each: extract `arg[0]` (the `(:wat::core::forms ...)` block).
3. Build `inner_names: HashSet<String>` from defines inside the
   forms-block (stripping `<T,...>` via `name.find('<')`).
4. Walk each inner form's body. For each call head:
   - skip reserved-prefix (`:wat::*`, `:rust::*`)
   - skip if in `inner_names`
   - else: if `sym.get(canonical).is_some()` in outer scope → fire
     `SandboxScopeLeak` with both spans.
5. Stop at nested sandbox boundaries — outer caller's recursion
   handles those.

Sandbox-primitive heads recognized:
- `:wat::kernel::run-sandboxed-ast`
- `:wat::kernel::run-sandboxed-hermetic-ast`
- `:wat::kernel::fork-program-ast`
- `:wat::kernel::spawn-program-ast`

Wired into `check_program` after arc 126's pair-deadlock check; runs
on every form + every function body.

### Slice 1 — runtime panic enrichment

`src/runtime.rs::SymbolTable.outer_symbols: Option<Arc<SymbolTable>>`
— set by the spawn driver
(`spawn::eval_kernel_spawn_program_ast`) AFTER the inner FrozenWorld
is constructed and BEFORE the spawn thread starts. Only the failure
path consults it; sandbox isolation stays intact for every success
path.

At `runtime.rs`'s user-defined function dispatch site (line ~2862),
a missed inner lookup now:
1. Strips `<T,...>` from the head keyword.
2. Checks `sym.outer_symbols` for the canonical name.
3. If found → fires `RuntimeError::SandboxScopeLeak` with both spans
   (call_span from the call form; outer_define_span from the outer
   function's `body.span()`).
4. Else → falls through to the existing `UnknownFunction` (genuine
   typo / missing dep).

## Tests

- `check::tests::sandbox_scope_leak_fires_with_diagnostic` — leak
  case fires at outer freeze. Asserts variant, message contains
  `sandbox-scope leak`, offending name, file:line:col, and `prelude`
  teaching word.
- `check::tests::sandbox_scope_no_leak_when_in_prelude` — clean case;
  helper IS in prelude; assert no misfire.
- `runtime::tests::runtime_sandbox_scope_leak_fires_with_outer_attached`
  — slice 1 runtime path; outer has the helper, inner doesn't,
  outer_symbols set; assert SandboxScopeLeak fires.
- `runtime::tests::runtime_unknown_function_when_outer_also_missing`
  — outer also missing; assert UnknownFunction fires (no misfire).
- `runtime::tests::runtime_no_leak_when_outer_not_attached` — entry
  program (no outer attached); assert UnknownFunction fires.

`cargo test --release --workspace`: 765 lib + 174 integration tests,
0 failures.

## Limitations

- `outer_define_span` uses the outer function's `body.span()` as a
  best-effort proxy. The `Function` struct doesn't carry the define
  form's span directly; a follow-up could thread that through if
  navigation precision matters.
- Runtime backstop only fires when the spawn path attaches
  `outer_symbols` (currently `eval_kernel_spawn_program_ast`).
  `fork-program-ast` and `run-sandboxed-hermetic-ast` may need their
  own attach if they don't route through the same primitive — audit
  follow-up if a future case escapes both layers.
- The walker matches `<T,...>`-stripping by simple prefix-find of
  `<`. If a name contains `<` for a non-type-parameter reason, this
  over-strips. Substrate convention is `<T,U,...>` only after
  identifier; collisions would require unusual user names.

## Generalizes

Same substrate-as-teacher pattern as arcs 110 (`CommCallOutOfPosition`),
115 (`InnerColonInCompoundArg`), 117 (`ScopeDeadlock`), 126
(`ChannelPairDeadlock`), 130 (`MalformedVariant`'s span retrofit).
Each is a dedicated CheckError variant + walker; Display IS the
migration brief; no `collect_hints` involvement.

The two-span discipline lands here for the first runtime variant
(`RuntimeError::SandboxScopeLeak`). Per arc 138's "errors carry
coordinates" doctrine — both layers' diagnostics name source
locations users / agents can navigate without grepping.

## Cross-references

- DESIGN: `docs/arc/2026/05/140-deftest-visibility-warning/DESIGN.md`.
- `wat/test.wat` — deftest macro definition (the sandbox boundary).
- `wat/std/sandbox.wat` — `run-sandboxed-ast` (the primitive consumed).
- `src/check.rs::validate_sandbox_scope_leak` — slice 2 walker.
- `src/runtime.rs::SymbolTable.outer_symbols` — slice 1 plumbing.
- `src/spawn.rs::eval_kernel_spawn_program_ast` — outer attach site.
- `docs/arc/2026/05/138-checkerror-spans/DESIGN.md` — sibling arc;
  this arc's two-span discipline draws from 138's doctrine.
- Memory `feedback_no_known_defect_left_unfixed` — discipline that
  course-corrected the slice ordering mid-arc.

## What this arc closes

The "user invoked a name not in scope" failure is now a teaching
diagnostic with file:line:col on both the bad call AND the
outer-scope helper they meant to reference. Two layers — compile
and runtime — guard the path. Sandbox isolation IS the design;
this arc makes the design legible at the failure point.
