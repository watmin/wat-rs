# Arc 029 — Nested quasiquote (macro-generating-macro support)

**Status:** opened 2026-04-23. Cut as the blocker-resolution arc for
arc 027 slice 4's `make-deftest` follow-up.

**Motivation.** Arc 027 slice 4 surfaced the need for a configured
`deftest` factory — the builder's shape:

```scheme
(:wat::test::make-deftest :trading::test::my-deftest 1024 :error)

(:trading::test::my-deftest :my::test-name
  ((:wat::load-file! "foo.wat"))
  (body))
```

`make-deftest` is a macro that REGISTERS a macro. Its template body
contains both outer-unquotes (substitute the outer macro's `dims` +
`mode` args) and inner-unquotes (the new macro's `name` / `prelude` /
`body` args, to be substituted LATER when the new macro is invoked).

wat's current `walk_template` has one quote depth. Every
`(:wat::core::unquote X)` substitutes at walk time regardless of
nesting. Inner unquotes can't survive through the outer expansion.

**Fix.** Track quote depth; only substitute at depth 1 (the outermost
quasiquote); preserve + peel at depth > 1.

---

## Semantics — Racket / CL convention

Quote depth starts at 1 when `walk_template` enters (we've just
stripped the outer `(:wat::core::quasiquote ...)` at
`expand_template`'s dispatch site).

Nested-form rules:

| Form | Depth action | Substitution behavior |
|---|---|---|
| `(:wat::core::quasiquote X)` | recurse on X at depth + 1 | preserve wrapper; walk X |
| `(:wat::core::unquote X)` at depth 1 | — | substitute X's bound value |
| `(:wat::core::unquote X)` at depth > 1 | recurse on X at depth - 1 | preserve wrapper; walk X |
| `(:wat::core::unquote-splicing X)` at depth 1 | — | splice X's bound list |
| `(:wat::core::unquote-splicing X)` at depth > 1 | recurse on X at depth - 1 | preserve wrapper; walk X |

Nested unquote (`,,X` in reader notation) is two `(:wat::core::unquote ...)`
wrappers. Inside a doubly-nested quasiquote (depth 3), the outer
`(:wat::core::unquote ...)` drops to depth 2, preserves. The inner
drops to depth 1, substitutes. So `,,X` inside `` `( `( ,,X ) ) ``
reads the outermost macro's binding of X at the outer expansion pass.

This matches Common Lisp, Scheme, Racket, and Clojure. `make-deftest`
becomes writable in one expansion pass.

---

## The forcing example — `make-deftest`

The builder's ergonomic target: configure dims, mode, AND the
per-test-file's common loads once, then each test is just a name
+ body:

```scheme
;; Once per test file:
(:wat::test::make-deftest :trading::test::my-deftest 1024 :error
  ((:wat::load-file! "wat/vocab/shared/time.wat")))

;; Every test afterward — no dims, no mode, no loads:
(:trading::test::my-deftest
  :my::test-name
  (body))
```

Implementation via nested quasiquote:

```scheme
(:wat::core::defmacro
  (:wat::test::make-deftest
    (name :AST<()>)
    (dims :AST<i64>)
    (mode :AST<wat::core::keyword>)
    (default-prelude :AST<()>)
    -> :AST<()>)
  `(:wat::core::defmacro
     (,name
       (test-name :AST<()>)
       (body :AST<()>)
       -> :AST<()>)
     `(:wat::test::deftest ,test-name ,,dims ,,mode ,,default-prelude ,body)))
```

Outer walk at depth 1:
- `(:wat::core::defmacro ...)` — preserve head, walk children.
- `,name` at depth 1 → substitute the outer macro's `name` arg
  (becomes the new macro's declared name).
- Inner `` `(:wat::test::deftest ...) `` at depth 2 — the nested
  quasi bumps depth.
- At depth 2: `,test-name` is `(unquote test-name)`. test-name is a
  SYMBOL. Preserve the unquote wrapper + keep the symbol; this
  unquote fires when the user later invokes their new macro.
- At depth 2: `,,dims` is `(unquote (unquote dims))`. Outer unquote
  at depth 2 → drop to 1, walk arg. Inner `(unquote dims)` at
  depth 1 → substitute dims via `unquote_argument` — outer arg
  binding (1024 literal).
- Same for `,,mode` and `,,default-prelude`. Values literally
  inserted where the double-unquote sat.
- `,body` at depth 2 → preserve the unquote wrapper.

Output = a fully-formed `(:wat::core::defmacro ...)` registration
for the user's new macro, with `1024`, `:error`, and the full
default-prelude list literally baked in, and only two parameters
(`test-name`, `body`) as deferred unquotes.

Fixpoint expansion (the existing `nested_macro_expands_to_fixpoint`
test pattern) then picks up the generated defmacro on the next
pass when the user calls it.

### Extending `unquote_argument` to handle literals

`,,X` at depth 2 produces `(unquote <substituted-value>)` in the
outer expansion. When the INNER (user-invoked) macro expands, its
walker hits that `(unquote 1024)` at depth 1 and calls
`unquote_argument(1024)`. Today's implementation only handles
Symbol arguments — everything else errors. Arc 029 slice 1 extends
it: non-symbol arguments are already-substituted literal values,
returned as-is. That's the Racket/CL convention for how `,,X`
resolves through two passes.

---

## Implementation

**Single function signature change.** `walk_template` gains a
`depth: u32` parameter. Entry point calls with 1.

**Single-dispatch walk_template body grows three match arms:**

```rust
fn walk_template(
    form: &WatAST,
    bindings: &HashMap<String, WatAST>,
    macro_scope: ScopeId,
    macro_name: &str,
    call_site_span: &Span,
    depth: u32,  // <— NEW
) -> Result<WatAST, MacroError> {
    match form {
        WatAST::List(items, _) => {
            // NEW: detect `(:wat::core::quasiquote X)` — bump depth.
            if let Some(arg) = match_unquote(items, ":wat::core::quasiquote") {
                let inner = walk_template(arg, bindings, macro_scope,
                                           macro_name, call_site_span, depth + 1)?;
                return Ok(WatAST::List(vec![
                    WatAST::Keyword(":wat::core::quasiquote".into(), call_site_span.clone()),
                    inner,
                ], call_site_span.clone()));
            }

            // UPDATED: unquote fires only at depth 1; otherwise peel + walk.
            if let Some(arg) = match_unquote(items, ":wat::core::unquote") {
                return if depth == 1 {
                    unquote_argument(arg, bindings, macro_name)
                } else {
                    let inner = walk_template(arg, bindings, macro_scope,
                                              macro_name, call_site_span, depth - 1)?;
                    Ok(WatAST::List(vec![
                        WatAST::Keyword(":wat::core::unquote".into(), call_site_span.clone()),
                        inner,
                    ], call_site_span.clone()))
                };
            }

            // Walk children; handle unquote-splicing at depth 1 as today,
            // peel at depth > 1.
            ...
        }
        ...
    }
}
```

`expand_template`'s final call becomes
`walk_template(quasi_body, ..., 1)`.

Matches the shape of the existing `nested_macro_expands_to_fixpoint`
unit test's two-level flat case but extends to true nesting.

---

## Slices

1. **Slice 1** — `walk_template` gains `depth` param; quasiquote
   / unquote / unquote-splicing get depth-aware handling. Entry
   point at `expand_template` passes 1. `unquote_argument` extends
   to return non-symbol arguments as-is (the `,,X` resolution
   path). Rust unit tests:
   - existing tests: pass unchanged (default depth 1 path).
   - `nested_quasiquote_preserves_inner_unquote`
   - `double_unquote_substitutes_at_outer_level`
   - `unquote_splicing_at_depth_two_peels`
   - `make_deftest_shaped_template_expands_correctly`
   - `unquote_of_literal_returns_literal` — new unquote_argument
     behavior.

2. **Slice 2** — Wat-level proof: `:wat::test::make-deftest` ships
   in `wat/std/test.wat`. Demo test in `wat-tests/std/test.wat`
   that uses `make-deftest` to build a configured `deftest` variant,
   then registers + invokes a test through it.

3. **Slice 3** — INSCRIPTION + doc sweep.
   - arc 029 INSCRIPTION.md.
   - USER-GUIDE.md macro chapter: nested-quasiquote subsection
     with `make-deftest` worked example.
   - CONVENTIONS.md: quote-depth semantics table; points at
     Racket / CL for lineage.
   - 058 FOUNDATION-CHANGELOG row.
   - arc 027 INSCRIPTION (when arc 027 closes) gains a pointer to
     arc 029's `make-deftest` delivery.

---

## Non-goals

- Quasiquote-splicing at nested depths via user-supplied Vec values
  (the existing `,@` syntax already only accepts macro-parameter
  names — this arc preserves that restriction).
- Reader-macro shortcuts `` ` `` / `,` / `,@` already land at parse
  time — unchanged.
- Syntax-rules / syntax-case pattern matching — out of scope.

---

## Why this is inscription-class

Nested quasiquote is a standard Lisp feature. Its absence in wat was
a substrate gap discovered by a real caller demanding macro-
generating-macro ergonomics. Same pattern as every cave-quest arc
since 017 — Phase work hits a wall; substrate gets the fix; main
quest resumes stronger. Arc 027's `make-deftest` follow-up lands
cleanly once this ships.
