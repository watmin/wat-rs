# Arc 143 Slice 2 — Sonnet Brief — Computed unquote in defmacro bodies

**Drafted 2026-05-02 (late evening).** Substrate-informed: orchestrator
crawled `src/macros.rs:541-767` + `src/runtime.rs:5878,15432` before
writing this brief. Every primitive/function this brief references has
been verified to exist with the assumed shape.

**The architectural framing:** the substrate's defmacro bodies are
currently restricted to PURE quasiquote templates with parameter
substitution. Inside `(quasiquote ...)`, `,name` substitutes a macro
parameter (Symbol-only) and `,@name` splices a parameter that's a
List. Arbitrary expressions inside `,(expr)` or `,@(expr)` are NOT
supported — they fall through and are returned as-is. This blocks any
macro that needs to compute its expansion based on substrate state
(e.g., `define-alias` needs `,(:wat::runtime::signature-of target-name)`).

This slice removes that restriction by extending the unquote walker
to evaluate List-shaped arguments at expand-time. Once shipped, every
future reflective macro (define-alias, sweep generators, spec
validators) gets the foundation for free.

**Goal:** extend `unquote_argument` and `splice_argument` to evaluate
List-shaped arguments via the existing `eval()` function, converting
the result back to AST via the existing `value_to_watast()`.

**Working directory:** `/home/watmin/work/holon/wat-rs/`.

## Required pre-reads (in order)

1. **`docs/arc/2026/05/143-define-alias/DESIGN.md`** —
   read "Findings", "Section: Slice 2 — Computed unquote",
   and "Resolution-order semantics".
2. **`src/macros.rs:541-770`** — the current `expand_template` /
   `walk_template` / `unquote_argument` / `splice_argument` chain.
   Note: the body must be `(quasiquote ...)` (line 603-622); inside,
   `,arg` is handled by `unquote_argument` (line 785+); only Symbol
   and already-substituted literal cases are supported.
3. **`src/macros.rs:471-530`** — `expand_form` (caller of
   `expand_macro_call`); shows the recursion + macro-call detection.
4. **`src/macros.rs:541-590`** — `expand_macro_call` (caller of
   `expand_template`); builds `bindings` from args + invokes the
   template walker.
5. **`src/runtime.rs:5878-5970`** — `value_to_watast(op, v, span)`
   converter. Used by `struct->form`. Same pattern transfers to
   computed unquote.
6. **`src/runtime.rs:15400-15470`** — the stdlib bootstrap site where
   `expand_all` is called. Note: at this point, `SymbolTable.functions`
   may not be fully populated (defines register AFTER macro expansion).
   The substrate-primitive dispatch path in `eval()` does NOT depend
   on populated `sym.functions` — primitives are hardcoded match arms.

## What to ship

### Substrate change — extend the unquote walker

**Current behavior of `unquote_argument` (`src/macros.rs:785+`):**

```rust
fn unquote_argument(
    arg: &WatAST,
    bindings: &HashMap<String, WatAST>,
) -> Result<WatAST, MacroError> {
    match arg {
        WatAST::Symbol(ident, sym_span) => match bindings.get(&ident.name) {
            Some(bound) => Ok(bound.clone()),
            None => Err(MacroError::UnboundMacroParam { ... }),
        },
        // Already-substituted literal — return as-is
        _ => Ok(arg.clone()),  // List, IntLit, etc. all fall through
    }
}
```

**New behavior:** when `arg` is a `WatAST::List`, evaluate it as a wat
expression at expand-time. The List represents a function call (e.g.,
`(:wat::runtime::signature-of target-name)`). To evaluate:

1. **Substitute macro params recursively in the List**: walk the List;
   for each `WatAST::Symbol` whose name matches a key in `bindings`,
   replace with the bound AST. Other nodes pass through unchanged.
   This is a NEW small helper `substitute_bindings(form, bindings) -> WatAST`.
2. **Evaluate the substituted form** via existing `crate::runtime::eval(form, env, sym)`.
3. **Convert the result Value to WatAST** via existing
   `crate::runtime::value_to_watast(op, val, span)`.
4. Return the resulting WatAST.

`splice_argument` extends similarly: when arg is a List-expression
that's not already a known-list-literal, evaluate, then verify the
resulting Value is a Vec and splice its elements (each converted via
`value_to_watast`).

### Threading `sym` through the chain

`unquote_argument` and `splice_argument` need `&SymbolTable` to call
`eval()`. This requires extending the entire call chain:

| Function | Current signature | New signature |
|---|---|---|
| `unquote_argument` | `(arg, bindings)` | `(arg, bindings, env, sym)` |
| `splice_argument` | `(arg, bindings, macro_name)` | `(arg, bindings, macro_name, env, sym)` |
| `walk_template` | `(form, bindings, macro_scope, macro_name, call_site_span, depth)` | `+ env, sym` |
| `expand_template` | `(template, bindings, macro_scope, macro_name, call_site_span)` | `+ env, sym` |
| `expand_macro_call` | `(def, args, call_site_span)` | `+ env, sym` |
| `expand_form` | `(form, registry, depth)` | `+ env, sym` |
| `expand_once` | `(form, registry)` | `+ env, sym` |
| `expand_all` | `(forms, registry)` | `+ env, sym` |

For `env`, use `&Environment::default()` at the bootstrap call sites
(macros expanded at stdlib load have no enclosing scope). For `sym`,
use `&SymbolTable::default()` at bootstrap (substrate primitives
dispatch fine without populated `sym.functions`).

The runtime macroexpand sites (`runtime.rs:6602`, `:6645`) ALREADY
have `sym` available — pass it through.

### `substitute_bindings` helper (new, ~30 LOC)

```rust
/// Walk `form`, replacing every `WatAST::Symbol` whose name is a
/// key in `bindings` with the bound AST. Recursive on `WatAST::List`.
/// Other variants pass through unchanged.
fn substitute_bindings(form: &WatAST, bindings: &HashMap<String, WatAST>) -> WatAST {
    match form {
        WatAST::Symbol(ident, _) => {
            if let Some(bound) = bindings.get(&ident.name) {
                bound.clone()
            } else {
                form.clone()
            }
        }
        WatAST::List(items, span) => {
            let new_items: Vec<WatAST> = items
                .iter()
                .map(|item| substitute_bindings(item, bindings))
                .collect();
            WatAST::List(new_items, span.clone())
        }
        other => other.clone(),
    }
}
```

### What stays unchanged

- The Symbol-substitution path in `unquote_argument` (when arg IS a
  bare Symbol, look up in bindings — same as today)
- The List-as-already-substituted-literal path? **Verify**: the
  current behavior for List arg is "return as-is" treating it as an
  already-substituted literal. The new behavior treats List arg as an
  expression-to-evaluate. This CHANGES behavior for any existing
  macro whose body has `,(some-list-literal)` — but per the existing
  comment in `unquote_argument`, the only known producer of
  already-substituted literals is the `,,X` (double-unquote) outer-
  pass path, which produces ASTs that are NOT call-shaped lists.
  Verify by reading existing macros (e.g., `wat/test.wat:387+`) +
  running the existing macro tests.

  **If existing macros depend on the "List = literal" behavior**,
  add a heuristic: a List is "callable" only if its first element is
  a `WatAST::Keyword` (matching the call-shape). Lists whose head is
  not a Keyword stay as literals.

## Tests

Add 4-6 unit tests to `src/macros.rs::tests` (the existing test
module at line 856+):

1. **Symbol unquote still works** — `,name` substitutes the bound
   AST (existing behavior; verify no regression).
2. **List literal in unquote still works** (if heuristic chosen) —
   `,(some-list-of-data)` returns the list as-is when head isn't a
   keyword.
3. **Computed unquote evaluates a substrate primitive call** —
   define a macro whose body is `` `(:wrapper ,(some-substrate-call arg)) ``;
   verify the expansion contains the evaluation result.
4. **Computed unquote substitutes macro params before evaluating** —
   define a macro that takes a param + uses it inside an unquoted
   expression; verify the expression sees the substituted value.
5. **Computed unquote-splicing** — `,@(expr)` evaluates `expr` and
   splices the resulting Vec elements.
6. **Computed unquote inside nested quasiquotes** — depth handling
   still works (per the existing arc 029 slice 1 nested-quasiquote
   logic).

Pick concrete substrate primitives that are SAFE to use in tests
(don't require sym.functions). Good candidates: `:wat::core::i64::+`,
`:wat::core::vec`, `:wat::core::String`, `:wat::runtime::signature-of`
(slice 1 shipped — verify it works in this expand-time eval context).

## Constraints

- **TWO Rust files modify:** `src/macros.rs` (the walker chain) and
  `src/runtime.rs` (only at the call sites of `expand_all` /
  `expand_once` / `register_*_defmacros` to thread sym; possibly
  `src/check.rs` for similar threading at its `expand_all` call sites).
- **No new substrate primitives.** This slice only extends the macro
  expander.
- **Workspace stays GREEN:** `cargo test --release --workspace` exits
  0; the existing 1 LRU pre-existing failure remains; ZERO new
  regressions; existing macro tests at `src/macros.rs:856+` ALL pass
  unchanged (no behavior break for existing macros).
- **No commits, no pushes.**

## Workflow per piece

1. Add `substitute_bindings` helper to `src/macros.rs`. Add a unit
   test for it (Symbol replacement + List recursion).
2. Run `cargo test --release -p wat`. Verify the helper test passes.
3. Extend `unquote_argument` with the List-as-expression path. Add
   the heuristic if needed for backward compat. Run cargo test;
   verify no regressions in existing macro tests.
4. Extend `splice_argument` similarly.
5. Thread `env` + `sym` through the chain (expand_template,
   walk_template, callers). Update bootstrap call sites to pass
   `&Environment::default()` and `&SymbolTable::default()`.
6. Run cargo test workspace; verify green.
7. Add the 4-6 new tests for computed unquote.
8. Run cargo test workspace; verify all green.

**STOP at first red:** if any step breaks existing macro tests or
introduces a regression, surface the failure + STOP. Don't grind.

## What success looks like

1. `cargo test --release --workspace`: exit=0; same baseline pass
   count + your 4-6 new tests; only the pre-existing 1 LRU failure
   remains.
2. Existing macro tests at `src/macros.rs:856+` all pass unchanged
   (no behavior break).
3. The new computed-unquote tests pass — proving you can write a
   macro body that calls a substrate primitive at expand-time and
   splices the result.
4. The signature of `expand_template` (and the chain through
   `unquote_argument`) carries `&SymbolTable` so future slices can
   build on the macro-evaluating-substrate-primitives capability.

## Reporting back

Target ~250 words:

1. **Files touched + LOC delta** — `src/macros.rs` lines (signatures
   extended + new logic in unquote_argument/splice_argument + new
   helper). `src/runtime.rs` lines (bootstrap call sites). Possibly
   `src/check.rs` if it has its own `expand_all` call sites.
2. **Backward-compat decision** — what heuristic (if any) you used
   to distinguish "List = expression to evaluate" vs "List = literal
   to return as-is." E.g., "head is a Keyword → eval; else literal."
3. **The new test bodies** — quote 2-3 of the new tests verbatim so
   the orchestrator can verify the discipline.
4. **Test totals** — `cargo test --release --workspace` passed /
   failed / ignored. Confirm 1 pre-existing failure still present
   (the arc 130 LRU stepping stone), no NEW regressions.
5. **Honest deltas** — anything you needed to invent or diverge from
   the brief (e.g., the env type — does it need to be an
   `Environment` or a fresh one per macro? Verify and report).

## Sequencing — what to do, in order

1. Read DESIGN.md + the 6 anchor docs above.
2. Read `src/macros.rs:541-770` cover-to-cover; understand the
   current walker.
3. Read `src/runtime.rs:5878-5970` (value_to_watast precedent).
4. Read `src/runtime.rs:15400-15470` (bootstrap site).
5. Add `substitute_bindings` helper + test.
6. Extend `unquote_argument` for List-as-expression. Run test.
7. Extend `splice_argument` similarly.
8. Thread sym through the chain.
9. Add 4-6 new computed-unquote tests.
10. Run cargo test workspace; confirm green; report.

Then DO NOT commit. Working tree stays modified for the orchestrator
to score.
