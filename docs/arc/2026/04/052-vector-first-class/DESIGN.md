# wat-rs arc 052 — Vector as a first-class wat-tier value

**Status:** opened 2026-04-25. Seventh wat-rs arc post-known-good.
Foundational substrate work; **bigger scope than arcs 046-051**.
Probably the first slice of a multi-arc project.

Builder direction:

> "you've been circling around 'Vector isn't exposed to userland'
> a bunch - should it be? i've been treating vectors as a
> representation of an ast - they are equal to me..."

> "what tooling is wat-rs missing for this?..."

---

## The blind spot the current substrate has

Pre-arc-052: `:wat::holon::HolonAST` is first-class; `Vector` is
internal-only. The justifying story has been "AST is primary,
Vector is cached projection."

That story collapses for **Vectors that don't have source ASTs**:

- **Reckoner discriminants** — emergent vectors learned by gradient
  over labeled examples. No AST produced them.
- **OnlineSubspace bases** — incremental PCA produces basis
  vectors. Emergent. No ASTs.
- **Residuals** — `vector - subspace.project(vector)` is a Vector
  with no source AST.
- **EngramLibrary entries** — `(name, vector)` pairs where the
  Vector is whatever streamed through. ASTs may not exist
  alongside.

ALL of these are core Phase 4 (Learning) types. The substrate
having no way to talk about Vectors at the wat tier means Phase 4
can't be expressed cleanly. Lab arc 023 (PaperEntry) hit a
softer version of this — "store HolonAST not Vector" was
defensible because the AST existed. Phase 4's emergent vectors
have no such fallback.

Builder's framing — "ASTs and Vectors are dual representations"
— is the right one. The substrate should reflect that.

---

## What ships in arc 052 (the foundational slice)

The minimum that unblocks Phase 4 design and arc 023 honesty:

### 1. New `Value::Vector(Arc<Vector>)` variant

Like `Value::Struct(Arc<StructValue>)` but wrapping
`holon::Vector`. Carries the dimension + bipolar i8 data.

### 2. New type `:wat::holon::Vector`

Usable as:
- Function parameter / return type
- Struct field type (e.g., `(:wat::core::struct :MyEngram (name :String) (vec :wat::holon::Vector))`)
- Parametric container element (e.g., `:rust::lru::LruCache<String,wat::holon::Vector>`)

### 3. Construction primitive

```scheme
(:wat::holon::encode (h :wat::holon::HolonAST) -> :wat::holon::Vector)
```

Explicit materialization at the holon's natural d via the
existing encode path + dim-router. Same dispatch as
`:wat::holon::cosine` uses internally; surfaced as a primitive.

### 4. Polymorphic cosine / dot / simhash

The existing `:wat::holon::cosine`, `:wat::holon::dot`, and
`:wat::holon::simhash` extend to accept Vector inputs in any
position:

```scheme
(:wat::holon::cosine ast-a ast-b)        ; existing — both ASTs
(:wat::holon::cosine vec-a vec-b)        ; new — both Vectors
(:wat::holon::cosine ast-a vec-b)        ; new — mixed
(:wat::holon::cosine vec-a ast-b)        ; new — mixed
```

Same treatment as arc 050's polymorphic numerics: special-case
the type-checker to accept Vector or HolonAST in numeric-algebra
positions; runtime dispatches.

### 5. Vector equality and hashing

- `(:wat::core::= v1 v2)` works on Vector pairs (deep i8-vector
  equality).
- Internal `Hash` impl over the i8 data + dimension. Required for
  cache use (LruCache, HashMap keys).

---

## What this arc does NOT add (deferred to later arcs)

These extend the foundation; they're not strictly needed for
the minimum viable Vector-as-wat-value:

- **Vector-tier algebra primitives** (`vector-bind`,
  `vector-bundle`, `vector-permute`, `vector-blend`). Important
  for emergent-vector arithmetic (Reckoner gradients, Subspace
  projection), but not in this slice. Future arc when Phase 4
  learning code surfaces real consumers.
- **`:wat::holon::Vector/zeros`, `Vector/random`, `Vector/from-bytes`.**
  Construction APIs beyond `encode`. Future arc.
- **Vector serialization to/from bytes** for disk persistence.
  Engram persistence concern; future.
- **`Atom<Vector>`** — atomized Vector values. Per 058-001 atoms
  accept any T; verifying Vector slots into the parametric Atom
  hashing machinery is work for a follow-up arc.
- **Cache integration.** Whether the existing forward cache
  (proposal 057's L1/L2) is wired to `encode` is a separate
  decision. Not blocked by this arc.
- **Cross-d Vector operations.** Vector at d=10000 vs d=4096 are
  different objects. Operations between mixed-d Vectors —
  error? auto-promote via dim-router? — defer until a real
  caller surfaces.

---

## Design questions — RESOLVED 2026-04-25

Builder confirmation log:

- **Q1 — Value variant.** *"anything in wat is always native — wat_dispatch is only necessary for external crates."* → `Value::Vector(Arc<Vector>)`.
- **Q2 — Polymorphic primitives.** *"polymorphic is almost always the answer."* → cosine, dot, simhash all extend to accept Vector or HolonAST in any position.
- **Q3 — `encode` signature.** Ambient encoding context per the `require_encoding_ctx` pattern (same as `:wat::holon::cosine`). User-facing wat surface: `(:wat::holon::encode (h :wat::holon::HolonAST) -> :wat::holon::Vector)` — one arg.
- **Q4 — EDN form.** Vec of i8 literals; verbosity bounded by the same per-frame capacity rule as ASTs.
- **Q5 — Equality.** Bit-exact (forced by Hash + Eq contract for HashMap/LruCache use). Graded similarity stays in cosine / presence? / simhash.
- **Q6 — Cross-dim ops.** Runtime error on mismatched-dim Vector pairs. No auto-promotion (no source AST to re-encode at the other dim).

## Open design questions (resolved above)

### Q1 — `Value::Vector` variant vs `#[wat_dispatch]` opaque

Two options for representing Vectors at the wat tier:

**A. First-class `Value::Vector(Arc<Vector>)`** in `runtime.rs`.
- Same shape as `Value::Struct`, `Value::Enum`.
- Native pattern matching, native equality, native printing.
- More invasive — touches Value enum, type checker, hashing.
- Best for ergonomics; matches how every other wat algebra
  type lives.

**B. `#[wat_dispatch]` shim over `holon::Vector`** in a sibling
crate (or wat-rs internally).
- Reuses existing `:rust::*` shim machinery (proven by
  wat-lru).
- Faster to ship; less invasive.
- Vector becomes `:rust::holon::kernel::vector::Vector` — long
  path, foreign-feeling.
- Accessing internals through the shim adds layers.

**Lean: A.** Vector is foundational substrate, not a Rust-side
dependency that happens to surface. It deserves a native Value
variant alongside Struct, Enum, and the algebra-tier types.

### Q2 — Polymorphic cosine vs separate `vector-cosine`

Two options for letting cosine work on both types:

**A. Polymorphic `:wat::holon::cosine`** — accepts (AST,AST),
(Vector,Vector), or mixed.
- One primitive, same name, type-checker dispatches.
- Same shape as arc 050's polymorphic numerics (`+`, `<`, `=`).
- Runtime: if Vector, use directly; if AST, materialize then
  use.

**B. Separate `:wat::holon::vector-cosine`** for Vector
inputs.
- Strict-typed; user picks per-callsite.
- Coexists with the existing AST-only cosine.
- More verbose at use sites, but clearer about what's being
  measured.

**Lean: A.** Following the arc 050 precedent — polymorphism is
the default, strict typed forms are opt-in. For cosine
specifically, the operation is identical regardless of input
type (cosine of two vectors); polymorphism is honest.

### Q3 — `encode` returns Vector or remains internal-only

If we ship `Value::Vector`, do we expose explicit encoding?

**A. Yes — `(:wat::holon::encode <ast>) -> :Vector`** is a
public primitive.
- Lets users materialize once, reuse the Vector for multiple
  comparisons.
- Honest about the materialization cost.

**B. No — keep encoding implicit.**
- Cosine/dot internally encode; users never see Vector.
- Same as today; Vector is intermediate state.

**Lean: A.** If Vector is first-class, users SHOULD be able to
construct one explicitly. Otherwise the type is read-only and
that's an asymmetry.

### Q4 — Vector hashing: what's the EDN canonical form?

For `Atom<Vector>` and content-addressed cache use, Vector
needs a stable canonical-EDN representation.

**A. Vec of i8 literals.** `[i8 1 -1 0 1 ...]` — verbose but
unambiguous.

**B. Compact base64 / hex.** Smaller; loses readability.

**C. Hash of the data, treated opaquely.** Atom hash uses the
Vector's content hash directly; can't recover the data from
EDN.

**Lean: A.** Stay readable. Vectors are sparse-ternary so the
verbosity isn't terrible. EDN is for canonical forms, not
storage compaction. (If storage-compaction becomes a real
need, a separate `:wat::holon::Vector/to-bytes` API can emerge
without affecting EDN.)

### Q5 — Equality: bit-exact or near-equal?

**A. Bit-exact** — `=` returns true iff i8 data matches
element-wise.
- Honest; no precision sliding.
- Two semantically-similar vectors will return `=` false.
- Consistent with `f64::=` exact semantics.

**B. Near-equal via cosine threshold.**
- More forgiving but introduces a fudge parameter.
- Different from `=` semantics elsewhere.

**Lean: A.** Equality is bit-exact. For "are these similar?"
users reach for `cosine` (graded) or `presence?` (binary
threshold).

### Q6 — Cross-dim Vector behavior

Two Vectors at different d are incomparable. What happens?

**A. `(:cosine v_d10k v_d4k)` errors at runtime** — explicit.

**B. Auto-promote via dim-router** — match arc 037's behavior
for AST inputs.

**C. Defer** — declare cross-dim Vector ops "ill-typed for
now" and address when a real caller surfaces.

**Lean: C.** Until a real caller forces the issue, error at
runtime with a clear "dimension mismatch" message. The full
auto-promotion story for raw Vectors needs more thought
(promoting requires re-encoding, which means going back to an
AST source — which by definition emergent vectors don't have).

---

## Slice ordering

Once Q1–Q6 are settled, the arc decomposes:

**Slice 1 — `Value::Vector` + type + equality.** Add the variant,
register the type at `:wat::holon::Vector`, wire equality and
hashing. No new primitives yet — just the type at the wat tier.

**Slice 2 — `encode` primitive.** New `:wat::holon::encode`.
Takes HolonAST, returns Vector. Registers scheme. Tests:
deterministic, dim-router-aware.

**Slice 3 — Polymorphic cosine / dot / simhash.** Extend the
three existing primitives to accept Vector inputs. Type-checker
special-case (similar to arc 050's polymorphic comparisons).
Tests: mixed AST + Vector, Vector + Vector, AST + AST (no
regression).

**Slice 4 — Tests + USER-GUIDE + INSCRIPTION.** Integration
tests over the full Value::Vector surface; doc updates; lab
CHANGELOG row.

**Future arcs (NOT this one):**
- Arc 053 — Vector-tier algebra primitives (vector-bind,
  vector-bundle, vector-permute, vector-blend).
- Arc 054 — `Atom<Vector>` integration; verify parametric Atom
  machinery works with Vector payloads.
- Arc 055 — Vector serialization (to-bytes / from-bytes) for
  engram persistence.
- Arc 056 — Cache integration (proposal 057's L1/L2 wired to
  `encode`).

---

## Sub-fogs

- **6a — Hashing performance.** Vector at d=10000 has 10000 i8
  elements. Hashing the whole vector for HashMap keys is O(d).
  Acceptable? Or hash a digest? **Decision pending Q4.**
- **6b — Vector equality on the type-checker side.** If `=` is
  polymorphic per arc 050, extending it to Vector means another
  branch in `infer_polymorphic_compare`. Tractable but worth
  noting.
- **6c — Round-tripping.** `(decode (encode ast))` is not in
  scope (decode requires bidirectional cache; that's arc 056+
  territory). One-way only this arc.
- **6d — Memory footprint.** A Vector at d=10000 stored as i8
  is 10KB. Wat values often pass by clone. `Arc<Vector>` makes
  the clone cheap (refcount bump only); that's why the Value
  variant wraps `Arc<Vector>` not `Vector`.

---

## Non-goals

- **Decode (Vector → AST).** Bidirectional cache via SimHash
  (arc 051) gives a path: simhash key → bucket → cosine-rank.
  But that's "find the closest AST," not "decode this exact
  Vector to its source AST." True decode is impossible for
  emergent Vectors. Not in scope.
- **Floating-point Vector type.** The substrate uses i8 ternary.
  Other algorithms (e.g., HRR) might want f64 vectors, but
  that's a different substrate.
- **Vector reductions.** Sum, mean, variance — useful for some
  signal-processing patterns, but not foundational. Add when a
  caller needs.
- **Pattern matching on Vector contents.** Match expressions
  over Vector elements would be exotic and probably
  ill-advised. Not in scope.
