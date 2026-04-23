# Arc 026 — `:wat::holon::eval-coincident?` family

**Status:** opened 2026-04-23. Slice 1 (`eval-coincident?` — base
AST variant) ships per builder directive. Slices 2–4 (-edn / -digest
/ -signed) stay deferred until a concrete second caller demands
them; the discipline from CONVENTIONS.md "When to add a primitive"
keeps the speculation in check.

**Motivation.** Phase 3.4 of the trading lab (the rhythm encoder)
surfaced a primitive the substrate didn't have: a way to ask
"are these two *expressions* equivalent after evaluation?"
without forcing the caller to hand-wrap scalars in
`:wat::holon::Atom`. The book's Chapter 28 retort — the AWS
principal moment — is exactly this shape: `(= (+ 2 2) (* 1 4))`
said plain.

Builder's framing:

> eval-coincident? accepts 2 args - a pair of forms - we eval
> those forms and drop the results into atoms and compared them
>
> we need the various forms of eval for this -digest and -signed
> with their required params taken with the forms and passed down

Two layers distinguish two different coincidence-predicates:

- **`coincident?`** (arc 023) — structural. Takes two already-built
  `:wat::holon::HolonAST` values, projects to vectors, checks
  `(1 - cosine) < coincident_floor`.
- **`eval-*-coincident?`** (this arc) — evaluation. Takes the same
  kind of arguments the existing `eval-*!` family takes (for each
  of two sides), evaluates each, lifts each result via
  `:wat::holon::Atom`, then runs `coincident?` on the atomized
  pair. Mirrors the whole eval-* family: four siblings covering
  trusted AST, EDN parse, digest-verified, signed-verified.

Builder's example:

```
(eval-coincident? (+ 2 2) (* 1 4))  ≡  (coincident? (Atom 4) (Atom 4))
```

Both forms evaluate to `:i64 4`. Both get atomized. Both atoms
encode to the same deterministic vector (same canonical-EDN hash
→ same seed). `coincident?` fires true.

---

## The family

Mirror of the shipped eval-* family (`runtime.rs:1803-1806`):

| Base form | Coincidence sibling | Arity | Verification |
|-----------|---------------------|-------|--------------|
| `eval-ast!` (1) | `eval-coincident?` | 2 | none |
| `eval-edn!` (2) | `eval-edn-coincident?` | 4 | none (parse-only) |
| `eval-digest!` (5) | `eval-digest-coincident?` | 10 | SHA-256 per side |
| `eval-signed!` (7) | `eval-signed-coincident?` | 14 | Ed25519 per side |

Each sibling's arity is `2 × base_arity`: the verification params
are per-side, honest, no shared-source magic.

All four siblings return the same uniform type:
`:Result<:bool, :wat::core::EvalError>`. Any failure on either
side (source fetch, digest mismatch, signature mismatch, parse
error, runtime error, non-atomizable result) becomes an `Err`.
Success is `Ok(<bool>)`.

---

## UX target

### `eval-coincident?` — two in-scope ASTs

```scheme
;; The book's retort:
(:wat::holon::eval-coincident?
  (:wat::core::quote (:wat::core::i64::+ 2 2))
  (:wat::core::quote (:wat::core::i64::* 1 4)))
;; → :Result<:bool, EvalError> — Ok(true)

;; Phase 3.4's test:
(:wat::holon::eval-coincident?
  (:wat::core::quote (:trading::encoding::rhythm::indicator-rhythm
                       "rsi" values 0.0 100.0 10.0))
  (:wat::core::quote <reference-expression>))
;; → :Result<:bool, EvalError> — Ok(true) iff the two expressions
;;   evaluate to atomically-equivalent holons.
```

The args are AST values — typically produced by `:wat::core::quote`
for single-form capture or `:wat::core::forms` for multi-form.
Execution runs each AST through the same `run_constrained` path
`eval-ast!` uses — mutation forms refused, the constrained-eval
invariant upheld.

### `eval-edn-coincident?` — two EDN sources

```scheme
(:wat::holon::eval-edn-coincident?
  :wat::eval::source "(:wat::core::i64::+ 2 2)"
  :wat::eval::source "(:wat::core::i64::* 1 4)")
;; → :Result<:bool, EvalError>
```

Per-side `(:wat::eval::<iface> <locator>)` pair. Parse-then-run
each side, atomize results, coincidence-check.

### `eval-digest-coincident?` — two digest-verified sources

```scheme
(:wat::holon::eval-digest-coincident?
  :wat::eval::file-path   "/tmp/prog-a.wat"
  :wat::verify::digest-sha256
  :wat::verify::hex-inline "0123…"
  :wat::eval::file-path   "/tmp/prog-b.wat"
  :wat::verify::digest-sha256
  :wat::verify::hex-inline "4567…")
;; → :Result<:bool, EvalError>
```

Side A's (source, algo, digest) precedes side B's. Each side runs
its own verification before parse, mirroring `eval-digest!`.

### `eval-signed-coincident?` — two signature-verified sources

```scheme
(:wat::holon::eval-signed-coincident?
  :wat::eval::file-path    "/tmp/prog-a.wat"
  :wat::verify::signed-ed25519
  :wat::verify::base64-inline "sig-A…"
  :wat::verify::base64-inline "pubkey-A…"
  :wat::eval::file-path    "/tmp/prog-b.wat"
  :wat::verify::signed-ed25519
  :wat::verify::base64-inline "sig-B…"
  :wat::verify::base64-inline "pubkey-B…")
;; → :Result<:bool, EvalError>
```

Side A's full sig-verify group precedes side B's. Parse-verify-run
per side, atomize, coincidence.

---

## Semantics — shared shape across all four

Given a pair of "eval inputs" — one per side, whose exact shape
depends on the variant (AST / EDN / digest / signed):

1. Resolve + verify + parse + run side A. Yields `Value_a`.
2. Resolve + verify + parse + run side B. Yields `Value_b`.
3. Atomize each via `value_to_atom` (existing, `runtime.rs:4560`).
   Accept set: `:i64 / :f64 / :bool / :String / :wat::core::keyword
   / :wat::holon::HolonAST / :wat::WatAST`. Non-atomizable types
   fail with `EvalError{kind="non-atomizable"}`.
4. Encode the two atoms, compute cosine, compare against
   `ctx.config.coincident_floor`. Return `Ok(<bool>)`.

Any failure (source fetch, verification, parse, mutation-form
refusal, runtime error, type mismatch at atomize) is wrapped as
`Err(<EvalError>)` — same discipline as the parent `eval-*!`
forms.

---

## Type signatures

```
:wat::holon::eval-coincident?
    : :wat::WatAST × :wat::WatAST
        -> :Result<:bool, :wat::core::EvalError>

:wat::holon::eval-edn-coincident?
    : :wat::core::keyword × :String × :wat::core::keyword × :String
        -> :Result<:bool, :wat::core::EvalError>

:wat::holon::eval-digest-coincident?
    : (:wat::core::keyword × :String × :wat::core::keyword
       × :wat::core::keyword × :String)  -- side A
    × (:wat::core::keyword × :String × :wat::core::keyword
       × :wat::core::keyword × :String)  -- side B
        -> :Result<:bool, :wat::core::EvalError>

:wat::holon::eval-signed-coincident?
    : (:wat::core::keyword × :String × :wat::core::keyword
       × :wat::core::keyword × :String × :wat::core::keyword × :String)  -- A
    × (:wat::core::keyword × :String × :wat::core::keyword
       × :wat::core::keyword × :String × :wat::core::keyword × :String)  -- B
        -> :Result<:bool, :wat::core::EvalError>
```

Concrete per-arg types match the existing eval-* parents. Source
locators can be `:String` (inline / file-path) or other parent-
supported shapes; ditto for verify payloads (hex / base64 /
file-path).

---

## Implementation

### Shared helper (`src/runtime.rs`)

```rust
/// Run one eval-coincident side: get a Value via a provided
/// closure (which applies verification + parse + run per variant),
/// then atomize via `value_to_atom`. Caller assembles two sides
/// and runs `coincident_of_two_atoms`.
fn atomize_eval_side<F>(
    op: &'static str,
    env: &Environment,
    sym: &SymbolTable,
    run_side: F,
) -> Result<HolonAST, RuntimeError>
where
    F: FnOnce(&Environment, &SymbolTable) -> Result<Value, RuntimeError>,
{
    let v = run_side(env, sym)?;
    let atom = value_to_atom(v)?;
    Ok(require_holon(op, atom)?.as_ref().clone())
}

/// Encode both sides, cosine, return `:Result<bool, EvalError>`.
/// Used by all four eval-coincident-family dispatchers — each one
/// resolves its two sides per its verification discipline, then
/// hands the resulting holon pair here.
fn coincident_of_two_holons(
    a: HolonAST,
    b: HolonAST,
    sym: &SymbolTable,
    op: &'static str,
) -> Result<Value, RuntimeError> {
    let ctx = require_encoding_ctx(op, sym)?;
    let va = encode(&a, &ctx.vm, &ctx.scalar, &ctx.registry);
    let vb = encode(&b, &ctx.vm, &ctx.scalar, &ctx.registry);
    let cosine = Similarity::cosine(&va, &vb);
    Ok(Value::bool((1.0 - cosine) < ctx.config.coincident_floor))
}
```

### Four dispatchers

Each mirrors its parent's structural pre-check + EvalError wrap
discipline. Sketch for the simplest:

```rust
fn eval_form_ast_coincident_q(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::MalformedForm {
            head: ":wat::holon::eval-coincident?".into(),
            reason: format!("takes exactly 2 arguments; got {}", args.len()),
        });
    }
    wrap_as_eval_result((|| -> Result<Value, RuntimeError> {
        let side_a = run_ast_side(&args[0], env, sym)?;
        let side_b = run_ast_side(&args[1], env, sym)?;
        let atom_a = value_to_atom(side_a)?;
        let atom_b = value_to_atom(side_b)?;
        let a = require_holon(":wat::holon::eval-coincident?", atom_a)?;
        let b = require_holon(":wat::holon::eval-coincident?", atom_b)?;
        coincident_of_two_holons((*a).clone(), (*b).clone(), sym,
                                 ":wat::holon::eval-coincident?")
    })())
}
```

The `run_ast_side` factor shares the `run_constrained` call
currently inline in `eval_form_ast`. Digest / signed variants
reuse `resolve_eval_source` / `parse_verify_algo_keyword` /
`resolve_verify_payload` from `eval_form_digest` / `eval_form_signed`.

### Dispatch registration

```rust
":wat::holon::eval-coincident?"        => eval_form_ast_coincident_q(args, env, sym),
":wat::holon::eval-edn-coincident?"    => eval_form_edn_coincident_q(args, env, sym),
":wat::holon::eval-digest-coincident?" => eval_form_digest_coincident_q(args, env, sym),
":wat::holon::eval-signed-coincident?" => eval_form_signed_coincident_q(args, env, sym),
```

### Check (`src/check.rs`)

Four scheme registrations. Each has a parametric-free signature
(the arg types are concrete `:wat::WatAST` / `:wat::core::keyword`
/ `:String` — no type vars). Return is the uniform Result.

### Reserved prefix

Already covered — `:wat::holon::*` reserved since arc 022. The
`eval-*-coincident?` names live naturally alongside `coincident?`.

---

## Slices

- **Slice 1** — `eval-coincident?` (base, AST form). The most-
  used shape; also unblocks Phase 3.4's immediate need.
- **Slice 2** — `eval-edn-coincident?`. String-parsing variant.
- **Slice 3** — `eval-digest-coincident?`. SHA-256 verification.
- **Slice 4** — `eval-signed-coincident?`. Ed25519 verification.

Slices 2–4 reuse the resolver helpers slice 1 factors out. Each
slice: runtime dispatcher + scheme + Rust unit tests + wat-level
tests. The doc sweep and INSCRIPTION land in slice 4.

---

## Tests

### Rust unit tests (`src/runtime.rs`)

Per slice, at minimum:

- True case — equivalent expressions / same source / verified
  matching bytes or sigs.
- False case — different scalars / different sources.
- Err case on one side — unverifiable digest, bad signature,
  parse failure, non-atomizable result — propagates as Result's
  Err.

Slice 1 specific:
- `eval_coincident_q_true_for_equivalent_arithmetic` —
  `(+ 2 2)` ≡ `(* 1 4)`.
- `eval_coincident_q_true_for_same_string`.
- `eval_coincident_q_false_for_different_scalars`.
- `eval_coincident_q_true_for_structurally_same_holon` — two
  hand-built Bind Atoms.
- `eval_coincident_q_accepts_mixed_types` — `4` vs `(Atom 4)`.
- `eval_coincident_q_err_on_non_atomizable` — e.g., a Value::Unit
  result wraps to Err rather than panicking.

### wat-level tests (`wat-tests/holon/eval_coincident.wat`)

Slice 1 ships:
- `test-eval-coincident-arithmetic-equivalence`.
- `test-eval-coincident-different-scalars`.
- `test-eval-coincident-strings`.
- `test-eval-coincident-hand-built-holons`.

Slices 2–4 ship parallel coverage per variant.

---

## Doc sweep

- `docs/CONVENTIONS.md` — `:wat::holon::*` namespace row lists
  five measurement-class predicates (cosine, dot, presence?,
  coincident?) and four eval-coincident-family predicates.
- `docs/USER-GUIDE.md` — section 6 "Algebra forms" adds the
  `eval-coincident?` family with the distinction vs `coincident?`
  called out explicitly.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — four
  new rows.
- `holon-lab-trading/docs/proposals/.../FOUNDATION.md` — "Where
  Each Lives" measurement section adds the family.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — one rolled-up row for the arc.

---

## Downstream caller

Phase 3.4's `test-short-window-shape` — the test that motivated
this arc — gets a cleaner shape via `eval-coincident?`:

```scheme
(:wat::test::assert-eq
  (:wat::core::match
    (:wat::holon::eval-coincident?
      (:wat::core::quote <rhythm-call-a>)
      (:wat::core::quote <rhythm-call-b>))
    -> :bool
    ((Ok b)  b)
    ((Err _) false))
  true)
```

Where both sides are quoted rhythm invocations. The evaluation-
layer coincidence means we don't need to construct the expected
AST shape by hand — the expected behavior is defined by a second
rhythm call with the reference inputs.

---

## What this arc ships

Four slices. Each is mechanical once slice 1's helpers are in
place.

---

## What this arc does NOT ship

- Changes to `coincident?` — it stays structural-only.
- Changes to `value_to_atom`'s accept-set — mirror exactly what
  `:wat::holon::Atom` already accepts.
- Changes to `run_constrained` / `parse_and_run` / verification
  helpers — arc 026 reuses them unchanged.
- Investigation of the empty-Bundle panic in `eval_algebra_bundle`
  — separate work; probably a separate arc if the builder
  confirms the panic is a defect rather than intentional.
- A single-form-with-multiple-sources verification primitive
  (e.g., "one source, verified two ways") — no caller demand; the
  two-sided symmetry is the honest shape.

---

## Why this is inscription-class

Implementation lands first, spec catches up. The primitive family
is a natural layering on top of the already-shipped `coincident?`
+ `Atom` + `value_to_atom` + the whole `eval-*!` family. No new
design question to resolve beyond what the builder's directive
already pinned. Same shape as arcs 019, 020, 023, 024, 025 —
pattern is standing practice.
