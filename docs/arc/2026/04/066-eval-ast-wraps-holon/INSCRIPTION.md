# wat-rs arc 066 — eval-ast! returns wrapped HolonAST per scheme — INSCRIPTION

**Status:** shipped 2026-04-26. One slice, one commit, ~30 minutes
of focused work — runtime fix to honor an existing scheme.

Builder direction (2026-04-26, post-arc-064 substrate-bug review):

> "for issue B... option A is the answer"

`:wat::eval-ast!`'s scheme has been `Result<HolonAST, EvalError>`
since arc 028, but the runtime returned the bare Value (e.g.
`Value::i64(4)` for `(+ 2 2)`). Callers matching `(Ok h)` got `h`
typed-as-HolonAST per the checker but actually a bare i64 at
runtime; downstream `(:wat::core::atom-value h)` errored with
"got: i64, expected: HolonAST". Static-vs-dynamic mismatch.

This arc honors the scheme: the runtime wraps the terminal value
as HolonAST before passing it through `wrap_as_eval_result`. The
substrate's promise becomes literally true.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/runtime.rs` — `eval_form_ast` updated to call `value_to_holon` on the inner-eval result before passing to `wrap_as_eval_result`. New `value_to_holon` helper (private) — primitives lift via the matching HolonAST leaf constructor (same dispatch as arc 065's `:wat::holon::leaf`); `Value::holon__HolonAST` passes through unchanged; non-HolonAST-expressible Values return TypeMismatch with a clear "form whose terminal value has a HolonAST representation" message. The pre-arc-066 test that asserted `Value::i64(42)` for `(+ 40 2)` updated to expect `HolonAST::I64(42)` per the new scheme. | ~50 Rust + ~10 doc | 6 new (i64 wrapped, bool wrapped, String wrapped, HolonAST passes through, non-expressible result returns Err, to-watast → eval-ast! round-trip) | shipped |

**wat-rs unit-test count: 675 → 681. +6. Workspace: 0 failing.**

Build: `cargo build --release` clean. `cargo test --release`
(workspace-wide per arc 057's `default-members`): 0 failures.

---

## Architecture notes

### `value_to_holon` reuses arc 065's primitive-lift conventions

Same dispatch shape as `:wat::holon::leaf` (arc 065):
`Value::i64 → HolonAST::I64`, `Value::String → HolonAST::String`,
etc. The two ops share semantics by design — `eval-ast!`'s
return wrap should produce the same HolonAST shape that a
hypothetical `(leaf <result>)` call would. Implemented as a
private helper rather than calling `eval_holon_leaf` directly so
the helper can return RuntimeError directly (instead of going
through the eval-arg path).

`Value::holon__HolonAST` passes through unchanged (per DESIGN Q2
recommendation a). Wrapping it in `HolonAST::Atom` would force
callers to unwrap a depth they didn't ask for; the contract is
"return the form's value as a HolonAST," and a HolonAST IS
already that.

### Non-expressible results return TypeMismatch as EvalError

DESIGN Q5 recommended adding `RuntimeError::NotHolonExpressible`
as a typed variant. Audit found: adding a new RuntimeError
variant requires touching the existing exhaustive match in
`runtime_error_to_eval_error_value` (where TypeMismatch already
maps to a clean `"type-mismatch"` kind tag in EvalError) plus
potentially every other RuntimeError-matching site.

The shipped path uses `RuntimeError::TypeMismatch` with a
specific `expected: "form whose terminal value has a HolonAST
representation (primitive or HolonAST)"` message. The kind tag in
EvalError reads `"type-mismatch"`; the diagnostic message carries
the full meaning. A future arc may promote this to a typed
variant when a real consumer surfaces a need to dispatch on
non-expressibility specifically.

### The round-trip claim becomes literal

Pre-arc-066, arc 057's docstring claimed *"a HolonAST tree
round-trips through to-watast → eval-ast! back to the same
HolonAST shape."* Post-arc-066, this is enforceable as a
substrate invariant. Test
`to_watast_eval_ast_round_trip_for_form` exercises it:

```scheme
;; build a form on the algebra grid
((form …) (from-watast (quote (+ 40 2))))
;; lift to runnable wat source
((ast …) (to-watast form))
;; run it; result is HolonAST per scheme
((Ok h) → 42 via atom-value)
```

The chain works as documented. The substrate's promise is no
longer aspirational.

### Runtime-visible breaking change

The change is observable: callers who were using `eval-ast!` and
then operating on the result as a bare value (without
`atom-value`) break. They worked through the bug; the shipped fix
takes that crutch away.

Callers who followed the static scheme (treated the result as
HolonAST and called atom-value) start working as documented.
Audit of the substrate's own tests found exactly one such caller
(`eval_ast_bang_runs_a_parsed_program`), updated as part of this
arc's commit. The lab side has no callers operating on the bare
result of `eval-ast!` — every site goes through `atom-value` or a
match arm that will continue to work post-arc-066.

---

## What this unblocks

- **Lab experiment 009 T11** — `(:wat::eval-ast! (:wat::holon::to-watast form))`
  followed by `(Ok h) → (atom-value h)` recovers the form's
  terminal value as documented. T1 / T2's "values coincide"
  assertion can now PROVE coincidence via the round-trip,
  instead of the accidental sentinel-coincidence pass that
  surfaced the substrate bug.
- **The diagnostic story closes** — capture (arc 016) + payload
  (arc 064) + display (arc 064) + Atom-honesty (arc 065) +
  eval-ast!-honesty (this arc). Every documented round-trip is
  literal; every assertion failure carries its own context;
  every constructor verb names the move it makes. The
  substrate-side diagnostic loop opened by experiment 009 T11
  is closed.
- **Future cache-as-coordinate-tree** — `from-watast` builds a
  coordinate; `to-watast → eval-ast!` retrieves the value. The
  pair is the substrate's "form-as-coordinate vs form-as-value"
  duality made operational.

---

## What this arc deliberately did NOT add

- **`RuntimeError::NotHolonExpressible` typed variant.** DESIGN
  recommended; shipped TypeMismatch instead to minimize blast
  radius. Future arc when a consumer surfaces a need to match
  on non-expressibility specifically.
- **Auto-lift of Vec / Tuple / Struct results to a Bundle.**
  Different semantic question (would need to decide on the
  Bundle's element-type encoding). DESIGN flagged as future arc;
  v1 returns Err for these cases.
- **Public `:wat::holon::from-value` primitive.** DESIGN Q3 option
  c. Internal helper sufficient; callers wanting Value→HolonAST
  outside of `eval-ast!` reach for arc 065's `leaf`.

---

## The thread

- **Arc 028** — `eval-ast!` shipped with `Result<HolonAST,
  EvalError>` scheme; runtime returned bare Value (the bug).
- **Arc 057** — to-watast docstring claimed "round-trips through
  eval-ast!"; aspirational, not enforced.
- **2026-04-26 (mid-T11 debugging)** — proofs lane sees a
  `(Ok h) → (atom-value h)` chain runtime-fail with TypeMismatch.
  The substrate bug surfaces.
- **2026-04-26 (DESIGN)** — proofs lane drafts the arc; option A
  selected (wrap at the substrate boundary).
- **2026-04-26 (this session)** — slice 1 ships in one commit:
  value_to_holon helper + eval_form_ast wrap + 1 test update +
  6 new tests + USER-GUIDE row + this INSCRIPTION.
- **Next** — T11 debugging resumes with the round-trip honest;
  T1 / T2's accidental-pass becomes a real pass.

PERSEVERARE.
