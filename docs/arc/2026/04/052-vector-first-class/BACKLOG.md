# wat-rs arc 052 — Vector as a first-class wat-tier value — BACKLOG

**Shape:** four slices for the foundational arc. Three follow-up
arcs sketched but separate.

**Pre-flight:** Q1–Q6 in DESIGN.md need builder review before
slice 1. Defaults proposed; explicit confirmation requested.

---

## Slice 1 — `Value::Vector` + type registration + equality

**Status: blocked on Q1–Q5 confirmation.**

`src/runtime.rs`:
- New `Value::Vector(Arc<holon::Vector>)` variant.
- `type_name()` returns `"Vector"`.
- Debug derive (or manual — print `Vector(d=10000)` summary).

`src/check.rs`:
- Register `:wat::holon::Vector` as a built-in type.
- `holon_vector_ty()` helper.

`src/runtime.rs::values_equal`:
- New arm `(Value::Vector(a), Value::Vector(b))` →
  bit-exact i8 array comparison.
- Cross-d Vectors → mismatch → equality false (not error).

Hashing — required for cache containers:
- Implement `Hash` impl over `holon::Vector` (currently may not
  exist in holon-rs; verify and add if needed).
- Wat `hashmap_key` recognizes Vector as a hashable key (or
  not — TBD by Q4).

**Sub-fogs:**
- **1a — Hash impl on `holon::Vector`.** Verify it exists; add
  if missing. Choose Q4-A representation (Vec of i8 literals
  for EDN; full-data hash for runtime hashing).
- **1b — Type checker's struct field type validation.** Verify
  `:wat::holon::Vector` works as a struct field via the existing
  unification path; add to test coverage.

## Slice 2 — `:wat::holon::encode` primitive

**Status: blocked on Q3 confirmation; slice 1 prerequisite.**

`src/runtime.rs`:
- New `eval_holon_encode(args, env, sym) -> Result<Value, RuntimeError>`.
  - Arity 1.
  - Eval arg, `require_holon`.
  - `require_encoding_ctx`, `require_dim_router`, pick d.
  - Get encoders, call `encode(&ast, &enc.vm, &enc.scalar, &ctx.registry)`.
  - Wrap result in `Value::Vector(Arc::new(vector))`.
- Dispatch arm `:wat::holon::encode`.

`src/check.rs`:
- Scheme: `(:fn(:wat::holon::HolonAST) -> :wat::holon::Vector)`.

**Sub-fogs:**
- **2a — Symmetry with cosine's d-picking.** Verify
  `eval_holon_encode` uses the same dim-router pick logic as
  `eval_algebra_cosine` to avoid d-divergence between an
  encoded Vector and a downstream cosine call.

## Slice 3 — Polymorphic cosine / dot / simhash

**Status: blocked on Q2 confirmation; slices 1+2 prerequisite.**

`src/runtime.rs`:
- Extend `eval_algebra_cosine`, `eval_algebra_dot`,
  `eval_algebra_simhash` to accept `Value::Vector` inputs in
  any position. Dispatch:
  - Both Vector: skip materialization, use vectors directly.
  - Both HolonAST: existing path.
  - Mixed: materialize the AST side, then operate.
- Dim must match for Vector-Vector (error if not — Q6 default).

`src/check.rs`:
- Replace existing scheme registrations for cosine/dot/simhash
  with special-case branches in `infer_list` (mirroring arc
  050's polymorphic numeric pattern).
- Branches accept (HolonAST | Vector) in either position;
  return `:f64` for cosine/dot, `:i64` for simhash.

**Sub-fogs:**
- **3a — Existing scheme removal.** Confirm the existing
  cosine/dot/simhash schemes can be removed (replaced by
  branch-based inference) without affecting other call sites
  that rely on the parametric form.

## Slice 4 — Integration tests + docs

**Status: obvious in shape** (once slices 1 – 3 land).

`tests/wat_vector_first_class.rs` ships ~12 tests:

1. **Construct + equality** — `(encode (Atom "x"))` returns
   Value::Vector; calling encode twice produces equal Vectors.
2. **Determinism** — encoding the same AST in two programs
   produces equal Vectors.
3. **Type checking** — `:wat::holon::Vector` as struct field;
   round-trip through the field.
4. **Polymorphic cosine — both Vectors** —
   `(cosine (encode a) (encode a))` returns 1.0 (within float
   tolerance).
5. **Polymorphic cosine — both ASTs** — existing behavior
   preserved.
6. **Polymorphic cosine — mixed** —
   `(cosine ast (encode ast))` returns 1.0.
7. **Polymorphic dot — same surface as cosine.**
8. **Polymorphic simhash — Vector input** —
   `(simhash (encode ast))` matches `(simhash ast)`.
9. **Cross-dim error** — encoding at one d, comparing at
   another should error or auto-handle (per Q6).
10. **Vector in LruCache** —
    `:rust::lru::LruCache<String,wat::holon::Vector>` round-trips
    a Vector through the cache.
11. **Vector pattern matching** — basic match-on-Vector-equality
    (uses values_equal).
12. **Distinct ASTs produce distinct Vectors** — orthogonality
    sanity.

USER-GUIDE Forms appendix gains rows for `:wat::holon::encode`
+ a Vector-handling note for the existing cosine/dot/simhash.

INSCRIPTION captures the Q1–Q6 resolutions, the slice-by-slice
delta, and the architectural shift (Vector as first-class).

`holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
gets a row.

---

## Future arc sketches (NOT this arc)

- **Arc 053 — Vector-tier algebra primitives.**
  `vector-bind`, `vector-bundle`, `vector-permute`,
  `vector-blend`. Lets emergent vectors compose without
  round-tripping through ASTs.
- **Arc 054 — `Atom<Vector>` integration.** Verify
  parametric-atom machinery accepts Vector payloads. Enables
  content-addressed atomization of learned vectors.
- **Arc 055 — Vector serialization.** `to-bytes` / `from-bytes`
  for engram persistence. Enables disk-backed engram
  libraries.
- **Arc 056 — Composition cache.** Wire L1/L2 (proposal 057)
  to `encode`. Forward cache: AST hash → Vector. Reverse
  cache: SimHash key → Vec<HolonAST>. Pure composition over
  wat-lru once Vector is wat-tier.

---

## Sub-fogs (cross-cutting)

- **A — Memory footprint of Value::Vector.** A Vector at
  d=10000 is 10KB of i8 data. `Arc<Vector>` keeps clone cheap.
  Verify wat values flowing through let-bindings + function
  calls don't accidentally deep-clone.
- **B — Hash performance.** O(d) hashing for HashMap keys at
  d=10000 is ~10μs per hash. Tolerable for cache use; may
  warrant a digest-based shorthand if hot-path. Defer to
  measurement.
- **C — Wat-lru integration.** wat-lru's value type is
  `Value`; verify `Value::Vector` doesn't trip the
  primitives-only key check (it's a value not a key, so it
  shouldn't, but verify).
