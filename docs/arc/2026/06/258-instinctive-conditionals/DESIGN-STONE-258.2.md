# DESIGN — Stone 258.2: `cond` reborn as a wat macro (burn the heresy)

**Status: STRIKE-READY target. cond's honest form — a wat macro over bare `if`, the Rust special
form annihilated.** Split: **258.2a** macro + corpus sweep (Rust cond goes dead); **258.2b**
annihilate the dead Rust cond.

## Why

`cond` is heresy in its current form: a Rust special form smeared across six sites
(`special_forms.rs`, `eval_cond` + `eval_cond_tail`, `infer_cond`, `Boundary::Cond`, the
`normalize.rs` arm), demanding a mandatory `-> :T` return annotation no Lisp asks for. Its honest
form is what instinct reaches for: a `defmacro` in `core.wat` expanding to nested bare `if`. The
pipeline guarantees a clean shadow — macro expansion is **step 4**, normalize is **step 7**, check
**step 8** (freeze.rs) — so a `cond` macro reduces every `(cond …)` to `if` before any Rust cond
machinery runs. Exactly the arc-249 `->`/`->>`→macro→delete-Rust move.

## Scope is small

`cond` appears in the corpus only in `wat/stream.wat` (2 uses) + 2 Rust test files. No dual-read
needed — the macro is bare-only and the handful of sites are swept in-stone.

## The macro (258.2a) — in `core.wat`, beside `->`/`->>`

```clojure
(:wat::core::defmacro :wat::core::cond
  [& clauses <- :AST<wat::holon::Holons>]
  -> :AST<wat::holon::HolonAST>
  ;; one clause left ⇒ the (:else body) base case ⇒ its body
  (:wat::core::if (:wat::core::empty? (:wat::core::rest clauses))
     (:wat::core::second
        (:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first clauses) "cond: empty"))
     ;; ≥2 clauses ⇒ (if <test> <body> (cond <rest…>))
     `(:wat::core::if
         ~(:wat::core::first (:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first clauses) "cond: empty"))
         ~(:wat::core::second (:wat::core::Option/expect -> :wat::holon::HolonAST (:wat::core::first clauses) "cond: empty"))
         (:wat::core::cond ~@(:wat::core::rest clauses)))))
```

- The macro body uses bare `if` (258.1) — itself stepped by the engine at expansion time.
- `(cond …)` in the expansion is **re-expanded to fixpoint** (expand.rs:133), terminating when one
  clause remains (depth = arm count, well under `EXPANSION_DEPTH_LIMIT`).
- `first`/`second`/`rest`/`empty?` return `Option` and are wrapped with `Option/expect` exactly as
  the `->` macro does (core.wat).
- NOTE for the build — **`second`'s exact return shape** and whether the arm's `(test body)` head/
  tail extraction needs `(first (first clauses))` (test) vs `(first clauses)` being the arm: verify
  against the corpus arm shape `(<test> <body>)` (a 2-element List). Adjust `first`/`second` nesting
  to match.

### Totality — REQUIRED, never optional (corrected 2026-06-10, the ADT reframe)

`cond` is **total**: a terminal `:else` is **required** (see DESIGN.md "wat is an ADT language" and
[[feedback_optional_is_a_smell]]). `:else` is cond's **wildcard** (ML/Rust `_`) and obeys the two
wildcard laws: it must be **last** (an arm after it is unreachable → error) and **present** (no
`:else` → non-exhaustive → error). This is `if`'s mandatory-else law surfacing at the bottom of the
nest — `cond` is nested `if`, and the innermost `if`'s else-branch *is* the `:else` body.

The macro enforces it by walking arms left-to-right:
- arm head is a **test** (a List): if it's the last clause → **non-exhaustive error**; else →
  ``(if <test> <body> (cond <rest…>))``.
- arm head is the **`:else` keyword** (detect via `keyword/to-string` ⇒ `":else"`, engine-pure):
  if it's the last clause → emit `<body>`; else → **`:else`-not-last (unreachable) error**.
- the macro raises errors the way `->` does — an expansion-time `Option/expect` on a forced `None`
  with the message.

The first sonnet build took the last arm unconditionally (neither total nor nil-fallthrough — it
*dropped* the last arm's test). That was a correctness regression caught by
`tests/wat_core_cond.rs::cond_refuses_missing_else`; the rewrite above restores totality.

## The sweep (258.2a) — drop `-> :T` from the ~4 cond sites

`wat/stream.wat:490,508` (`(cond -> :T arms)` → `(cond arms)`) + the 2 Rust test files. The arms
themselves are unchanged.

## The annihilation (258.2b) — delete the now-dead Rust cond

`special_forms.rs:147`, `eval_cond` + `eval_cond_tail` + dispatch arms (runtime.rs:2699,3468),
`infer_cond` + dispatch (check.rs:3850-area), `validate_cond_shape`, `Boundary::Cond`
(boundary.rs:43,63), `normalize_cond` + its arm (normalize.rs:117,202), the `:wat::core::cond`
entry in the engine's form list (eval.rs:426). Pure deletion of dead code, gated by "still green."
Advances 251.6 (one fewer `normalize.rs`/`boundary.rs` special case).

## Probe (RED at HEAD) — 258.2a

`tests/probe_arc258_stone2_cond_macro.rs`, all bare (no `-> :T`):
- C01: `(cond ((= 1 1) 10) (:else 20))` evals to `10`. (RED at HEAD: Rust cond demands `-> :T`.)
- C02: `(cond ((= 1 2) 10) (:else 20))` evals to `20`. (else branch.)
- C03: `(cond ((= 1 2) 10) ((= 2 2) 20) (:else 30))` evals to `20`. (3-arm — proves fixpoint recursion.)

## Gate

- `cargo test --release --test probe_arc258_stone2_cond_macro` → 3/3 (RED at HEAD).
- `cargo build --release` clean; full suite: only the 4 nursery deadlock-reds (stream.wat's cond
  still works through the macro after the sweep).
