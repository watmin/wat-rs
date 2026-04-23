# Arc 029 — Nested quasiquote — BACKLOG

**Shape:** three slices, leaves-to-root. Status markers:
- **ready** — dependencies satisfied; can be written now
- **obvious in shape** — will be ready when the prior slice lands
- **foggy** — needs design work before it's ready

---

## Slice 1 — walk_template depth-tracking

**Status: ready.**

Target: `src/macros.rs`. Extend `walk_template` to thread a `depth:
u32` parameter. `expand_template` enters at depth 1 (just stripped
the outer `(:wat::core::quasiquote ...)`).

New dispatch rules inside `walk_template`:
- `(:wat::core::quasiquote X)` form: recurse on X at depth+1;
  preserve the quasiquote wrapper in output.
- `(:wat::core::unquote X)` at depth 1: substitute (existing
  behavior via `unquote_argument`).
- `(:wat::core::unquote X)` at depth > 1: recurse on X at depth-1;
  preserve the unquote wrapper in output.
- `(:wat::core::unquote-splicing X)` at depth 1: splice (existing).
- `(:wat::core::unquote-splicing X)` at depth > 1: recurse on X at
  depth-1; preserve wrapper.

Rust unit tests (in `#[cfg(test)] mod tests` at end of macros.rs):
- `nested_quasiquote_preserves_inner_unquote`
- `double_unquote_substitutes_at_outer_level`
- `unquote_splicing_at_depth_two_peels`
- `triple_nesting_depth_three`
- `make_deftest_shaped_template_expands_correctly` — the canonical
  forcing case.

**Sub-fogs (expected to resolve at implementation):**
- **1a — Span preservation.** The outer macro's span is on the
  outer list. When we wrap inner preserved unquotes with
  `(:wat::core::unquote ...)` at depth > 1, what span goes on the
  fresh list? Likely: `call_site_span.clone()` (matches existing
  template-origin span discipline from arc 016).
- **1b — quasiquote-inside-splicing.** Does the current splice
  handler recurse-walk its argument? It doesn't today — just
  fetches the bound list. At nested depth we need to walk the
  arg so inner quasiquote/unquote/unquote-splicing forms get
  depth-aware treatment. Check + fix in implementation.

## Slice 2 — wat-level make-deftest proof

**Status: obvious in shape** (once slice 1 lands).

Target: `wat/std/test.wat` + `wat-tests/std/test.wat`. Ship
`:wat::test::make-deftest` with default-prelude baked in:

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

Demo test in `wat-tests/std/test.wat`:

```scheme
(:wat::test::make-deftest :wat-tests::std::test::my-deftest 1024 :error ())

(:wat-tests::std::test::my-deftest
  :wat-tests::std::test::test-make-deftest-works
  (:wat::test::assert-eq (:wat::core::i64::+ 2 2) 4))
```

If slice 1's depth tracking works, this registers two macros —
one via `make-deftest`, one via the expanded defmacro — and then
invokes the test successfully.

Pair with a hermetic variant: `:wat::test::make-deftest-hermetic`
that expands the inner to `:wat::test::deftest-hermetic`.

Lab migration candidate (slice 2 optional, or arc 027 follow-up):
`wat-tests/vocab/shared/time.wat` can adopt the configured
shape:

```scheme
(:wat::test::make-deftest :trading::test::vocab::shared::time::tdt
  1024 :error
  ((:wat::load-file! "wat/vocab/shared/time.wat")))

(:trading::test::vocab::shared::time::tdt
  :trading::test::vocab::shared::time::test-encode-time-facts-count
  <body>)
```

Five lines of test instead of seven per test.

## Slice 3 — INSCRIPTION + doc sweep

**Status: obvious in shape** (once slices 1 + 2 land).

- `docs/arc/2026/04/029-nested-quasiquote/INSCRIPTION.md`.
- `docs/USER-GUIDE.md` — macro chapter gains a "Nested quasiquote"
  subsection with the `make-deftest` worked example.
- `docs/CONVENTIONS.md` — quote-depth semantics table; Racket / CL
  lineage note.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — new
  `:wat::test::make-deftest` (+ hermetic variant) rows.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md` —
  new row for arc 029.
- arc 027 INSCRIPTION (when arc 027 closes) gains a footer note
  pointing at arc 029's `make-deftest` delivery + a follow-up slice
  in arc 027 migrating lab tests to use it.

---

## Working notes (updated as slices land)

- Opened 2026-04-23. Cut from arc 027 slice 4's post-migration
  discovery — the builder's configured-deftest ask surfaced the
  nested-quasiquote substrate gap.
