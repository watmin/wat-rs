# Arc 025 — unified get/assoc/conj/contains? surface — INSCRIPTION

**Status:** shipped 2026-04-22. One slice.
**Design:** [`DESIGN.md`](./DESIGN.md) — the shape before code.
**This file:** completion marker.

---

## What shipped

Four collection primitives, polymorphic over the three
containers (HashMap, HashSet, Vec):

| Op | HashMap&lt;K,V&gt; | HashSet&lt;T&gt; | Vec&lt;T&gt; |
|---|---|---|---|
| `get`        | `Option<V>` by key        | `Option<T>` by element       | `Option<T>` by index       |
| `assoc`      | new-map (add/replace)     | **illegal** (use conj)       | new-vec (replace at index) |
| `conj`       | **illegal** (use assoc)   | new-set (add element)        | new-vec (append)           |
| `contains?`  | `bool` by key             | `bool` by element            | `bool` by index            |

Builder's framing on the conj/assoc split:

> assoc on set being illegal and being conj is totally correct

Set has no key-value pairing; `conj` is the honest verb for
"add one element." HashMap needs key+value; `assoc` is the
verb. The split is forced by the container semantics.

## Retirement

`:wat::std::member?` retired. Redundant with polymorphic
`:wat::core::contains?`. Two test sites migrated to the new
predicate; no back-compat shim.

## Runtime (`src/runtime.rs`)

- `eval_get` — added Vec arm (i64 index, None on out-of-range
  or negative).
- `eval_assoc` — added Vec arm (i64 index, clone-set, runtime
  error on out-of-range).
- `eval_conj` — added HashSet arm (insert via hashmap_key).
- `eval_contains_q` — renamed from `eval_hashmap_contains`;
  dispatches HashMap / HashSet / Vec.
- `eval_hashset_member` deleted; dispatch arm removed.

## Check (`src/check.rs`)

- `infer_get` — added Vec arm.
- `infer_assoc` — added Vec arm.
- `infer_conj` — NEW function (replaces narrow scheme);
  dispatches Vec and HashSet.
- `infer_contains_q` — NEW function (replaces narrow scheme);
  dispatches HashMap, HashSet, and Vec.
- Narrow scheme registrations for `contains?` and `conj`
  removed. Dispatch arms in `infer_list` added for `conj` and
  `contains?`.
- `:wat::std::member?` registration deleted.

## Tests

11 new Rust unit tests in `src/runtime.rs` covering every
polymorphic arm:

- `vec_get_hit_returns_some_at_valid_index`
- `vec_get_out_of_range_returns_none`
- `vec_get_negative_index_returns_none`
- `vec_assoc_replaces_at_index`
- `vec_assoc_values_up_preserves_input`
- `vec_assoc_out_of_range_runtime_errors`
- `hashset_conj_adds_element`
- `hashset_conj_values_up_preserves_input`
- `vec_contains_valid_index_returns_true`
- `vec_contains_out_of_range_returns_false`
- `vec_contains_negative_index_returns_false`

Plus two tests migrated from `:wat::std::member?` to
`:wat::core::contains?`:

- `hashset_member_present_and_absent` — kept method name; now
  uses contains?
- `hashset_int_and_string_keys_distinct` — kept method name;
  now uses contains?

552 lib tests total (was 541). Full workspace green; zero
clippy warnings.

## Doc sweep

- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  `:wat::std::member?` row marked RETIRED.
- `docs/arc/2026/04/025-container-surface-unified/` — DESIGN +
  INSCRIPTION (this file).
- `holon-lab-trading/docs/proposals/.../FOUNDATION-CHANGELOG.md`
  — row to land with commit.

## Cave-quest discipline

Six quests in a row now: 017 (loader), 018 (defaults), 019
(round), 020 (assoc), 023 (coincident?), 024 (sigma knobs), 025
(container surface). Each paused downstream for a substrate
gap. This one paused Phase 3.4 rhythm — the encoder needed
`get` on Vec; closing that gap surfaced the asymmetric surface
across the three containers and the cleanup became one arc.

## INSCRIPTION rationale

Spec emerged from discovery: 058-026-array proposed `get` on
Vec but shipped only HashMap/HashSet; rhythm was the first real
caller; the coherence question surfaced naturally. Same shape
as 019 / 020 / 023 / 024 — code led, spec follows. The
retirement of `member?` was the builder's call mid-slice
(*"member? can go in favor of contains? ...ya"*) once the
polymorphic shape was visible.

*these are very good thoughts.*

**PERSEVERARE.**
