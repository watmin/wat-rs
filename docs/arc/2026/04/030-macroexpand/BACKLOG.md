# Arc 030 — macroexpand / macroexpand-1 — BACKLOG

**Shape:** two slices, leaves-to-root. Status markers:
- **ready** — dependencies satisfied; can be written now
- **obvious in shape** — will be ready when the prior slice lands
- **foggy** — needs design work before it's ready

---

## Slice 1 — runtime primitives + SymbolTable carries the registry

**Status: ready.**

Targets: `src/macros.rs`, `src/runtime.rs`, `src/check.rs`,
`src/freeze.rs`.

Changes:
- `src/macros.rs` — new `pub fn expand_once(form, registry) ->
  Result<WatAST, MacroError>` helper. Runs one macro-call
  expansion at the top level (no subtree fixpoint). If the form
  is a macro call: expand via `expand_macro_call`, return result.
  If not a macro call: return form unchanged.
- `src/runtime.rs` — add a `macro_registry: Option<Arc<MacroRegistry>>`
  field to `SymbolTable`. Populated at freeze time by
  `freeze.rs::FrozenWorld::freeze`. Runtime primitives that need
  it call a new `sym.macro_registry() -> Option<&MacroRegistry>`
  accessor.
- `src/runtime.rs` — two new eval handlers:
  - `eval_macroexpand_1(args, env, sym)` — arity 1, accepts
    `:wat::WatAST`, calls `expand_once`, returns `:wat::WatAST`.
  - `eval_macroexpand(args, env, sym)` — arity 1, accepts
    `:wat::WatAST`, calls `expand_form` (existing fixpoint),
    returns `:wat::WatAST`.
- Dispatch arms in the main eval match for the two keyword paths.
- `src/check.rs` — scheme registrations for both primitives.
- `src/freeze.rs` — `FrozenWorld::freeze` installs the macro
  registry on the symbols table (Arc-clone so runtime has access).

**Rust unit tests** (`#[cfg(test)] mod tests` additions to
`src/macros.rs` or a new test module in `src/runtime.rs`):
- `macroexpand_1_alias_expands_one_step`
- `macroexpand_1_non_macro_returns_unchanged`
- `macroexpand_runs_to_fixpoint`
- `macroexpand_1_nested_quasi_preserves_inner_unquote` — the
  arc 029 diagnostic case
- `macroexpand_returns_wat_ast_value`

**Wat-level test** (`wat-tests/std/test.wat`):
- deftest that uses macroexpand-1 to get the AST of a simple alias
  expansion, asserts on the result's shape via match + atom-value.
- deftest that uses macroexpand to drive a two-step chain to
  fixpoint.

**Sub-fogs:**
- **1a — SymbolTable registry plumbing.** Does the freeze pipeline
  already have MacroRegistry accessible at the FrozenWorld
  assembly site? Yes (the expand pass consumed it; just need to
  Arc::clone into the frozen symbols). Verify at implementation.
- **1b — `:wat::WatAST` as runtime value.** Already a first-class
  value type (`Value::wat__WatAST(Arc<WatAST>)` from arc 010).
  Nothing new needed.
- **1c — Pretty-printing expanded AST.** Deferred — users can
  inspect via `atom-value` + structural match. If a test-oriented
  `to-string` primitive is trivial, ship it in slice 1 as a
  convenience. If non-trivial, defer to later.

## Slice 2 — INSCRIPTION + doc sweep + diagnose arc 029

**Status: obvious in shape** (once slice 1 lands).

Writing:
- `docs/arc/2026/04/030-macroexpand/INSCRIPTION.md` — standard
  shape. What shipped, tests, commit refs.
- `docs/USER-GUIDE.md` — new "Debugging macros" subsection with
  macroexpand / macroexpand-1 examples. Point at the make-deftest
  diagnostic as the canonical use case.
- `docs/CONVENTIONS.md` — short note on when to reach for
  macroexpand vs reading the macro definition directly.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — two
  new rows under `:wat::core::*` for the primitives.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — new row.

Diagnostic work for arc 029:
- New wat-level test at `wat-tests/holon/` or similar exercising
  `:wat::test::make-deftest` with non-empty default-prelude.
  Uses macroexpand-1 to capture what the generated inner macro
  actually produces. Asserts on the expected shape; when the
  assertion fails, the captured AST is the bug signature.
- Update `docs/arc/2026/04/029-nested-quasiquote/BACKLOG.md`
  with the root cause (once macroexpand reveals it) and the
  shipped fix. Close arc 029 slice 3 with the updated
  INSCRIPTION.

**Sub-fogs:**
- **2a — root cause of make-deftest-with-prelude bug.** Unknown
  until slice 1's macroexpand lets us see it. Slice 2's work is
  to USE the tool to find the bug, then fix it in arc 029.
- **2b — whether the fix lands in arc 030 or arc 029.** If the
  bug is in walk_template's depth handling, it's arc 029 scope.
  If it's in expand_form's fixpoint logic, it might be arc 030
  scope. TBD at diagnosis time.

---

## Working notes (updated as slices land)

- Opened 2026-04-23 following builder's "we're missing a tool"
  observation during arc 029 make-deftest debugging.
