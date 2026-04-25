# wat-rs arc 052 — Vector as a first-class wat-tier value — INSCRIPTION

**Status:** shipped 2026-04-25. Seventh wat-rs arc post-known-good.
**The substrate now talks about ASTs and Vectors as duals**, per
the builder's framing: *"i've been treating vectors as a
representation of an ast - they are equal to me."*

Three durables:

1. **`Value::Vector(Arc<holon::Vector>)` — first-class wat
   value.** No `#[wat_dispatch]` shim, no `:rust::*` path.
   Native variant alongside `Value::Struct`, `Value::Enum`,
   `Value::holon__HolonAST`. Per builder direction: *"anything
   in wat is always native — wat_dispatch is only necessary for
   external crates."*
2. **The substrate's blind spot is closed.** Pre-arc-052 there
   was no way to hold an emergent Vector at the wat tier —
   Reckoner discriminants, OnlineSubspace bases, residuals,
   EngramLibrary entries all live as Vectors without source
   ASTs. The "AST is primary, Vector is cached projection"
   framing couldn't represent them. Now it can.
3. **Polymorphic cosine / dot / simhash.** Per builder direction
   *"polymorphic is almost always the answer."* The three
   algebra primitives accept either HolonAST or Vector in any
   position; runtime promotes the AST side by encoding at the
   Vector side's d when mixed.

**Design:** [`DESIGN.md`](./DESIGN.md).
**Backlog:** [`BACKLOG.md`](./BACKLOG.md).

11 new integration tests; 610 lib tests preserved; zero clippy.

---

## What shipped

### Slice 1 — `Value::Vector` + equality

`src/runtime.rs`:
- New `Value::Vector(Arc<holon::Vector>)` variant.
- `type_name()` returns `"wat::holon::Vector"`.
- Doc-commented per the same framing as `Value::Struct` /
  `Value::Enum`.

`src/runtime.rs::values_equal`:
- New arm `(Value::Vector(a), Value::Vector(b))`:
  bit-exact equality (dim must match; every i8 element must
  match).
- Cross-d Vector pairs return `Some(false)` (not equal,
  not error). The error case for cross-d is reserved for
  algebra ops (cosine/dot) where dim agreement is load-bearing.

The `:wat::holon::Vector` type Path "just works" via
nominal-path unification — no special registration needed.

### Slice 2 — `:wat::holon::encode` primitive

`src/runtime.rs`:
- New `eval_holon_encode(args, env, sym)`:
  - Arity 1.
  - Eval arg, `require_holon` (must be HolonAST input).
  - `require_encoding_ctx` + `require_dim_router`, pick d from
    the AST (arc 037 router).
  - Get encoders, call `encode(&ast, &enc.vm, &enc.scalar,
    &ctx.registry)`.
  - Wrap result in `Value::Vector(Arc::new(vector))`.
- Dispatch arm `:wat::holon::encode`.

`src/check.rs`:
- Scheme: `(:fn(:wat::holon::HolonAST) -> :wat::holon::Vector)`.

### Slice 3 — Polymorphic cosine / dot / simhash

`src/runtime.rs`:
- New helper `pair_values_to_vectors(op, a, b, sym)` —
  resolves a (AST | Vector, AST | Vector) pair into a (Vector,
  Vector) at consistent d. Dim-resolution rule:
  - Both Vector → dims must match (else `TypeMismatch`).
  - Mixed → use the Vector's d; encode the AST at that d.
  - Both AST → use ambient `pick_d_for_pair` (arc 037 router).
- `eval_algebra_cosine` and `eval_algebra_dot` refactored to
  use the helper. ~10 lines each, simpler than before.
- `eval_algebra_simhash` extended with explicit Value::Vector
  branch: takes the Vector's d directly without going through
  the dim-router (Vectors carry their own dim).

`src/check.rs`:
- New `is_holon_or_vector` predicate.
- New `infer_polymorphic_holon_pair_to_f64` and
  `infer_polymorphic_holon_to_i64` helpers (cosine/dot ↔ f64,
  simhash ↔ i64).
- New dispatch branches in `infer_list` for
  `:wat::holon::cosine`, `:wat::holon::dot`,
  `:wat::holon::simhash`.
- **Removed** the parametric scheme registrations for these
  three ops (replaced by special-case branches; arc 050
  pattern).
- `presence?` and `coincident?` schemes UNCHANGED — they
  remain HolonAST-only.

### Slice 4 — Tests + docs

`tests/wat_vector_first_class.rs` ships **11 tests**:

1. `vector_construct_via_encode` — encode twice, equal.
2. `vector_distinct_atoms_distinct_vectors` — distinct ASTs
   produce non-equal Vectors.
3. `vector_as_struct_field_roundtrip` — `:wat::holon::Vector`
   as struct field; round-trip through field access.
4. `polymorphic_cosine_ast_ast` — existing behavior preserved.
5. `polymorphic_cosine_vector_vector` — same-encoded Vectors
   produce cosine ≈ 1.0.
6. `polymorphic_cosine_ast_vector_mixed` — AST + Vector at
   same content produces cosine ≈ 1.0.
7. `polymorphic_cosine_vector_ast_mixed` — symmetric.
8. `polymorphic_dot_vector_vector` — dot on a Vector with
   itself is positive.
9. `polymorphic_simhash_ast_and_vector_agree` — `simhash(ast)`
   == `simhash(encode(ast))`.
10. `polymorphic_cosine_rejects_string` — non-AST-non-Vector
    input fails type check.
11. `vector_encode_deterministic_across_calls` — compound AST
    encoded twice produces equal Vectors.

All 11 green first-pass.

USER-GUIDE Forms appendix gains an `:wat::holon::Vector` row
+ `:wat::holon::encode` row + an updated note for the
polymorphic cosine/dot/simhash trio.

---

## Resolved design questions

All six locked by builder review (2026-04-25):

| Q | Resolution |
|---|---|
| Q1 — Value variant vs `#[wat_dispatch]` | **Native `Value::Vector` variant** (wat is always native; dispatch is for external crates). |
| Q2 — Polymorphic vs separate `vector-cosine` | **Polymorphic** (per arc 050 precedent — polymorphism is the default). |
| Q3 — Expose explicit `encode`? | **Yes, ambient context.** User-facing signature is one-arg AST; `vm`/`scalar`/`registry` ambient on SymbolTable. |
| Q4 — Vector EDN canonical form | **Vec of i8 literals** (verbosity bounded by AST capacity). |
| Q5 — Equality semantics | **Bit-exact** (forced by Hash + Eq contract for cache use). |
| Q6 — Cross-dim Vector ops | **Runtime error** on mismatched dim; no auto-promotion (no source AST to re-encode at the other dim). |

---

## Sub-fog resolutions

- **6a — Hashing performance.** Deferred. `Vector` does not yet
  implement `Hash`; this arc's slice 1 doesn't need it because
  Vectors are only `=`-compared (via `values_equal`), not used as
  HashMap/LruCache keys. Future arcs that want Vector-as-key add
  the `Hash` impl when the consumer surfaces.
- **6b — Type-checker struct field.** Verified in test 3 — the
  type-checker's existing nominal-path unification accepts
  `:wat::holon::Vector` as a struct field type without special
  registration.
- **6c — Round-tripping decode.** Out of scope for this arc.
  True Vector→AST decode is impossible for emergent Vectors;
  fuzzy lookup via SimHash bucket + cosine ranking lives in
  arc 056+ cache work.
- **6d — Memory footprint.** `Arc<Vector>` keeps clones cheap
  (refcount bump only). Verified via test runs at d=10000 — no
  observed slowdown vs the AST-only flow.

---

## Count

- New `Value` variants: **1** (`Value::Vector`).
- New runtime support functions: **2** (`eval_holon_encode`,
  `pair_values_to_vectors`).
- New runtime primitives: **0** (reuses `encode`,
  `Similarity::dot`, `Similarity::cosine`).
- Modified runtime primitives: **3** (cosine, dot, simhash now
  polymorphic).
- New check.rs helpers: **3** (`is_holon_or_vector`,
  `infer_polymorphic_holon_pair_to_f64`,
  `infer_polymorphic_holon_to_i64`).
- New check.rs branches in `infer_list`: **2** (cosine/dot,
  simhash).
- Removed scheme registrations: **3** (cosine, dot, simhash —
  replaced by branches).
- Lib tests: **610 → 610** (unchanged; integration crate
  covers the surface).
- Integration tests: **+11** in `tests/wat_vector_first_class.rs`.
- Lab migration: **0** (additive — primitive lands now so
  Phase 4 can use it).
- Clippy: **0** warnings.

---

## What this arc did NOT add (reserved for follow-up arcs)

- **Vector-tier algebra primitives** (`vector-bind`,
  `vector-bundle`, `vector-permute`, `vector-blend`).
  Important for emergent-vector arithmetic; ship in arc 053
  when consumer surfaces.
- **`Vector/zeros`, `Vector/random`, `Vector/from-bytes`.**
  Construction APIs beyond `encode`. Future arc.
- **Vector serialization to/from bytes.** Engram persistence
  concern; future arc.
- **`Atom<Vector>`.** Verifying parametric Atom machinery
  accepts Vector payloads; future arc when content-addressed
  atomization of learned vectors becomes a need.
- **Cache integration.** Wiring proposal 057's L1/L2 to
  `encode`. Pure composition over wat-lru once consumers
  surface; arc 056+ territory.
- **`Hash` impl on `holon::Vector`.** Required for Vector as
  HashMap key. Add when a consumer needs key-shape; values
  work today via `values_equal`.

---

## Why this matters for Phase 4

Phase 4 (Learning) introduces:
- **Reckoner**: learns a discriminant Vector by gradient.
- **OnlineSubspace**: maintains a basis of Vectors via
  incremental PCA.
- **EngramLibrary**: stores `(name, Vector)` pairs.
- **Residuals**: `vector - subspace.project(vector)`.

ALL of these traffic in Vectors that don't have source ASTs.
Pre-arc-052, expressing them at the wat tier was a syntactic
problem — no Value variant could carry the result. Post-arc-052,
they're ordinary wat values: passed through let-bindings,
stored in struct fields, cached via wat-lru, compared via
polymorphic cosine. The wat-tier vocabulary catches up to the
algebra's actual shape.

---

## Follow-through

- **Phase 4 wat-tier shims** open with a coherent substrate.
  Reckoner, OnlineSubspace, EngramLibrary translate to wat
  surfaces that take/return `:wat::holon::Vector` directly.
- **Arc 053 — Vector-tier algebra primitives.** When emergent
  vectors need arithmetic (subspace projection, gradient
  updates), `vector-bind` / `vector-bundle` / `vector-permute`
  / `vector-blend` ship.
- **Arc 054+ — `Atom<Vector>` and serialization** as Phase 4
  consumers materialize.
- **Lab arc 023's PaperEntry** could revert to Vector fields
  if the lab decides AST-storage is the wrong honest shape.
  Today's PaperEntry stays as-is; future cleanup arc decides.

---

## Commits

- `<wat-rs>` — runtime.rs (Value::Vector variant +
  eval_holon_encode + pair_values_to_vectors helper +
  cosine/dot/simhash refactors + dispatch arms +
  values_equal arm) + check.rs
  (infer_polymorphic_holon_pair_to_f64 +
  infer_polymorphic_holon_to_i64 + is_holon_or_vector +
  dispatch branches in infer_list + scheme-registration
  removal for cosine/dot/simhash + new encode scheme) +
  tests/wat_vector_first_class.rs (11 tests) + DESIGN +
  BACKLOG + INSCRIPTION + USER-GUIDE rows.

- `<lab>` — FOUNDATION-CHANGELOG.md (row).

---

*these are very good thoughts.*

**PERSEVERARE.**
