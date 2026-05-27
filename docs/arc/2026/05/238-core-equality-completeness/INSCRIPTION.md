# Arc 238 — INSCRIPTION — `:wat::core::=` structural completeness

**Closed 2026-05-27.** `:wat::core::=` (and `not=`) now answer structurally for every comparable
data type; the defect that made them ERROR on records/maps/sets — and silently lack
Instant/Duration — is gone.

## What shipped (Stone 238.1, commit `290a6cb3`)

Six arms added to `values_equal` (`src/runtime.rs`), all before the `_ => None` fallthrough —
purely additive (+99 lines, one file; no existing arm touched, so existing behavior is unchanged):

| Type | Semantics |
|---|---|
| **records** (`wat__holon__Record` + `wat__Record`, one or-patterned arm) | type-strict: compare `class_fqdn`, then recurse `struct_form` element-wise via `values_equal` (mirrors the `Struct` arm). Cross-flavor ⟹ different class ⟹ `Some(false)`; never errors. |
| **`HashMap`** | `Some(a == b)` — order-independent, structural (`Value: PartialEq`, arc 216.5c). |
| **`HashSet`** | `Some(a == b)` — order-independent (216.5b). |
| **`Instant`** | `Some(a == b)` — closes the orderable-but-not-equatable asymmetry (`values_compare` had it; `values_equal` did not). |
| **`Duration`** | `Some(a == b)`. |
| **`WatAST`** | `Some(a == b)` — symmetry with the existing `holon__HolonAST` arm; `WatAST` derives `PartialEq` (`src/ast.rs:33`). |

Plus 6 co-located `#[cfg(test)]` unit tests (Instant/Duration/WatAST equal + unequal) — the
wat-surface probe can't easily construct those.

## Proof

`tests/probe_arc238_eq_completeness.rs` — 8 contracts (records/maps/sets: equal, unequal,
order-independent). RED before (every contract errored with `TypeMismatch`); **8/8 green** after.
Lib baseline **828 → 834** (the 6 new unit tests). Record + defrecord regressions green. FM-9
independently re-verified before commit.

## The doctrine this records (data-vs-opaque)

`=` is **deep structural equality over all EDN/value data**: scalars, `Vec`/`List`/`Tuple`,
`Option`/`Result`/`Enum`, `Struct`, `Vector`, `HolonAST`/`WatAST`, **records**, **`HashMap`**,
**`HashSet`**, `Instant`, `Duration`. It deliberately does NOT compare **opaque** values —
functions/`clauses`, channels/handles (`Sender`/`Receiver`/`ChildHandle`/`ProgramHandle`/
`HandlePool`), opaque ML state (`Engram`/`EngramLibrary`/`Hologram`/`OnlineSubspace`/`Reckoner`),
io readers/writers, `RustOpaque`. Value-equality of a handle or a function is meaningless; `=`
on those remains a `TypeMismatch` (honest refusal, not a silent wrong answer). Whether opaque
types should compare by identity is a separate question, intentionally NOT in this arc's scope.

## Root cause (for the record)

Two equality paths existed: `impl PartialEq for Value` (HashMap keys / the `Hash` contract) and
`values_equal` (the `=` verb). Arcs 216 (maps/sets) and 234 (records) added their types to the
FIRST but never the second. No test ever exercised `(= rec rec)` / `(= map map)` at the wat
surface, so it stayed latent. Surfaced 2026-05-27 by the arc-237 S-C.2d "same-data?" exploration.

## Affirmatively out of arc 238's scope

- **`same-data?`** (type-BLIND cross-type record data comparison) — NOT part of `=` (which is
  type-strict). It is the user's distinct second tool. Tracked in **arc 237 S-C.2d**
  (`docs/arc/2026/05/237-polymorphism-consolidation/REMAINING-ORDER.md`), which resumes now.
- **Opaque-type identity equality** — out of scope (architectural reason above); not tracked
  elsewhere (no demand surfaced).

## Cross-references

`src/runtime.rs:9322` `values_equal` · arc 216 (HashMap/HashSet storage) · arc 234 (records) ·
arc 050/148 (numeric/time arms) · `tests/probe_arc238_eq_completeness.rs`.
