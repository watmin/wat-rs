# Arc 069 — `:wat::holon::coincident-explain` (the diagnostic primitive)

**Status:** PROPOSED 2026-04-27. Pre-implementation reasoning artifact.
**Predecessors:** arc 023 (`coincident?` predicate), arc 037 (per-d
encoders + sigma machinery), arc 057 (typed HolonAST leaves), arc 067
(flat default dim router).
**Downstream consumer:** holon-lab-trading proof 018 (the dual-store
fuzzy cache demo) — testing surfaced a failure mode where
`coincident?` returned `false` for two HolonASTs whose Thermometer
leaves should be coincident at default sigma per the consumer's
math, but the substrate gives no signal as to *why* it's false.
Future consumers in proofs / wards / runtime debugging will use this
primitive whenever a coincidence judgement disagrees with
expectation.

Builder direction (2026-04-27, mid-flight on proof 018's fuzzy
cache demo):

> "what diagnostic is missing to help you - this is a strong
> signal we are missing a tool"

Followed by, after the assistant proposed two options (lighter
test-flavored assert vs general-purpose primitive):

> "can you implement B now? B delivers what A needs?"

The recognition: when `coincident?` returns false unexpectedly,
the consumer has no way to inspect *why*. They can compute the
expected cosine from a mental model of the encoding, but if the
substrate's actual cosine differs (encoding, normalization,
binding-effect), the consumer can't see it. The substrate has
`:wat::holon::cosine` for the raw scalar, but no single primitive
that bundles the cosine WITH the threshold WITH the encoded
dimension WITH a "would-pass-at-sigma" suggestion.

This arc ships that bundled diagnostic.

---

## Why this arc, why now

**Concrete failing case from proof 018:**

```scheme
(let* (((a :wat::holon::HolonAST)
         (:wat::holon::Bind
           (:wat::holon::Atom "rsi-thought")
           (:wat::holon::Thermometer 70.0 0.0 100.0)))
       ((b :wat::holon::HolonAST)
         (:wat::holon::Bind
           (:wat::holon::Atom "rsi-thought")
           (:wat::holon::Thermometer 70.3 0.0 100.0))))
  (:wat::holon::coincident? a b))   ;; → false (expected true)
```

Consumer's mental model: bipolar Thermometer over [0, 100] with
`|delta|=0.3` gives `cos = 1 - 2·0.3/100 = 0.994`. At d=10000,
sigma=1, floor = 0.01, threshold cos > 0.99 → coincident.

But the substrate returns false. The consumer has three theories
and no way to distinguish them without reading runtime source:

1. Mental model of the encoding is wrong (cos is actually 0.6, not 0.994)
2. Calibration boundary (cos is 0.989, just under threshold)
3. Bind compounding degrades the cosine (cos of inner Thermometers ≠ cos of Bind-wrapped pair)

Each fix is different. Without diagnostics, the consumer flails.

**The pattern is not unique to the trader.** Every consumer that
makes a coincidence-based decision (caches, classifiers, gates,
wards) will hit this same blind spot eventually. The substrate
should expose its own measurement.

---

## What's already there (no change needed)

| Surface | Status |
|---------|--------|
| `:wat::holon::cosine` (`a b -> :f64`) | shipped — raw cosine of two HolonASTs |
| `:wat::holon::coincident?` (`a b -> :bool`) | shipped (arc 023) — `(1 - cos) < floor` |
| `Encoders::coincident_floor(sym)` | shipped — current floor at the encoder's d |
| `SymbolTable::coincident_sigma_fn` | shipped — per-d sigma function (arc 024) |
| `pair_values_to_vectors` | shipped (arc 052) — encodes both inputs at consistent d |
| Polymorphism over (HolonAST, Vector) inputs | shipped (arc 061) — both inputs work |

The plumbing all exists. What's missing is the bundled-output
primitive that exposes it.

## What's missing (this arc)

| Op / change | What it does |
|----|----|
| `wat::holon::CoincidentExplanation` (new built-in struct) | Tagged struct with six fields: `cosine`, `floor`, `dim`, `sigma`, `coincident`, `min-sigma-to-pass` |
| `:wat::holon::coincident-explain` (new primitive) | `(a b) → :wat::holon::CoincidentExplanation`. Polymorphic over HolonAST/Vector pairs (same shape as `cosine` / `coincident?`). |
| `eval_algebra_coincident_explain` in `runtime.rs` | The Rust impl. Calls the existing pair_values_to_vectors machinery, computes cosine, reads floor + sigma + dim, computes min-sigma-to-pass, returns the struct. |
| USER-GUIDE row + a worked example | Doc surface for the new primitive. |

Five pieces. Most of the cost is the struct registration + the new
primitive's eval function. The math is a handful of f64 operations.

---

## The new surface

### `wat::holon::CoincidentExplanation`

A built-in struct (registered via the same path as Option/Result/
StepResult). Lives in the `wat::holon` namespace beside the other
algebra-grid types.

```scheme
(:wat::core::struct :wat::holon::CoincidentExplanation
  (cosine :f64)              ;; the raw cosine of the two encoded vectors
  (floor :f64)               ;; the current coincident floor (sigma/sqrt(d))
  (dim :i64)                 ;; the dimension at which encoding happened
  (sigma :i64)               ;; the current sigma (from coincident_sigma_fn)
  (coincident :bool)         ;; whether (1 - cosine) < floor — same answer as coincident?
  (min-sigma-to-pass :i64))  ;; smallest sigma at which the pair would coincide
```

Rust-side:

```rust
pub struct CoincidentExplanation {
    pub cosine: f64,
    pub floor: f64,
    pub dim: i64,
    pub sigma: i64,
    pub coincident: bool,
    pub min_sigma_to_pass: i64,
}
```

### `:wat::holon::coincident-explain`

```
:wat::holon::coincident-explain
  :  :HolonAST × :HolonAST → :wat::holon::CoincidentExplanation
```

Polymorphic over (HolonAST, Vector) inputs in the same way
`coincident?` is (arc 061). When inputs are mixed (one AST, one
Vector), encodes the AST at the Vector's dim. When both are AST,
goes through the dim router. The d returned in the struct is the
dim where comparison happened.

**`min-sigma-to-pass` math:**

```
We need (1 - cos) < sigma/sqrt(d), so:
  sigma > (1 - cos) * sqrt(d)
  min_sigma_to_pass = ceil((1 - cos) * sqrt(d))
```

Edge cases:
- `cos >= 1.0 - 1.0/sqrt(d)` (already coincident at sigma=1):
  `min_sigma_to_pass = 1`.
- `cos = 1.0` exactly (byte-identical encoded vectors):
  `min_sigma_to_pass = 1` (always coincident).
- `cos <= 0.0` (orthogonal or anti-correlated):
  `min_sigma_to_pass` is whatever ceil((1 - cos) * sqrt(d)) gives —
  honestly large; consumers can read the cosine directly to see
  the situation is not "near-coincident, just need looser tolerance"
  but "structurally distant."

---

## Decisions to resolve

### Q1 — Struct vs Tuple return

Two options:

- **(a) Struct** `CoincidentExplanation` with named fields.
- **(b) 6-tuple** `(f64, f64, i64, i64, bool, i64)` with documented positions.

**Recommended: (a) struct.** Six fields is enough that positional
access becomes confusing. Named fields are self-documenting.
Matches the substrate's convention for multi-field returns
(`Prediction`, `BundleResult`, etc.).

### Q2 — `min-sigma-to-pass` when already coincident

Two options:

- **(a)** Return 1 (the smallest meaningful sigma).
- **(b)** Return the current sigma (whatever's set; says "you're
  passing at the current setting").

**Recommended: (a) 1.** The field's semantic is "smallest sigma at
which this pair would coincide" — independent of current sigma.
"Already coincident at sigma=1" is the most informative answer.

### Q3 — `min-sigma-to-pass` when cos < 0

Two options:

- **(a)** Return ceil((1 - cos) * sqrt(d)) honestly — could be huge.
- **(b)** Return `i64::MAX` or some sentinel meaning "won't coincide
  at any reasonable sigma."

**Recommended: (a) honest math.** Consumers can read the cosine
directly to see this is structural. A sentinel value would just
add a special case the caller has to handle.

### Q4 — Polymorphism over HolonAST / Vector inputs

`coincident?` (arc 061) accepts any combination of HolonAST and
Vector. `coincident-explain` should match.

**Recommended: yes, full polymorphism.** Reuse `pair_values_to_vectors`
(the same helper `coincident?` and `cosine` use).

### Q5 — Naming

Three candidates considered:

- **`coincident-explain`** — verb-noun; matches `macroexpand`
  shape (an existing substrate idiom).
- **`coincident-info`** — drier but ambiguous (what info?).
- **`why-not-coincident?`** — predicate-shape; only useful when
  result is false; awkward for the case where coincidence holds
  and the consumer wants the cosine anyway.

**Recommended: `coincident-explain`.** It explains the coincidence
judgement — tells the full story, not just the false-case story.

### Q6 — Whether to ship the test-friendly wrapper too

Could ship `wat::test::assert-coincident-with-explanation` as a
companion: an assert that fails with the full diagnostic in the
failure payload.

**Recommended: NO, defer.** Once `coincident-explain` ships, the
test-side wrapper is a 5-line wat helper any consumer can write.
Keep this arc focused on the substrate primitive; the wrapper can
land in lab-userland or a future test-stdlib arc.

### Q7 — Tests

Inline in `src/runtime.rs::mod tests`. Cover:

- **Already coincident at sigma=1:** Two byte-identical
  HolonASTs. Result has `cosine = 1.0`, `coincident = true`,
  `min-sigma-to-pass = 1`.
- **Near-coincident:** Thermometer values within tolerance over
  some range. Result has `cosine` in (0.99, 1.0), `coincident =
  true`, `min-sigma-to-pass = 1`.
- **Just below threshold:** Thermometer values just outside
  tolerance. Result has `cosine` in (0.98, 0.99), `coincident =
  false`, `min-sigma-to-pass = 2`.
- **Distant:** Two unrelated HolonASTs. Result has `cosine`
  near 0, `coincident = false`, `min-sigma-to-pass = ceil((1 - cos) * 100)` for d=10000.
- **Polymorphic input — HolonAST + Vector:** Confirm the
  primitive accepts mixed inputs and reports the d at which the
  comparison happened.
- **Vectors at different d:** Confirm cross-d Vector pairs error
  with `TypeMismatch` (same as `coincident?` per arc 061).

About 8 test cases total, ~80 lines.

---

## What ships

One slice. Pure substrate addition.

### Files touched

- `src/runtime.rs` — `Value::holon__CoincidentExplanation` variant (or
  reuse `Value::Struct` with the registered struct type — match
  whatever the existing conventions for built-in structs use);
  `eval_algebra_coincident_explain`; dispatcher entry for
  `:wat::holon::coincident-explain`.
- `src/freeze.rs` — register `wat::holon::CoincidentExplanation` as
  a built-in struct alongside the others.
- `docs/USER-GUIDE.md` — appendix row + a worked example.

### Acceptance criteria

- All 8 inline tests pass.
- `cargo clippy --lib` clean.
- USER-GUIDE.md compiles.
- One commit; arc 069 marker in commit message.
- INSCRIPTION.md alongside the DESIGN, post-ship.

### Estimated effort

~150 lines Rust + ~80 lines tests + ~50 lines docs. Single arc;
single slice; pattern matches arc 068's shape.

---

## Open questions (defer to inscription)

- **Whether to expose the same diagnostic for `presence?`** (arc
  023's sibling predicate). `presence-explain` would be the
  symmetric add. Defer; ship coincident-explain first; if a
  presence-using consumer surfaces the same need, file
  `presence-explain` as a follow-up arc with shared scaffolding.
- **Whether to add a string-rendering helper.** A
  `coincident-explain-render` that produces a human-readable
  string from the struct could be useful for failure-time
  diagnostic prints. Defer; consumers can format the struct
  fields directly until a clear need surfaces.
- **Whether to track which dimension actually fired** when the
  router has multiple tiers. Today (post arc 067) the default is a
  single tier at d=10000; multi-tier configurations may want the
  diagnostic to expose which tier was used. Trivial to add since
  the eval already calls the router; defer until multi-tier returns.

---

## Slices

One slice. Pattern matches arcs 058–068. Single commit.

## Consumer follow-up

After this arc lands:

- **Proof 018** uses `coincident-explain` in a diagnostic pass to
  determine why 70.0 vs 70.3 over R=100 isn't coinciding (or
  confirms it should and surfaces a substrate bug).
- The trader's L1+L2 cache (umbrella 059, slice 1) integrates
  `coincident-explain` into a verbose-mode lookup that prints the
  diagnostic when a cache miss was *expected* to hit. Catches
  calibration drift early.
- Future ward arcs can use `coincident-explain` for runtime
  invariant assertions: "this pair should coincide; if not, fail
  with the explanation."

The diagnostic loop closes: substrate gap surfaces during proof
work → arc DESIGN → arc shipped → consumer uses the now-honest API
→ proof 018's debug session has the data it needs.
