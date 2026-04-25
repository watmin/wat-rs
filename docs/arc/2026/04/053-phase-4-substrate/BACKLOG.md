# wat-rs arc 053 — Phase 4 substrate — BACKLOG

**Shape:** six slices. Largest arc this session. Each slice is
shippable; commit between slices to surface progress.

---

## Slice 1 — Vector algebra primitives

**Status: ready** (post-arc-052).

`src/runtime.rs`:
- 4 new dispatch arms:
  - `:wat::holon::vector-bind` → `Primitives::bind(&va, &vb)` → `Value::Vector`
  - `:wat::holon::vector-bundle` → `Primitives::bundle(refs)` → `Value::Vector`
  - `:wat::holon::vector-blend` → `Primitives::blend_weighted(va, vb, w1, w2)` → `Value::Vector`
  - `:wat::holon::vector-permute` → `Primitives::permute(va, k)` → `Value::Vector`
- Helpers: `require_vector`, `require_vec_of_vectors` (for bundle).

`src/check.rs`:
- 4 schemes (all strict-typed; no polymorphism — these are
  Vector-only, no AST overload needed since users have `encode`
  to produce Vectors first).

Tests: 6 cases (one per primitive + 1 for bind-then-unbind sanity
+ 1 for bundle of empty/singleton).

## Slice 2 — OnlineSubspace

**Status: ready** (slice 1 not strictly required — Vector arith
is independent of subspace).

`src/runtime.rs`:
- New `Value::OnlineSubspace(Arc<ThreadOwnedCell<OnlineSubspace>>)`.
- `type_name()` returns `"wat::holon::OnlineSubspace"`.
- 10 dispatch arms for the methods listed in DESIGN section 2.
- `values_equal` rejects (returns `None`).

`src/check.rs`:
- 10 schemes.

Tests: ~6 cases (construct, update changes residual, project
returns Vector, eigenvalues length matches k, dim/k/n queries,
basic shape).

## Slice 3 — Reckoner + supporting types

**Status: ready** (slice 2 not required).

`src/runtime.rs`:
- New `Value::Reckoner(Arc<ThreadOwnedCell<Reckoner>>)`.
- 8 dispatch arms.
- Prediction packed as Tuple in observe/predict.
- Label as i64 throughout.

`src/check.rs`:
- 8 schemes.

Tests: ~6 cases (discrete construct + observe + predict,
continuous construct + observe + predict, resolve + curve,
labels accessor).

## Slice 4 — Engram

**Status: ready** (slice 5 needs this for `/get` if exposed; we
defer get for now, so order doesn't matter).

`src/runtime.rs`:
- New `Value::Engram(Arc<RefCell<Engram>>)`.
- 5 dispatch arms.

`src/check.rs`:
- 5 schemes.

Tests: minimal (~3 cases — name, eigenvalue-signature, residual).

## Slice 5 — EngramLibrary

**Status: ready after slice 2** (needs OnlineSubspace).

`src/runtime.rs`:
- New `Value::EngramLibrary(Arc<ThreadOwnedCell<EngramLibrary>>)`.
- 6 dispatch arms.
- `/match-vec` returns `Value::Vec` of `Value::Tuple((String, f64))`.

`src/check.rs`:
- 6 schemes.

Tests: ~6 cases (construct, add a subspace, match-vec returns
sorted, len/contains/names accessors, empty library matches
empty Vec).

## Slice 6 — Docs

**Status: obvious in shape** (once slices 1 – 5 land).

- `docs/arc/2026/04/053-phase-4-substrate/INSCRIPTION.md`.
- `docs/USER-GUIDE.md` Forms appendix gains rows for the four
  new types + their core methods.
- Lab `FOUNDATION-CHANGELOG.md` row.
- wat-rs commit + push.
- Lab repo separate commit + push for the CHANGELOG row.

---

## Sub-fogs (cross-cutting)

- **Test crate startup.** Some tests need a default dim-router
  set via `(:wat::config::set-dim-router! ...)`. Verify default
  is sufficient or wire a small test fixture.
- **Vector→Vec<f64> conversion overhead.** Each method call
  allocates. Acceptable for arc 053; future arcs can optimize
  with batch APIs if hot-path.
- **Memory footprint.** OnlineSubspace at d=10000 with k=16 holds
  the basis matrix. ~1.3 MB per subspace. EngramLibrary holds
  many; user manages capacity.
