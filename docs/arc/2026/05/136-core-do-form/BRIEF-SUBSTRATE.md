# Arc 136 — `:wat::core::do` Form Substrate BRIEF (slice 1a)

**Drafted 2026-05-06.** Slice 1a of arc 136's `do` form work.
Per DESIGN: substrate special form, Clojure-faithful, no `-> :T`.

User direction (verbatim, captured in DESIGN):
> *"i don't think do is a macro... i think its just a form the
> runtime provides... the exception case would be confusing
> having it be a long let chain"*

> *"do is value bearing — same concern as let"* + Clojure
> reference (https://clojuredocs.org/clojure.core/do): non-final
> forms' return types are UNCONSTRAINED; final form's type IS the
> do's type.

> *"right now... let and a supposed do form is already implicitly
> typed so the type declaration is just unnecessary noise"*

## Goal

Mint `:wat::core::do` as a substrate special form alongside `if` /
`match` / `let` / `let*` / `try` / `option::expect` /
`result::expect`. No `-> :T` slot. Clojure-faithful semantics:
evaluate each form left-to-right; discard each non-final's result;
return the value of the final form. Final form's inferred type IS
the do form's type — substrate infers; recipient unification at
the consuming site is the static check.

## Target shape

```scheme
(:wat::core::do f1 f2 f3 ... fN)
```

- `(:wat::core::do)` — zero forms; ill-formed (parse error).
- `(:wat::core::do f1)` — single form; degenerate; evaluates to
  f1's value; type IS f1's inferred type. (Sonnet's call: parse
  error, or accept-with-degenerate-semantics. Recommend accept;
  matches Clojure's `(do x) => x`.)
- `(:wat::core::do f1 f2 ... fN)` — evaluate each in sequence;
  discard non-final results; return fN's value. Do form's inferred
  type = fN's inferred type.

## Substrate edits

### `src/check.rs`

Add `infer_do(args, env, ...)`:
- args.is_empty() → MalformedForm "do form requires at least one form"
- Walk args[0..N-1] (non-finals); call `infer` on each (must internally type-check); IGNORE the resulting type (no unification with anything)
- Call `infer` on args[N-1] (final form); return its inferred type

Mirror the function's shape from `infer_if` or `infer_let_star`
minus the `-> :T` parsing logic. There's no migration-hint helper
needed (this is a NEW symbol; no shape pair to detect).

### `src/runtime.rs`

Add `eval_do(args, env, ...)`:
- For each non-final arg, `eval` it; discard the result
- `eval` the final arg; return its value

Add tail-call path `eval_do_tail` (or equivalent integration with
the existing tail-call mechanism) so the final form's tail call
flows through cleanly.

Add `step_do` to the incremental evaluator so eval-step!
preserves do semantics.

### `src/special_forms.rs`

Register `:wat::core::do` with sketch — a variadic special form
where args is "1 or more forms." Look at how arc 144 slice 2
registered `if` / `let` / `let*` for the registration pattern.

### NEW `tests/wat_arc136_do_form.rs`

6-10 unit tests covering:

1. **Empty:** `(:wat::core::do)` → MalformedForm parse error
2. **Single form:** `(:wat::core::do 42)` → evaluates to 42 (i64)
3. **Multi form:** `(:wat::core::do (:println "log") (:println "log2") 42)` → evaluates to 42; both printlns called
4. **Type flow at recipient:** `(:wat::core::define (:probe -> :i64) (:wat::core::do (:println "log") 42))` → type-checks; declared probe returns i64; do's inferred type = 42's i64 = matches probe sig
5. **Recipient mismatch:** declare probe `-> :wat::core::String` but do's final form returns i64 → TypeMismatch at the recipient (probe's body slot)
6. **Non-final type unconstrained:** `(:wat::core::do "string-not-unit" 42)` → type-checks cleanly; "string-not-unit" evaluated and discarded; do returns 42
7. **Reflection round-trip:** lookup-form `:wat::core::do` shows the variadic sketch
8. **Tail-call sanity:** do form in a function body's tail position; tail-call optimization preserved
9. **Nested:** `(:wat::core::do (:wat::core::do (:println "inner") 1) 2)` → both prints called; outer returns 2
10. **Mixed with let*:** typed `let*` whose body is a `do` form — types compose cleanly

Use the existing test harness pattern (`check_errors`, `eval`,
etc.) from `tests/wat_arc145_typed_let.rs`'s shape (for reference)
or other recent `tests/wat_arc*.rs` files.

## Constraints

- **Substrate-only edits.** EXACTLY 4 files: `src/check.rs`,
  `src/runtime.rs`, `src/special_forms.rs`, NEW
  `tests/wat_arc136_do_form.rs`. NO consumer wat edits. NO other
  test files modified. NO other crate.
- **DO COMMIT** when tests pass — this is a clean substrate
  addition; no breaking change (new symbol; existing
  let*-with-unit-bindings sites untouched).
- **STOP at first red** — distinguish substrate-internal red
  (parse error inside check.rs/runtime.rs/special_forms.rs;
  unexpected) vs test-failure-in-new-tests-only (drives your work
  via the substrate-as-teacher loop on YOUR new tests).
- No grinding.

## Pre-flight crawl (mandatory)

1. **`docs/arc/2026/05/136-core-do-form/DESIGN.md`** — read in
   full, especially the new top section (post-back-out
   realization).
2. **`docs/arc/2026/05/145-typed-let/DESIGN.md`** — top section
   (the back-out realization) — context for why do is no-`-> :T`.
3. **`src/check.rs::infer_if`** — pattern reference for special-form
   inference logic.
4. **`src/check.rs::infer_let_star`** — pattern reference for
   variadic special-form handling (you can use the OLD pre-arc-145
   shape since arc 145 was reverted).
5. **`src/runtime.rs::eval_let_star`** — pattern reference for eval
   layer.
6. **`src/special_forms.rs`** — registry sketch pattern from arc 144
   slice 2.

## Pre-flight verification (test BEFORE editing)

```bash
cargo test --release --workspace 2>&1 | grep -cE "FAILED"
```

Must be 0 (workspace currently clean post-arc-145 back-out at HEAD
= `6b6b75a`).

## Verification (after edits)

```bash
cargo test --release --test wat_arc136_do_form 2>&1 | tail -10
```

Expect: all 6-10 tests pass.

```bash
cargo test --release --workspace 2>&1 | grep -E "test result:|FAILED" | tail -5
```

Expect: 0 failed across all crates (no breaking change to consumer
sites).

## Out of scope

- Slice 1b (consumer migration of let*-with-unit-bindings → do)
- Slice 2 closure paperwork (INSCRIPTION + 058 row + USER-GUIDE)
- Lab consumers (`holon-lab-trading/`) — separate workspace

## Reporting (~250 words)

1. **Pre-flight crawl confirmation:** DESIGN, arc 145 DESIGN
   top section, infer_if, infer_let_star, eval_let_star,
   special_forms.rs all read.

2. **Edit summary:**
   - infer_do — variadic; non-finals' types discarded; final's type returned
   - eval_do, eval_do_tail, step_do — sequential eval; non-final results dropped; final value returned
   - special_forms.rs — sketch registered
   - tests/wat_arc136_do_form.rs — 6-10 new tests

3. **LOC delta:** before/after across the 4 files.

4. **Verification:**
   - `cargo test --test wat_arc136_do_form` — pass count
   - `cargo test --workspace` — 0 failed

5. **Path:** Mode A clean (substrate ships; tests pass; workspace
   stays 0-failed) / Mode B substrate-internal-bug / Mode C
   unexpected interaction with existing forms.

6. **Honest deltas:** any edge case in tail-call / step paths;
   any registry-sketch shape question; any test that surfaced an
   interaction you didn't anticipate.

7. **Commit + push** when Mode A clean. Use a commit message
   following the project's pattern. INSCRIPTION + 058 row are slice
   2 closure (out of scope here).

## Time-box

60 minutes wall-clock (predicted upper-bound 30 min; 2× cap).

## Why this brief

This is a clean substrate addition mirroring the established
special-form pattern. No consumer migration needed. Once shipped,
arc 136 slice 1b (consumer sweep let*-with-unit-bindings → do)
can spawn separately — that sweep is deletion-of-noise via the
ergonomic do form, not a breaking change.

Mode A clean = the substrate gains a clean Clojure-faithful
sequencing form; arc 109 wind-down has one more value-bearing
core form completed; arc 136 slice 1b ready to spawn next.
