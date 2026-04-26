# Arc 066 — `eval-ast!` returns wrapped HolonAST per its scheme

**Status:** shipped 2026-04-26. See `INSCRIPTION.md` for the
canonical post-ship record. One delta from the spec: shipped
`RuntimeError::TypeMismatch` (with a clear "form whose terminal
value has a HolonAST representation" message) instead of a new
`RuntimeError::NotHolonExpressible` variant. The kind tag in
EvalError reads `"type-mismatch"`; the diagnostic message carries
the meaning. Future arc promotes to a typed variant if a
consumer surfaces a need to match on non-expressibility
specifically.

**Predecessor:** arc 057 (typed-holon-leaves) closed the algebra
under itself — every primitive value has a HolonAST representation.
This arc honors that closure at the `eval-ast!` boundary so the
type signature stops lying.

**Consumer:** experiment 009's diagnostic surfaced that
`:wat::eval-ast!` is statically typed `Result<HolonAST, EvalError>`
but at runtime returns the bare Value (e.g., `Value::i64(4)` for
`(+ 2 2)`, NOT `Value::holon__HolonAST(HolonAST::I64(4))`). A caller
matching `(Ok h)` gets `h` typed as HolonAST per the checker but
actually i64 at runtime. Calling `atom-value` on h then runtime-rejects:
*"got: i64, expected: wat::holon::HolonAST"*.

The substrate currently has a static-vs-dynamic mismatch at this
boundary. Per the simple/honest exercise:

- **Simple**: ONE return shape, always `Result<HolonAST, EvalError>`. No
  polymorphism, no exceptions. ✓
- **Honest**: scheme matches runtime. The substrate's promise (the
  `to-watast` docstring's claim that *"a HolonAST tree round-trips
  through to-watast → eval-ast! back to the same HolonAST shape"*)
  becomes literally true instead of aspirational. ✓

This is a substrate bug. Fix it.

Builder direction (2026-04-26, post-arc-064 substrate-bug review):

> "for issue B... option A is the answer"

Option A: wrap the eval result as HolonAST in `eval_form_ast` before
the `wrap_as_eval_result` step. The runtime delivers what the scheme
promises.

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `:wat::eval-ast!` (scheme: `WatAST → Result<HolonAST, EvalError>`) | shipped — scheme is right; runtime is wrong |
| `wrap_as_eval_result` (wraps Result-shape) | shipped — one layer too high |
| `value_to_atom` / `watast_to_holon` (for HolonAST construction) | shipped (arc 057) |
| `:wat::core::atom-value` (HolonAST primitive leaf → wat Value) | shipped — caller's extraction primitive after eval-ast! |

The infrastructure to convert a Value to its HolonAST representation
already exists. `value_to_atom`'s primitive-leaf branch is exactly
what `eval_form_ast` needs to call on its result.

## What's missing (this arc)

| Change | What it does |
|----|----|
| `eval_form_ast` (`runtime.rs`) | Wraps the inner `eval(...)` result as HolonAST before passing to `wrap_as_eval_result` |
| `value_to_holon` helper (extracted from `value_to_atom`) | Optional — pull the primitive-leaf-to-HolonAST conversion into a reusable helper if `value_to_atom`'s shape doesn't fit cleanly |

One real change. Possibly one helper extraction.

---

## Decisions to resolve

### Q1 — Which Values get wrapped, and how?

The eval result can be any Value the substrate can produce. The
arc 057 algebra closure covers:

- `Value::i64(n)` → `HolonAST::I64(n)`
- `Value::f64(x)` → `HolonAST::F64(x)`
- `Value::bool(b)` → `HolonAST::Bool(b)`
- `Value::String(s)` → `HolonAST::String(s)`
- `Value::wat__core__keyword(k)` → `HolonAST::Symbol(k)`
- `Value::holon__HolonAST(h)` → already a HolonAST; return as-is
  (or wrap as `HolonAST::Atom(h)`?)
- `Value::Vec(...)` / `Value::Tuple(...)` / `Value::Struct(...)` →
  no direct HolonAST representation; **error**
- `Value::wat__WatAST(a)` → `watast_to_holon(&a)` (lower the AST)
- Channel handles, ProgramHandles, etc. → no HolonAST representation; **error**

**Recommended:**

- For HolonAST-representable Values (primitives + HolonAST-input):
  wrap as the matching HolonAST variant.
- For non-HolonAST-representable Values (channels, etc.): return an
  EvalError describing the mismatch — the form's terminal value
  isn't HolonAST-expressible, so eval-ast! can't honor its scheme.
- Explicit error message: *"eval-ast! requires a form whose terminal
  value has a HolonAST representation; got <type>"*.

This makes the scheme's contract explicit: eval-ast! is for forms
whose results are algebra-expressible. Non-expressible results get
a clean error, not a static-vs-dynamic mismatch.

### Q2 — `Value::holon__HolonAST(h)` — wrap or pass through?

Two interpretations:

- **(a) Pass through** — h is already a HolonAST; the result IS h
- **(b) Wrap as Atom** — `HolonAST::Atom(h)` — the result is the
  opaque-identity wrap of h

**Recommended: (a) pass through.** The eval-ast! contract is
"return the form's value as a HolonAST." If the value IS a HolonAST
already, return it directly. Wrapping would introduce a depth the
caller must unwrap, which violates simple.

### Q3 — Should `value_to_holon` be a public substrate primitive?

Right now `value_to_atom`'s primitive-leaf branch implements the
conversion. We could:

- **(a)** Inline the conversion in `eval_form_ast` (private function)
- **(b)** Extract `value_to_holon` as an internal helper (private)
- **(c)** Expose `:wat::holon::from-value` as a wat-level primitive

**Recommended: (b) extract internal helper.** Used by
`eval_form_ast`, possibly used by future arcs (e.g., a polymorphic
`show` follow-up that renders Values via HolonAST round-trip).
Public exposure (option c) is overkill — callers who want the
Value→HolonAST conversion can reach for `leaf` (post-arc-065).

### Q4 — Migration impact on callers

Existing callers of `eval-ast!`:

- The match arm `(Ok h) → (atom-value h)` is the canonical pattern.
  After this arc, `h` is genuinely a HolonAST::I64 (or matching
  primitive variant); `atom-value` correctly extracts the i64.
  **Existing pattern starts working as documented.**
- Callers who were unwrapping h directly (without atom-value) and
  expecting a bare i64 break. They migrate to add the `atom-value`
  call.
- `wrap_as_eval_result`'s contract is unchanged; callers' Result
  matching stays the same.

The change is RUNTIME-VISIBLE. Callers that worked through the bug
(treating h as bare value) break; callers that followed the scheme
(treating h as HolonAST and using atom-value) start working.

### Q5 — Error type when result isn't HolonAST-expressible

`EvalError` is the existing error type for eval failures. Adding a
new variant `EvalError::NotHolonExpressible { type_name: String }`
or similar gives callers a typed reason to dispatch on.

**Recommended:** YES, add the variant. Callers can match
`(Err NotHolonExpressible)` to handle the case explicitly. The
error message includes the type name for diagnostics.

---

## What ships

One slice. Pure runtime fix to honor the existing scheme.

- `eval_form_ast` in `runtime.rs` — wraps the inner eval result
  as HolonAST before `wrap_as_eval_result`
- `value_to_holon` helper — extracted from `value_to_atom`'s
  primitive-leaf branch (private)
- `EvalError::NotHolonExpressible` variant — added for the case
  where the result has no HolonAST representation
- Tests inline in `src/runtime.rs::mod tests`:
  - `(eval-ast! (to-watast (from-watast (quote (+ 2 2)))))` →
    `Ok(HolonAST::I64(4))` (post-arc-065 syntax)
  - Same with f64, bool, String, keyword
  - Eval result is HolonAST → pass through unchanged
  - Eval result is non-expressible (e.g., a Vec) → `Err(NotHolonExpressible)`
  - Round-trip: HolonAST tree → to-watast → eval-ast! → same shape
    (this is the docstring's claim made literally true)
- `docs/USER-GUIDE.md` — eval-ast! row updated; round-trip
  example fixed; new error variant documented

Estimated effort: ~50 lines Rust + ~30 lines tests + doc
updates. Single commit. Pattern matches arcs 058–065.

---

## Open questions

- **Should `eval-ast!` also work for forms whose result is a
  Value::Vec of primitives** (representable as HolonAST::Bundle)?
  Probably yes, future arc — too broad for v1. Today's behavior:
  rejected with NotHolonExpressible.
- **Forms that produce side effects** (channel sends, file writes):
  the eval still runs the side effects; the return value is the
  question. If the side-effect-producing form's terminal value is
  HolonAST-expressible (e.g., the form ends with a primitive),
  works fine. Otherwise rejected. Same posture as v1.
- **Idempotency of round-trip**: the docstring's claim
  ("HolonAST tree round-trips through to-watast → eval-ast!")
  becomes a substrate-level invariant that arc 066 enforces. A
  round-trip test (build a HolonAST, to-watast, eval-ast!, compare)
  becomes the canonical regression test.

## Slices

One slice. Single commit. Pattern matches arcs 058–065.

## Consumer follow-up

After this arc lands, experiment 009's helpers can use the
documented round-trip pattern reliably:

```scheme
;; Post-arc-065 + arc-066:
((form :HolonAST) (:wat::holon::from-watast (:wat::core::quote (:wat::core::+ 2 2))))
((ast :WatAST) (:wat::holon::to-watast form))
((result :Result<HolonAST, EvalError>) (:wat::eval-ast! ast))
((value :i64)
  (:wat::core::match result -> :i64
    ((Ok h) (:wat::core::atom-value h))    ;; h is HolonAST::I64; atom-value extracts i64
    ((Err _) -1)))
```

The chain works as documented. T11's terminal-value claim
becomes provable via the round-trip; T1 and T2's value coincidence
becomes a real proof (not the accidental -1 == -1 pass).

The diagnostic loop closes: substrate bug found → arc DESIGN →
arc shipped → consumer uses the now-honest API → tests prove
what they claim.
