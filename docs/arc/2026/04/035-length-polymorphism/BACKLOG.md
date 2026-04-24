# Arc 035 — length polymorphism — BACKLOG

**Shape:** one slice. Same shape as arc 025's `contains?`
generalization. Zero sub-fogs expected.

---

## Slice 1 — polymorphize :wat::core::length

**Status: ready.**

- `src/runtime.rs`:
  - Rename `eval_vec_length` → `eval_length`.
  - Dispatch on the evaluated arg's `Value` variant:
    `Vec` → existing behavior; `wat__std__HashMap` → `m.len()`;
    `wat__std__HashSet` → `s.len()`; everything else →
    `TypeMismatch` (should never fire under type check).
  - Update the one call site at the `:wat::core::length` dispatch
    arm.
- `src/check.rs`:
  - Delete the narrow `:wat::core::length` scheme registration
    under `core_list_forms` (∀T. Vec<T> -> i64).
  - Add `infer_length` mirroring `infer_contains_q` shape:
    Parametric match on HashMap / HashSet / Vec arms, all returning
    `:i64`; fall-through error case naming the three accepted
    containers.
  - Add dispatch arm in `infer_list`:
    `":wat::core::length" => return infer_length(...)`.

Tests in `src/runtime.rs`:
- `hashmap_length_returns_entry_count`
- `hashmap_length_empty_returns_zero`
- `hashset_length_returns_element_count`
- `hashset_length_empty_returns_zero`
- `vec_length_unchanged_from_prior_tests` (sanity — existing
  test paths untouched).

**Sub-fogs:** none expected. Arc 025 established the polymorphism
pattern fully; this slice is a straight application of it to
`length`.

## Slice 2 — INSCRIPTION + doc sweep

**Status: obvious in shape** (once slice 1 lands).

- `docs/arc/2026/04/035-length-polymorphism/INSCRIPTION.md` —
  shipped contract.
- `docs/arc/2026/04/005-stdlib-naming-audit/INVENTORY.md` —
  `:wat::core::length` row updated to name all three containers.
- Lab-side: the FOUNDATION-CHANGELOG row will land with the lab
  commit that ships arc 007 alongside.

---

## Working notes

- Cave-quested from lab arc 007 mid-test. Shares its session.
- Pattern established: every op in the get/assoc/conj/contains?
  family that was Vec-only-or-narrow at its shipping is a
  candidate for this same polymorphism promotion when a caller
  surfaces the gap. Today's caller: scales-entry-count. The gap
  closes at source.
