# wat-rs arc 053 — Phase 4 substrate (Vector algebra + learning types as native wat values)

**Status:** opened 2026-04-25. Eighth wat-rs arc post-known-good.
**Largest arc this session.** Foundational substrate for Phase 4
of the trading lab (Reckoner + OnlineSubspace + EngramLibrary).

Builder direction:

> "we don't need wat_dispatch - just build what we need - holon
> is an explicit dep - if you want to use the macro you can - but
> i think we can just interface with the holon api directly in
> our core"

> "get them all in - A - just do the work now - we know we'll
> need them - build a arc to do all of them"

Native `Value` variants for OnlineSubspace, Engram, EngramLibrary,
Reckoner — not `#[wat_dispatch]` shims. Direct holon API access
from wat-rs's runtime. Matches arc 052's Vector-as-native-variant
pattern.

---

## Scope

This arc unblocks the lab's Phase 4 files
(`window_sampler.rs`, `scalar_accumulator.rs`, `engram_gate.rs`).
That requires three substrate concerns shipped together:

### 1. Vector-tier algebra primitives

`scalar_accumulator.rs` uses `Primitives::bind`, `bundle`,
`blend_weighted`, `permute` — operating on raw `Vector` inputs
post-arc-052. Today these only work on HolonAST sub-trees
(via `encode`); arc 053 surfaces them as primitives over
`Value::Vector`:

- `(:wat::holon::vector-bind v1 v2) -> :Vector` — XOR-like binding.
- `(:wat::holon::vector-bundle vs) -> :Vector` — superposition over a Vec<Vector>.
- `(:wat::holon::vector-blend v1 v2 w1 w2) -> :Vector` — weighted combination.
- `(:wat::holon::vector-permute v k) -> :Vector` — circular shift by k.

### 2. OnlineSubspace as native wat value

`Value::OnlineSubspace(Arc<ThreadOwnedCell<OnlineSubspace>>)` —
mutable per-thread, zero-Mutex (CSP-safe). Core methods only;
advanced surface deferred to follow-up arcs.

Methods (~10):
- `/new (dim, k) -> :OnlineSubspace` — constructor
- `/dim (s) -> :i64`
- `/k (s) -> :i64`
- `/n (s) -> :i64` — observation count
- `/threshold (s) -> :f64`
- `/eigenvalues (s) -> :Vec<f64>`
- `/update (s, vec) -> :f64` — mutates; returns residual
- `/residual (s, vec) -> :f64`
- `/project (s, vec) -> :Vector`
- `/reconstruct (s, vec) -> :Vector`

Defer: `with_params`, `with_reflexive_params`,
`snapshot`/`from_snapshot`, `anomalous_component`, `update_batch`,
`subspace_alignment`, the entire `StripedSubspace` family.

### 3. Reckoner as native wat value

`Value::Reckoner(Arc<ThreadOwnedCell<Reckoner>>)`. Discrete and
Continuous modes shipped via separate constructors. Supporting
types (Label, Prediction) handled minimally.

Methods (~8):
- `/new-discrete (name, dims, recalib_interval, label_names) -> :Reckoner`
- `/new-continuous (name, dims, recalib_interval, default_value, buckets) -> :Reckoner`
- `/observe (r, vec, label, weight)` — mutates; label is `:i64` (Label::from_index)
- `/predict (r, vec) -> :Prediction` — Prediction is a wat tuple
- `/resolve (r, conviction, correct)` — curve update
- `/curve (r) -> :Option<(f64,f64)>` — (curve_a, curve_b) if valid
- `/labels (r) -> :Vec<i64>`
- `/dims (r) -> :i64`

**Supporting types:**
- `Label` exposed as plain `:i64` (Label::from_index/Label::index).
- `Prediction` exposed as a wat tuple
  `:(Vec<(i64,f64)>, Option<i64>, f64, f64)` —
  (label-cosine pairs, winning direction, conviction, raw_cos).
  No new Value variant; tuple destructuring at the use site.
- `ReckConfig` is encoded in the constructor name (`/new-discrete`
  vs `/new-continuous`); not a wat-tier value.

Defer: `with_curve_params`, `with_max_observations`, `decay`,
`discriminant` accessor, `accuracy_at`, `curve_valid`,
`curve_params`, `resolved_count`, `label_name`, `recalib_count`.

### 4. Engram as native wat value

`Value::Engram(Arc<RefCell<Engram>>)`. Mostly read-only;
`residual` mutates internally for lazy reconstruction.

Methods (~5):
- `/name (e) -> :String`
- `/eigenvalue-signature (e) -> :Vec<f64>`
- `/n (e) -> :i64`
- `/residual (e, vec) -> :f64`

Construction is via `EngramLibrary/add` — Engram doesn't have a
public direct constructor in holon. We expose the read methods
for any Engram retrieved via library `/get`.

Defer: `surprise_profile`, `metadata`, `metadata_mut`,
`subspace`, `save`/`load`.

### 5. EngramLibrary as native wat value

`Value::EngramLibrary(Arc<ThreadOwnedCell<EngramLibrary>>)`.
The collection-and-query type — primary Phase 5 use case.

Methods (~6):
- `/new (dim) -> :EngramLibrary`
- `/add (lib, name, subspace)` — mutates. Simplified: skip
  surprise_profile + metadata args (default empty).
- `/match-vec (lib, probe, top_k, prefilter_k) -> :Vec<(String,f64)>`
- `/len (lib) -> :i64`
- `/contains (lib, name) -> :bool`
- `/names (lib) -> :Vec<String>`

Defer: `add_from_engram`, `match_spectrum`, `match_alignment`,
`remove`, `get`/`get_mut`, `save`/`load`, `dim`.

---

## Architecture decisions

### Mutability — `ThreadOwnedCell` for state-bearing types

OnlineSubspace, Reckoner, EngramLibrary mutate via update/observe/add
methods. The substrate's CSP discipline (zero-Mutex) demands
`ThreadOwnedCell<T>` — wat-rs's existing per-thread-ownership cell
(used by wat-lru and IO readers/writers).

`Engram` uses `RefCell` (single-threaded, lazy-internal-state
caching). Engrams travel cross-thread less commonly; if needed, it
can be migrated to ThreadOwnedCell in a follow-up.

### Vector ↔ &[f64] conversion

OnlineSubspace methods take `&[f64]`. Wat's `:wat::holon::Vector`
carries i8 internally. Conversion: `Vector::to_f64()` on input;
construct `Vector` from `Vec<f64>` on output.

For methods returning `Vec<f64>` (eigenvalues, project,
reconstruct), the wat-tier type is the rich Vector
(`:wat::holon::Vector`) when the output represents a thought
vector (project, reconstruct), or `:Vec<f64>` when it represents
a list of scalars (eigenvalues).

### Path naming

All four types live at `:wat::holon::*` (algebra-tier path), per
arc 052's precedent:
- `:wat::holon::OnlineSubspace`
- `:wat::holon::Engram`
- `:wat::holon::EngramLibrary`
- `:wat::holon::Reckoner`

Methods at `:wat::holon::Type/method` (struct-style auto-method
convention).

### No equality semantics

Learning machines have no meaningful value-equality. Two Reckoners
trained on the same labels produce different internal accumulators
depending on observation order. Two OnlineSubspaces at the same
dim/k differ in their actual basis. Equality across these would be
content-comparison via specific methods (e.g., `subspace_alignment`),
not generic `=`.

`Value::Vector` had bit-exact equality. Phase-4 types reject `=`
for now. `values_equal` returns `None` (TypeMismatch) for these
variants.

### Cross-d behavior

OnlineSubspace.update(&[f64]) requires the slice match the
subspace's dim. Mismatched dim is a runtime error from holon-rs.
We propagate.

---

## Slices

**Slice 1 — Vector algebra primitives.** Add 4 dispatch arms
(`vector-bind`, `vector-bundle`, `vector-blend`,
`vector-permute`) using `Primitives::*`. Schemes registered at
arc-052-style polymorphic-or-strict pattern. Tests.

**Slice 2 — OnlineSubspace.** Value variant, ~10 methods,
schemes, tests.

**Slice 3 — Reckoner + supporting types.** Value variant, ~8
methods, Label as i64, Prediction as tuple, schemes, tests.

**Slice 4 — Engram.** Value variant, ~5 methods, RefCell-based
mutability, schemes, tests.

**Slice 5 — EngramLibrary.** Value variant, ~6 methods, schemes,
tests.

**Slice 6 — INSCRIPTION + USER-GUIDE + lab CHANGELOG.**

---

## What this arc does NOT add

- Advanced methods on each type (deferred per per-type lists above).
- StripedSubspace family.
- Save/load persistence.
- Engram direct construction (only via EngramLibrary/add).
- Atom<Subspace>, Atom<Reckoner>, etc. — atomization of learning
  machines isn't on the table.
- Equality / hashing for the four state-bearing types.
- Lab adoption — that's separate Phase 4 lab arcs after this one.

---

## Sub-fogs

- **Test fixtures.** Tests need actual training data to exercise
  learning. We can construct synthetic vectors via `encode` (arc
  052) + simple `Atom`s, train, observe behavior. No external
  data needed.
- **Prediction tuple shape.** `Vec<(i64,f64)>` for the scores —
  is `(label-index, cosine)` the right ordering? Lean: yes,
  matches holon's `LabelScore { label, cosine }` field order.
- **Label as plain i64.** Loses type-safety (can't distinguish
  `Label` from arbitrary `i64` in wat). Tradeoff: simpler API,
  no new variant. If type-safety becomes a real concern, future
  arc adds `:wat::holon::Label` newtype.
