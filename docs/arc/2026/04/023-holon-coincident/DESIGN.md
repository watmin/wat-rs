# Arc 023 — `:wat::holon::coincident?` primitive

**Status:** opened 2026-04-22.

**Motivation.** While building outstanding test coverage for the
trading lab's Phase 3.3 `scaled-linear`, the need for a
VSA-native equivalence check surfaced. `presence?` exists (cosine
above noise-floor → "there's signal") but the dual — "these two
holons are the same to the algebra" — was being written inline
every time as:

```scheme
(:wat::core::< (:wat::core::f64::- 1.0 (:wat::holon::cosine a b))
               (:wat::config::noise-floor))
```

That's a load-bearing predicate asking its own question. It
deserves a name.

A web search through Kanerva 2009, Schlegel et al 2022, Kleyko et
al 2023 two-part survey, Plate HRR, Gayler MAP confirmed: **the
bidirectional use of the noise-floor — as presence threshold AND
as equivalence threshold — is consistent with VSA theory but not
named in the published literature**. Classical VSA is vector-first
(cleanup-to-codebook); having TWO constructed ASTs to compare for
structural equivalence is a programming-languages move that wat's
AST-first design surfaces naturally.

The agent's closing note: *"You have room to name it."*

Named under the gaze discipline: **`coincident?`** — two points
occupying the same location on the hypersphere within the
algebra's tolerance. Geometric, specific, self-describing.

---

## UX target

```scheme
;; Presence — "is there signal above random?"
(:wat::holon::presence? target reference)
;; → :bool — cosine(target, reference) > noise-floor

;; Coincidence — "are these the same holon within the algebra's tolerance?"
(:wat::holon::coincident? a b)
;; → :bool — (1 - cosine(a, b)) < noise-floor
```

Same noise-floor value, two dual predicates, one substrate.

Typical usage — structural equivalence assertion in tests:

```scheme
(:wat::test::assert-eq
  (:wat::holon::coincident? fact expected)
  true)
```

---

## Semantics

Given two holons `a` and `b`:

1. Encode both to vectors via the committed EncodingCtx.
2. Compute cosine similarity (same code path as `:wat::holon::cosine`).
3. Return `bool`: `(1 - cosine) < noise-floor`.

Equivalent to `cosine > (1 - noise-floor)`. At d=1024, noise-floor
≈ 0.156, so the threshold is cosine > 0.844. Much stricter than
presence? (cosine > 0.156) — presence? answers "is there ANY
signal?"; coincident? answers "are these THE SAME HOLON?".

---

## Type signature

```
:wat::holon::coincident? : :wat::holon::HolonAST × :wat::holon::HolonAST -> :bool
```

Same shape as `presence?`. Both args are holons; result is
`:bool`.

---

## Implementation

### Runtime (`src/runtime.rs`)

`eval_algebra_coincident_q` — mirrors `eval_algebra_presence_q`:

```rust
fn eval_algebra_coincident_q(
    args: &[WatAST],
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::ArityMismatch {
            op: ":wat::holon::coincident?".into(),
            expected: 2,
            got: args.len(),
        });
    }
    let a = require_holon(":wat::holon::coincident?", eval(&args[0], env, sym)?)?;
    let b = require_holon(":wat::holon::coincident?", eval(&args[1], env, sym)?)?;
    let ctx = require_encoding_ctx(":wat::holon::coincident?", sym)?;

    let va = encode(&a, &ctx.vm, &ctx.scalar, &ctx.registry);
    let vb = encode(&b, &ctx.vm, &ctx.scalar, &ctx.registry);
    let cosine = Similarity::cosine(&va, &vb);
    Ok(Value::bool((1.0 - cosine) < ctx.config.noise_floor))
}
```

Dispatch arm in the evaluator's algebra branch:

```rust
":wat::holon::coincident?" => eval_algebra_coincident_q(args, env, sym),
```

### Check (`src/check.rs`)

Register the scheme next to `presence?`:

```rust
env.register(
    ":wat::holon::coincident?".into(),
    TypeScheme {
        type_params: vec![],
        params: vec![holon_ty(), holon_ty()],
        ret: bool_ty(),
    },
);
```

### Reserved prefix

Already covered — `:wat::holon::*` is a reserved prefix (arc 022).

---

## Tests

### Rust unit tests (`src/runtime.rs`)

Parallel to `presence_q_*` tests:

- `coincident_q_true_for_self` — `(coincident? (Atom "x") (Atom "x"))` returns true. Self-cosine = 1.0, error = 0 < floor.
- `coincident_q_true_for_structurally_same` — two hand-built identical-structure holons (same Bind/Thermometer/Atom shape) return true.
- `coincident_q_false_for_unrelated` — `(coincident? (Atom "x") (Atom "y"))` returns false. Random cosine is near 0, error ≈ 1 > floor.
- `coincident_q_false_for_partially_correlated` — two holons whose cosine is above floor but well below 1 (e.g., Bind sharing one atom). Confirms coincident? is stricter than presence?.

### wat-level tests (`wat-tests/holon/coincident.wat`)

- `test-coincident-on-self-is-true` — `(coincident? atom atom)` via deftest.
- `test-coincident-on-structural-same-is-true` — hand-built equivalent holons.
- `test-coincident-on-unrelated-is-false` — two orthogonal atoms.
- `test-coincident-is-stricter-than-presence` — a pair where `presence?` is true and `coincident?` is false. E.g., `a = Bind(k, v1)`, `b = Bind(k, v2)` where v1 and v2 share some overlap but aren't the same.

---

## Doc sweep

- `docs/CONVENTIONS.md` — namespace table row for `:wat::holon::*`
  updated to name four measurements (cosine, dot, presence?,
  coincident?) not three.
- `docs/USER-GUIDE.md` — section 6 "Algebra forms" adds
  `coincident?` as the fourth measurement. Section 3 "Mental
  model" Axis 1 bullet expanded.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` — the
  `:wat::holon::*` table gets a new row.
- `holon-lab-trading/docs/proposals/.../FOUNDATION.md` — "Where
  Each Lives" measurement section adds coincident?.
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — new row for arc 023.

---

## Downstream simplification

The trading lab's Phase 3.3 test
`test-fact-is-bind-of-atom-and-thermometer` currently inlines
`(:wat::core::< (:wat::core::f64::- 1.0 (:wat::holon::cosine ...))
(:wat::config::noise-floor))`. Simplifies to:

```scheme
(:wat::test::assert-eq
  (:wat::holon::coincident? fact expected)
  true)
```

Also touch the existing `presence?`-based tests where they were
really asking equivalence (wat-tests/holon/Circular.wat,
Reject.wat, Sequential.wat, Subtract.wat, Trigram.wat) — audit
each and migrate the ones that are equivalence claims rather than
presence claims.

---

## What this arc ships

One slice. Mechanical:

1. Runtime dispatch + scheme registration + unit tests.
2. wat-level coverage (wat-tests/holon/coincident.wat).
3. Doc sweep (CONVENTIONS, USER-GUIDE, INVENTORY, FOUNDATION,
   CHANGELOG).
4. Lab simplification — rewrite Phase 3.3 fact-structure test
   using coincident?.
5. INSCRIPTION recording the shipped contract.

---

## What this arc does NOT ship

- Changes to `presence?` semantics or signature.
- Changes to cosine / dot.
- New behavior on existing holon primitives.
- Migration of every existing `presence?` caller. Each caller
  gets a read to decide whether it wanted presence or coincidence;
  the audit is part of this arc's wat-tests/holon migration.

---

## Why this is inscription-class

Implementation lands first, spec catches up. The need surfaced in
testing, the name was picked under the gaze discipline, the
semantic is a natural consequence of the existing noise-floor
framing — no new design question to resolve. Same shape as arcs
019 (round), 020 (assoc), which also shipped as INSCRIPTIONs to
pre-existing spec gaps.
