# Arc 029 — Nested quasiquote — INSCRIPTION

**Shipped:** 2026-04-23. Three slices. Debugged a sneaky expand-time
bug mid-arc that surfaced the need for arc 030 (macroexpand tool).
That cave-quest cross-reference is honest — arc 029's slice 2 wat-
level test tripped on a bug that was only diagnosable with a macro-
expansion primitive wat didn't yet have.

**Commits:**
- `ea16ceb` — DESIGN + BACKLOG opened
- `6bc6c54` — slices 1 + 2 (walk_template depth + make-deftest factory)
- `0e39738` — fix: `expand_form` preserves quasiquote bodies
- `39ea54d` — fix: `expand_form` also preserves QUOTE bodies; `expand_all`
  now takes `&mut MacroRegistry` (was cloning it, discarding newly-
  registered macros)
- (this commit) — slice 3 (INSCRIPTION + doc sweep)

---

## What shipped

### Slice 1 — walk_template depth-tracking

`walk_template` gained a `depth: u32` parameter. `expand_template`
enters at depth 1 (just stripped the outer quasiquote). Three new
match arms:

- `(:wat::core::quasiquote X)`: recurse on X at depth+1; preserve
  wrapper.
- `(:wat::core::unquote X)` at depth 1: substitute (existing behavior
  via `unquote_argument`).
- `(:wat::core::unquote X)` at depth > 1: recurse on X at depth-1;
  preserve wrapper.
- Same two-tier pattern for `(:wat::core::unquote-splicing X)`.

Racket / Common Lisp / Clojure convention — the four majors converge
on the same depth-peel rule because the substrate permits no other
shape for macro-generating-macro templates to work.

`unquote_argument` extended: non-symbol args (already-substituted
literal values from an outer pass) pass through unchanged instead
of erroring. That's the `,,X` resolution path — by the time the
inner unquote fires at depth 1, X has already been replaced by its
literal value at the outer pass.

`splice_argument` similarly extended to handle already-substituted
list values from nested quasiquote resolution.

### Slice 2 — make-deftest factory at wat level

`:wat::test::make-deftest` + `:wat::test::make-deftest-hermetic` shipped
in `wat/std/test.wat`. The factory templates use `,,X` (double unquote)
for the configured-at-outer-pass values and `,X` (single unquote) for
the new macro's own parameters that fire at invocation time.

```scheme
(:wat::core::defmacro
  (:wat::test::make-deftest
    (name :AST<()>)
    (dims :AST<i64>)
    (mode :AST<wat::core::keyword>)
    (default-prelude :AST<()>)
    -> :AST<()>)
  `(:wat::core::defmacro
     (,name (test-name :AST<()>) (body :AST<()>) -> :AST<()>)
     `(:wat::test::deftest ,test-name ,,dims ,,mode ,,default-prelude ,body)))
```

(Later reshaped: arc 030 flipped arg order; arc 031 dropped mode/dims
entirely. This INSCRIPTION records the initial shape at 029 ship time.)

Forcing demo in `wat-tests/std/test.wat`:

```scheme
(:wat::test::make-deftest :wat-tests::std::test::cfg-deftest 1024 :error ())

(:wat-tests::std::test::cfg-deftest
  :wat-tests::std::test::test-make-deftest-runs
  (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))
```

Two macros registered through one substrate pass. First makes the
second; second makes the test body. Green.

### Slice 2 — the mid-arc bug

After slice 1 + slice 2 landed, the wat-level demo for a
make-deftest variant with a **non-empty** default-prelude failed.
The bug wasn't in the depth-tracking; it was in the passes around
the template walk.

Two failures compounded:

1. `expand_form` was recursing INTO `(:wat::core::quote X)` bodies
   and expanding macros within. Pre-029, a quoted form was literal
   data — but a macro-generating-macro's inner template IS a quote
   that contains macro names the OUTER macro wants to defer. The
   outer pass was eagerly expanding the inner template's macro
   references, baking them into the generated defmacro before its
   own parameters existed.

2. `expand_all` was calling `.clone()` on the macro registry and
   passing a local copy downward. When a form registered a new
   defmacro (like the `make-deftest` expansion producing a fresh
   `:wat-tests::std::test::cfg-deftest` macro), the registration
   landed in the clone and was discarded at function return. User
   code calling `:wat-tests::std::test::cfg-deftest` hit
   `UnknownMacro`.

Fixes (commits `0e39738` + `39ea54d`):
- `expand_form`: preserve check extends from just quasiquote to
  quote + quasiquote. Both are "literal data" by the substrate's
  semantics; expand_form must not walk their bodies.
- `expand_all`: signature changed from `&MacroRegistry` to
  `&mut MacroRegistry`. Freshly-registered macros in one form's
  expansion survive into the next form's expansion pass.

Both fixes shipped as one-character edits + the propagation of
`&mut` through a few call sites. The bugs were subtle; the fixes
were surgical.

### Arc 030 cave-quested off this debug

The bug above was hard to see. I was eprintln'ing template outputs
through multi-pass expansions, rebuilding, printing again, grep-
searching across 500-line outputs. The shape that kept catching me:
"here's what my template looked like after expansion" — which is
exactly what Lisp's `macroexpand` primitive returns.

Arc 030 opened mid-debugging of arc 029 to ship
`:wat::core::macroexpand` + `macroexpand-1` primitives. Those
primitives THEN let me reproduce the arc 029 bug in three lines of
wat at a deftest call site, see the pre-expanded AST, confirm the
macro's body wasn't being preserved, commit the fix.

Arc 030's INSCRIPTION covers its own delivery; arc 029 just notes
the cross-reference: **the tool that debugs a bug is substrate too.**

### Slice 3 — INSCRIPTION + doc sweep

This commit.

---

## Tests

**Rust unit tests** (`src/macros.rs`):
- `nested_quasiquote_preserves_inner_unquote`
- `double_unquote_substitutes_at_outer_level`
- `unquote_splicing_at_depth_two_peels`
- `make_deftest_shaped_template_expands_correctly`
- `unquote_of_literal_returns_literal` — new `unquote_argument`
  pass-through behavior

**Wat-level test** (`wat-tests/std/test.wat`):
- Empty-prelude `make-deftest` registration + invocation. Extended
  during arc 030 follow-up to cover non-empty default-prelude after
  the `expand_form` fix.

**Rust integration test** (`tests/wat_make_deftest.rs`):
- Regression test for the expand-time bug. Builds a
  `make-deftest`-registered inner macro with non-empty default-
  prelude; asserts the registered body is
  `(quasiquote (:wat::test::deftest ...))` — deftest NOT pre-
  expanded. Then expands the user's call via macroexpand-1 (arc
  030's tool, used here in anger) and asserts on the expected
  shape. Updated at arc 031 slice 2 for the post-arg-drop shape.

**Workspace:** zero regressions. Every pre-029 macro test passed
unchanged (default depth-1 path preserved).

---

## Sub-fog resolutions

**1a — Span preservation.** Resolved: `call_site_span.clone()` on
every newly-constructed list during depth > 1 peeling. Matches arc
016's template-origin span discipline — synthetic lists attribute
to the caller's position, not to a nonexistent source location.

**1b — quasiquote-inside-splicing.** Resolved: splice handler
recurses through its argument when the argument carries nested
quasiquote/unquote forms. The extra walk costs one pass per depth
level; bounded by user's nesting depth.

---

## What did NOT change

- **Reader-macro shortcuts.** `` ` `` / `,` / `,@` syntactic sugar
  still lands at parse time and produces the `(:wat::core::quasiquote ...)`
  / `(:wat::core::unquote ...)` / `(:wat::core::unquote-splicing ...)`
  forms `walk_template` now handles in depth-aware fashion.
  The reader didn't need changes.
- **Existing single-depth macros.** Every pre-029 macro runs
  through `walk_template` at depth 1; behavior unchanged. Only
  nested-quasiquote templates (previously failing) now work.
- **Cycle detection.** `EXPANSION_DEPTH_LIMIT` and the existing
  fixpoint-convergence check stay intact. Arc 030's `macroexpand`
  uses the same limits.
- **Syntax-rules / syntax-case.** Still out of scope. wat's macro
  system stays procedural (defmacro + quasiquote template) — the
  simpler shape Common Lisp uses, not Scheme's pattern-based one.

---

## The lineage

Common Lisp's backquote dates to 1970s Maclisp. Guy L. Steele
and Richard Gabriel documented it formally. Racket's `quasiquote`
is the direct descendant; Clojure's syntax-quote is a hygienic
variant. Every one of those implementations tracks depth the same
way wat now does. This isn't a wat-specific invention — it's the
standard Lisp move, one arc late because wat's macro system
shipped the depth-1-only shape initially.

The chain:
```
Maclisp (1970) → Common Lisp (1984) → Scheme (1986) →
Racket (1995) → Clojure (2007) → wat (2026-04-23)
```

Arc 029 joined that line.

---

## What comes next

**Arc 030** — macroexpand tool (cave-quested off arc 029's debug;
already shipped in commit `437273f`). Slice 2 INSCRIPTION pending.

**Arc 031** — sandbox inherits outer config (Path B for the
make-deftest ergonomics arc). Already shipped end-to-end. The
configured-deftest shape arc 029 made possible reaches its
minimum form at arc 031:

```scheme
(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::make-deftest :deftest
  ((:wat::load-file! "...")))

(:deftest :my-test body)
(:deftest :another body)
```

Arc 029 is the macro-system enabler; arc 031 is the Config-system
completion; together they deliver the honest-minimum ergonomic
shape that kicked off this whole series in arc 027's slice 4.

*the ergonomic testing story is closed.*
