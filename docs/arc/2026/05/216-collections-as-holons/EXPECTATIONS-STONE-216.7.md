# EXPECTATIONS — Arc 216 Stone 216.7 — Tuple round-trip

Mode A target: 12/12 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | Value::Tuple audit | Internal storage shape documented in SCORE (`Arc<Vec<Value>>` expected; verify); insertion points for atomization arms identified |
| 2 | `is_atomizable` Tuple arm | `src/check.rs` — Tuple<T1, T2, ...> atomizable iff ALL element types atomizable; recursive predicate |
| 3 | `value_to_atom` Tuple arm | `src/runtime.rs` — adjacent to Vec arm; encodes as positional-Bind Bundle (`Bundle([Bind(I64(0), t0), Bind(I64(1), t1), ...])`); mirrors Vec encoding shape |
| 4 | `atom-value` reverse for Tuple | Bundle with positional Binds + consumer-declared `Tuple<T1, T2, ...>` type → `Value::Tuple([t1, t2, ...])`; heterogeneous decode (per-position type) |
| 5 | `HolonRepresentable` impl for Rust tuples | `src/comms/mod.rs` — fixed-arity impls for `(T1, T2)`, `(T1, T2, T3)`, ... up to chosen ceiling (sonnet picks; documents) |
| 6 | Probe 1 — 2-tuple primitives | `(i64, String)` round-trips through Atom + atom-value |
| 7 | Probe 2 — 3-tuple primitives | `(bool, i64, String)` round-trips |
| 8 | Probe 3 — Heterogeneous decode | Bundle with mixed-type positional Binds decodes correctly via consumer-declared `Tuple<T1, T2>` |
| 9 | Probes 4-5 — Nested + Tuple-of-collection | `((i64, i64), String)` AND `(Vec<i64>, String)` round-trip |
| 10 | Probe 6 — Tuple containing HashSet | `(HashSet<i64>, String)` round-trips (composition with 216.1) |
| 11 | Probes 7-10 — Predicate + HolonAST shape + HolonRepresentable cascade + Process-tier IPC | Predicate admits atomizable Tuple + rejects non-atomizable; positional keys are 0..n-1; Rust-level HolonRepresentable round-trip; `pair::<(String, i64)>()` round-trips |
| 12 | SCORE doc inscribed | `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.7.md` — scorecard + arity ceiling + deltas + verification summary + elapsed time |

## Independent prediction (calibration record)

**Target runtime:** 45-75 min Mode A
**Upper bound:** 90 min
**Confidence:** high

**Rationale:**
- Mechanical translation of Stone 216.2's Vec pattern to Tuple variant — same encoding shape
- Substrate is settled (216.5a-d complete; impl Hash for Value canonical)
- Pattern is well-known: positional-Bind Bundle works for both Vec (homogeneous) and Tuple (heterogeneous)
- Risk: HolonRepresentable for Rust tuples might need a macro if arity ceiling is high (STOP-2 trigger)
- Risk: Value::Tuple internal shape could differ from assumed Arc<Vec<Value>> (STOP-1 trigger)

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites 216.2 (Vec template), 216.5a (Hash foundation), 216.5d (substrate impeccable), 216.6 (process-tier cascade). Pattern lineage is dense.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- DESIGN.md inscription — orchestrator-direct (already done)
- holon-rs Symbol docstring — orchestrator-direct (already done; cross-repo)
- Option / Result tagged literals — Stone 216.8
- Instant / Uuid / Duration — Stone 216.9
- INSCRIPTION + closure — Stone 216.10

## Honesty deltas accepted

- HolonRepresentable arity ceiling (2-5 vs 2-12) — sonnet picks based on practical Rust patterns; documents
- Value::Tuple internal storage — sonnet documents actual shape after grep; adjusts encoding accordingly
- Probe count adjustment if a probe overlaps with existing coverage — sonnet picks; documents

## Honesty deltas NOT accepted

- Probe substitution — STOP-3 trigger
- Touching DESIGN.md — orchestrator-direct work; sonnet stays out
- Touching holon-rs — cross-repo; orchestrator-direct
- Extending scope to Option / Result — separate stone
