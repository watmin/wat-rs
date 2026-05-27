# Arc 238 — `:wat::core::=` structural completeness

**Status:** OPEN 2026-05-27. Correctness-defect arc (small diff, foundational impact).
**Pauses:** arc 237 records thread (S-C.2d onward) — records-first-class work does not proceed
on a `=` that errors on records. 237 resumes when `=` is sane.

## The defect (proven)

wat-surface `:wat::core::=` (verb → `eval_eq` → `values_equal` in `src/runtime.rs:9322`)
**errors** on comparable data it should compare. Proven empirically (`tests/probe_diagnostic_eq_on_records.rs`):

```
(= (:my::Pt 1 2) (:my::Pt 1 2))   => Err TypeMismatch (wat::Record)
(= {:a 1 :b 2}  {:a 1 :b 2})      => Err TypeMismatch (wat::core::HashMap)
(= #{1 2 3}     #{1 2 3})         => Err TypeMismatch (wat::core::HashSet)
```

`values_equal` returns `None` for any pair it has no arm for; `eval_eq` maps `None` →
`RuntimeError::TypeMismatch`. So `=` (and its inverse `not=`) raise an error instead of
answering, for the most-used structured types.

## Root cause

Two equality paths exist in the substrate, and the composite types only made it into one:
- **`impl PartialEq for Value`** (`runtime.rs:~888`) — used for HashMap KEYS / the `Hash` contract.
  arc 216 (maps/sets) + arc 234 (records) each gave their type a `PartialEq` arm here.
- **`values_equal`** (`runtime.rs:9322`) — backs the wat `=` verb. The composite types were
  **never added here.** Vec/List/Tuple/Option/Result/Enum/Struct/Vector/HolonAST/scalars are
  present; records/maps/sets/Instant/Duration are not.

No test ever did `(= rec rec)` / `(= map map)` at the wat surface, so it sat latent (test-vantage
gap — the record/map tests checked Rust `PartialEq` and predicates, never the `=` verb).
`feedback_absence_is_signal`: `=` erroring on these was the gap pointing at this arc.

## The audit (FM-2: the COMPLETE missing-data list, not just the 3 first seen)

`values_equal` HANDLES (18): `bool Enum f64 holon__HolonAST i64 Option Result String Struct Tuple
u8 Unit Vec Vector wat__core__Char wat__core__keyword wat__core__List wat__core__Uuid`.

**MUST ADD (comparable data, currently errors):**
| Variant | Arm | Semantics |
|---|---|---|
| `wat__holon__Record` + `wat__Record` | ONE or-patterned arm (both flavors bind `class_fqdn`+`struct_form`) | compare `class_fqdn` (≠ → `Some(false)`); recurse `struct_form` element-wise via `values_equal` — exactly the `Struct` arm shape. **Type-strict** (user's `=`: same type + identical values). Cross-flavor ⟹ cross-class ⟹ `false` (never error — mirrors `Enum`). |
| `wat__std__HashMap` | `(HashMap(a), HashMap(b)) => Some(a == b)` | delegate to `Value`'s `PartialEq` (storage is `Arc<HashMap<Value,Value>>`, arc 216.5c) — order-independent, structural, total. |
| `wat__std__HashSet` | `(HashSet(a), HashSet(b)) => Some(a == b)` | delegate to `PartialEq` (`Arc<HashSet<Value>>`, arc 216.5b) — order-independent. |
| `Instant` | `(Instant(a), Instant(b)) => Some(a == b)` | mirror the `values_compare` Instant arm (`chrono::DateTime<Utc>: Eq`). Closes the orderable-but-not-equatable asymmetry. |
| `Duration` | `(Duration(a), Duration(b)) => Some(a == b)` | i64-nanos; mirror `values_compare`. |
| `wat__WatAST` | `(wat__WatAST(a), wat__WatAST(b)) => Some(a == b)` — **IF** `WatAST: PartialEq` | symmetry with the existing `holon__HolonAST` arm. VERIFY the impl exists; if not, STOP + surface (don't fabricate). |

**CORRECTLY ABSENT (opaque / not value-data — keep erroring):** `wat__core__fn`,
`wat__core__clauses` (callables — pointer identity, not value-equal); `*Sender`/`*Receiver`/
`ChildHandle`/`ProgramHandle`/`HandlePool` (channels/handles); `Engram`/`EngramLibrary`/
`Hologram`/`OnlineSubspace`/`Reckoner` (opaque ML state); `io__IOReader`/`io__IOWriter`;
`RustOpaque`. These have `Arc::ptr_eq` arms in `PartialEq` (for keying) but value-equality of an
opaque handle/fn is meaningless — `=` erroring (or a teaching error) is honest. **Out of arc 238
scope:** whether opaque types should `=`-by-identity is a separate question; this arc fixes
value-DATA only.

## Cross-numeric note (honest delta to document)

The `Vec` arm recurses via `values_equal`, which promotes `i64`↔`f64` (arc 050), so `[1] = [1.0]`
→ true. The new map/set arms delegate to `PartialEq` (no promotion), so `#{1} = #{1.0}` → false.
This minor inconsistency is acceptable: sets/maps are `Hash`-keyed (type-sensitive at storage),
and unordered comparison via `values_equal`-recursion would require O(n²) cross-type matching.
Records' fields are type-locked (a field is `x <- :i64`; no mixed-numeric possible), so the
record arm's `values_equal` recursion has no promotion exposure. Documented; not a blocker.

## Scope boundary — this delivers TYPE-STRICT `=` only

This arc makes `=` mean: **same type + identical concrete values** (the user's locked definition).
It does NOT add `same-data?` (the type-BLIND cross-type record comparison) — that returns to arc
237 (S-C.2d) once `=` is sane. Arc 238 is purely "`=` stops erroring on data and answers."

## Stones

- **238.1** — add the missing `values_equal` arms (records or-patterned + HashMap + HashSet +
  Instant + Duration + WatAST-if-PartialEq). Additive (all before `_ => None`); baseline-preserving
  (existing arms untouched; only previously-erroring comparisons change). FM-2-bis probe =
  `tests/probe_diagnostic_eq_on_records.rs` evolved to assert GREEN (`Ok(true)` for equal, plus
  `Ok(false)` for unequal/cross-type). Sonnet writes; substrate-as-teacher confirms WatAST.
- **238.2** — INSCRIPTION + USER-GUIDE line ("`=` is deep structural over all EDN data incl.
  records/maps/sets") + cross-references (arc 216 maps/sets gap, arc 234 records gap, arc 148
  Instant/Duration asymmetry). Arc closure. Then 237 resumes (S-C.2d).

## Cross-references
- `src/runtime.rs:9322` `values_equal` (the fix site) · `:5651` `eval_eq` (None → TypeMismatch).
- `runtime.rs:~888` `impl PartialEq for Value` (the OTHER equality path — already complete; this
  arc brings `values_equal` to parity for data types).
- arc 216 (HashMap/HashSet storage), arc 234 (records), arc 050/148 (numeric/time arms).
