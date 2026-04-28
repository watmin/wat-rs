# wat-rs arc 069 — `:wat::holon::coincident-explain` — INSCRIPTION

**Status:** shipped 2026-04-27. One slice, one commit, ~30 minutes
of focused work — substrate diagnostic primitive surfaced from
proof 018's debug session.

Builder direction (2026-04-27, mid-flight on proof 018's fuzzy
cache demo):

> "what diagnostic is missing to help you - this is a strong
> signal we are missing a tool"

> "can you implement B now? B delivers what A needs?"

Proof 018 hit a `coincident?` returning false where the consumer's
math said it should be true (Thermometer 70.0 vs 70.3 over [0, 100]
at d=10000). The substrate gave no signal as to *why* — three
distinct theories (encoding wrong / calibration boundary / Bind
compounding) all explain the same `false`. The consumer flailed.

This arc closes that gap. `coincident-explain` is the
substrate's introspection of its own coincidence judgement.

---

## What shipped

| Slice | Module | LOC | Tests | Status |
|-------|--------|-----|-------|--------|
| 1 | `src/types.rs` — `:wat::holon::CoincidentExplanation` registered as a built-in struct (six fields: cosine, floor, dim, sigma, coincident, min-sigma-to-pass). `src/check.rs` — new `infer_polymorphic_holon_pair_to_path` helper + dispatch arm. `src/runtime.rs` — `eval_algebra_coincident_explain` reuses the existing `pair_values_to_vectors` + encoder + sigma plumbing; computes `min_sigma_to_pass = max(1, ceil((1 - cos) * sqrt(d)))`; constructs the StructValue. `wat/std/test.wat` — `:wat::test::assert-coincident` now consumes the explanation and renders the cosine / floor / dim / sigma / min-sigma-to-pass into the failure's `actual` payload via a new `:wat::test::render-coincident-explanation` helper. `docs/USER-GUIDE.md` — appendix row + a worked diagnostic walkthrough section under §The four measurements. | ~110 Rust + ~30 wat + ~50 docs | 8 new (byte-identical, near-coincident, just-below-threshold, distant, polymorphic HolonAST/Vector input, dim-reflects-router-choice, arity-mismatch, agrees-with-`coincident?`) | shipped |

**wat-rs unit-test count: 707 → 715. +8. Workspace: 0 failing.**

Build: `cargo build --lib` clean. `cargo clippy --lib`: zero
warnings. `cargo test --workspace`: 0 failures.

---

## Architecture notes

### Reuses every existing piece

The existing `:wat::holon::coincident?` already does the work — it
calls `pair_values_to_vectors` (polymorphic input), gets the
encoder via `ctx.encoders.get(d)`, reads `coincident_floor(sym)`,
computes cosine, returns the boolean. `coincident-explain` does
exactly the same dance and additionally reads `sigma` directly from
`coincident_sigma_fn`, computes the `min_sigma_to_pass` math, and
packages the six fields into a Struct.

No new encoding code. No new sigma machinery. No new
polymorphism. The arc is pure surface — exposing what's already
inside.

### `min-sigma-to-pass` math

The coincident floor is `σ/√d`; the predicate fires when
`(1 - cos) < σ/√d`. Solving for σ:

```
σ > (1 - cos) · √d
min_sigma_to_pass = max(1, ceil((1 - cos) · √d))
```

Q3 of DESIGN settled on **honest math everywhere** (no sentinel
value for the "structurally distant" case). For orthogonal pairs
(`cos ≈ 0`), the value is large; the consumer reads `cosine`
directly to see the situation isn't "near-coincident, just need
looser tolerance" but "structurally distant — no σ fix will help."
A sentinel would force a special case.

### Polymorphic over (HolonAST, Vector) inputs

Per Q4 — uses the same `pair_values_to_vectors` helper as
`cosine` / `coincident?`. Mixed inputs (one AST, one Vector)
encode the AST side at the Vector's d. Pre-encoded Vectors
short-circuit. AST/AST pairs go through the dim router. The
`dim` field reports the actual encoding d.

### Built-in struct registration

`:wat::holon::CoincidentExplanation` lives in `src/types.rs`
alongside `:wat::kernel::Failure` etc. The auto-generated
`/new` constructor + per-field accessors (`/cosine`, `/floor`,
`/dim`, `/sigma`, `/coincident`, `/min-sigma-to-pass`) land
in the symbol table at freeze time via `register_struct_methods`
— same path arc 029 set up for builtins. Tests use
`:wat::core::struct-field` for positional access since they
build a bare SymbolTable that doesn't run `register_struct_methods`.

### Type checker

New `infer_polymorphic_holon_pair_to_path` helper — same arg
discipline as the bool/f64 siblings, parameterized on the return
struct path. The `#[allow(clippy::too_many_arguments)]` is
because clippy's threshold (7) is one less than this helper needs;
the cleaner factor would be a struct, but parity with the other
inference helpers wins. Future refactor if more
`infer_polymorphic_*_to_<X>` siblings land.

---

## Test coverage

Eight tests cover every claim in the DESIGN:

- **byte-identical** → `cosine = 1.0`, `coincident = true`,
  `min-sigma-to-pass = 1`.
- **near-coincident** (Thermometer ε within window) → `cosine >
  0.99`, `coincident = true`, `min-sigma-to-pass = 1`.
- **just-below-threshold** (Thermometer ε just outside window) →
  `coincident = false`, `min-sigma-to-pass > 1` — the diagnostic
  literally tells the caller how much wider sigma needs to be.
- **distant** (unrelated atoms) → `cosine ≈ 0`, `coincident =
  false`, `min-sigma-to-pass ≥ 16` — honest math, no sentinel.
- **polymorphic input** (AST + Vector) → both shapes accepted.
- **dim reflects router** → multi-d configurations report the
  actual encoding d, not a hard-coded constant.
- **arity mismatch** → surfaces as `RuntimeError::ArityMismatch`.
- **agrees with `coincident?`** — the "doesn't lie" invariant: for
  every input pair the struct's `coincident` field equals the
  bare `:wat::holon::coincident?` result.

---

## What this unblocks

- **Proof 018** uses `coincident-explain` to determine whether
  70.0 vs 70.3 over [0, 100] at d=10000 is just-below-threshold
  (calibration), structurally distant (encoding shape), or
  already coincident at sigma=1 (substrate bug).
- **Lab umbrella 059's L1+L2 cache** can integrate the diagnostic
  into a verbose-mode lookup that prints the explanation when a
  cache miss was *expected* to hit. Catches calibration drift
  early.
- **Future ward arcs** can use `coincident-explain` for runtime
  invariant assertions that fail with the full diagnostic in the
  failure payload.

---

## What this arc deliberately did NOT add

- **`presence-explain`** — the symmetric diagnostic for `presence?`
  (arc 023's sibling predicate). DESIGN flagged as defer; ship
  `coincident-explain` first; if a presence-using consumer
  surfaces the same need, a follow-up arc shares the scaffolding.
- **String-rendering helper** (`coincident-explain-render`).
  Consumers can format the struct fields directly until a clear
  need surfaces.
- **`wat::test::assert-coincident-with-explanation`** — a
  test-friendly assert that fails with the full diagnostic in
  the failure payload. Once `coincident-explain` shipped, this
  becomes a 5-line wat helper any consumer can write —
  lab-userland or a future test-stdlib arc.

---

## The thread

- **Arc 023** — `coincident?` shipped (the algebra-grid identity
  predicate). Returned bool; no introspection.
- **Arc 024** — `set-coincident-sigma!` knob. The σ field that
  `coincident-explain` now reports.
- **Arc 037** — per-d encoders + dim router. The `dim` field that
  `coincident-explain` now reports.
- **Arc 061** — coincident? polymorphism over (HolonAST, Vector).
  The polymorphism `coincident-explain` inherits.
- **2026-04-27 (proof 018)** — proofs lane sees `coincident?`
  return false unexpectedly; can't see why.
- **2026-04-27 (DESIGN)** — proofs lane drafts arc 069; option B
  selected (general-purpose primitive over a test-flavored assert).
- **2026-04-27 (this commit)** — slice 1 ships in one commit:
  struct registration + check.rs helper + runtime impl + 8 tests
  + USER-GUIDE worked walkthrough + this INSCRIPTION.
- **Next** — proof 018 resumes its debug session with the
  diagnostic in hand.

PERSEVERARE.
