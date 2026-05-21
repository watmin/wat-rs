# EXPECTATIONS — Arc 216 Stone 216.2 — Vector round-trip

Mode A target: 17/17 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `value_to_atom` extended for Vector | New match arm adjacent to HashSet (Stone 216.1); produces `Bundle` of positional-Binds with i64 keys 0..n-1 |
| 2 | `atom-value` reverse for Vector | Extracts `Vec<T>` from `Bundle` of positional-Binds when consumer declares `T = :wat::core::Vector<U>` |
| 3 | HolonRepresentable impl for Vec<T> | `src/comms/mod.rs` — mirrors HashSet pattern from 216.1; bounds `T: HolonRepresentable + Send + 'static` |
| 4 | `is_atomizable` extended for Vector | `Vector<T>` atomizable iff `T` atomizable; mirrors HashSet entry |
| 5 | Probe 1 — Forward | `(value_to_atom [1 2 3])` → Bundle of three Binds with sequential i64 keys (0, 1, 2) |
| 6 | Probe 2 — Reverse | `(atom-value <bundle> -> :wat::core::Vector<wat::core::i64>)` → `Vec<i64>[1,2,3]` |
| 7 | Probe 3 — Empty vec round-trip | `[]` → Bundle([]) → `[]`; length 0 preserved |
| 8 | Probe 4 — Single element | `[42]` round-trips |
| 9 | Probe 5 — Multi-T types | Works for `Vec<i64>`, `Vec<String>`, `Vec<bool>`, `Vec<keyword>` |
| 10 | Probe 6 — Order preservation | Round-trip preserves element order via i64 key sequence |
| 11 | Probe 7 — Nested vector | `Vec<Vec<i64>>` round-trips |
| 12 | Probe 8 — Mixed nesting | `Vec<HashSet<i64>>` round-trips (composes with Stone 216.1) |
| 13 | Probe 9 — Check passes | `(:wat::holon::Atom my-vec)` for atomizable T type-checks |
| 14 | Probe 10 — Check fails | Non-atomizable T fails at check; diagnostic |
| 15 | Probe 11 — HolonRepresentable cascade | Compile-time `Vec<String>: HolonRepresentable` |
| 16 | Probe 12 — Reverse-shape validation | Bundle with non-sequential keys (e.g., [Bind(0,a), Bind(2,b)]) → None on extract |
| 17 | WAT-CHEATSHEET updated | Atomizable-set section extended for `Vector<T>` |

## Independent prediction (calibration record)

**Target runtime:** 60-75 min Mode A
**Upper bound:** 90 min
**Confidence:** medium-high

**Rationale:**
- Stone 216.1 shipped in ~29 min; 216.2 is comparable shape with Bind complexity added
- Templates established: value_to_atom dispatch arm; atom-value reverse path; HolonRepresentable impl; is_atomizable predicate
- New work: Bind construction in forward (key = Atom(I64(i))); positional-invariant validation in reverse (keys must be 0..n-1)
- Risk: shape-dispatch between bare-atom (HashSet) and Bind (Vector) paths in atom-value — sonnet picks unified-with-shape-detection vs separate-paths; either is honest
- Probes mirror 216.1 + 2 new (order preservation, reverse-shape validation)

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- HashMap round-trip — Stone 216.3
- Sandbox-walker validation — Stone 216.5
- Consolidated predicate refactor — Stone 216.4
- INSCRIPTION — Stone 216.6

## Honesty deltas accepted

- atom-value shape-dispatch choice (unified vs separate paths) — sonnet picks; documents
- HolonRepresentable bounds — if `T: Send + 'static` proves too tight for `Vec<T>`, document the adjustment
- Positional-invariant edge cases (empty, single, large) — handled; documented
