# EXPECTATIONS — Arc 216 Stone 216.1 — HashSet round-trip

Mode A target: 15/15 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `value_to_atom` extended for HashSet | `src/runtime.rs:12762` — new match arm for `Value::wat__std__HashSet(s)`; produces `HolonAST::Bundle(items)` where each item is `value_to_atom` of the element |
| 2 | `:wat::core::atom-value` reverse for HashSet | Extracts `HashSet<T>` from `Bundle` of bare atoms when consumer declares `T = :wat::core::HashSet<U>` via `-> :T` annotation |
| 3 | `HolonRepresentable` impl for HashSet | `src/comms/mod.rs` — `impl<T> HolonRepresentable for HashSet<T> where T: HolonRepresentable + Hash + Eq + Send + 'static`; mirrors String impl pattern at line 107 |
| 4 | check.rs atomizable predicate extended | `HashSet<T>` is atomizable iff `T` is atomizable (recursive; composes per DESIGN Q6) |
| 5 | Probe 1 — Forward | `(value_to_atom #{1 2 3})` produces `HolonAST::Bundle` containing three atoms |
| 6 | Probe 2 — Reverse | `(atom-value <bundle> -> :wat::core::HashSet<wat::core::i64>)` produces `HashSet<i64>{1,2,3}` |
| 7 | Probe 3 — Empty set round-trip | `#{}` → Bundle([]) → `#{}`; length 0 preserved |
| 8 | Probe 4 — Single element | `#{42}` round-trips to `#{42}` |
| 9 | Probe 5 — Multi-T types | Works for `HashSet<i64>`, `HashSet<String>`, `HashSet<bool>`, `HashSet<keyword>` |
| 10 | Probe 6 — Dedupe semantic | Reverse trip with duplicate atoms in Bundle still produces a set with unique elements |
| 11 | Probe 7 — Nested set | `HashSet<HashSet<i64>>` round-trips correctly (recursive atomization works) |
| 12 | Probe 8 — Check passes | `(:wat::holon::Atom my-hashset)` type-checks for atomizable T |
| 13 | Probe 9 — Check fails | Non-atomizable T fails at check; diagnostic names the offending position |
| 14 | Probe 10 — HolonRepresentable cascade | Compile-time check: `HashSet<String>` satisfies `HolonRepresentable` bound |
| 15 | WAT-CHEATSHEET updated | Brief mention HashSet<T> is atomizable; reference arc 216 DESIGN |

## Independent prediction (calibration record)

**Target runtime:** 45-60 min Mode A
**Upper bound:** 75 min
**Confidence:** medium-high

**Rationale:**
- Stone 4.1 (typealias) shipped in ~7 min; 4.2 (verb trio) in ~20 min; 4.3 (dig trio) in ~15 min
- This stone is comparable to 4.2 — substrate extensions in runtime.rs + check.rs + new file; ~10 probes
- Pattern templates direct (value_to_atom dispatch arms; String HolonRepresentable impl)
- Risk: recursive atomization in value_to_atom may surface error-handling subtleties; the `Result<_, _>::collect()` pattern needs honest propagation
- Risk: HolonRepresentable blanket-impl interaction (mod.rs:81 mentions a future blanket impl path) — check for conflicts

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Vector / HashMap round-trip — Stones 216.2 / 216.3
- Sandbox-walker validation — Stone 216.5
- Consolidated atomizable predicate refactor — Stone 216.4
- INSCRIPTION — Stone 216.6

## Honesty deltas accepted

- HolonRepresentable trait surface may need slight adjustment if generic T bounds don't compose cleanly (Send + 'static constraints in particular); document the choice
- check.rs predicate placement (in atomizable check function vs piecemeal in each constructor) — sonnet picks; documented
- Error model for failed atomization mid-stream (e.g., partial Bundle construction) — sonnet picks; documented
- Probe 7 (nested) may need recursive HolonRepresentable bound to work; if substrate cleanly handles this via the predicate, no issue; if not, flag
