# BRIEF — Arc 216 Stone 216.7 — Tuple round-trip + (orchestrator: doctrine inscription + Symbol docstring)

**Stone scope (sonnet portion):** mint Tuple round-trip through the substrate. Add `is_atomizable` arm, `value_to_atom` arm, `atom-value` reverse path, `HolonRepresentable` impl (or arms) for `Value::Tuple` / Rust tuples. Tuple is collection-category per the encoding doctrine: positional-Bind Bundle, same shape as Vec from 216.2. Mechanical translation.
**Type:** Sonnet Mode A.
**Time budget:** 45-75 min target; 90 min STOP.
**Depends on:** Stone 216.2 (`e4a63ed` — Vec positional-Bind Bundle pattern; the template), Stone 216.5d (`ef7e0c6` — substrate impeccable; `impl Hash for Value` canonical), Stone 216.6 (`9fb058d` — process-tier cascade validation).
**Unblocks:** Stone 216.8 (sum-type tagged literals), Stone 216.9 (EDN-tagged scalars), Stone 216.10 (INSCRIPTION + arc closure).

## Orchestrator already did

Before this BRIEF was drafted:

- **DESIGN.md forward-correction** — appended `## Encoding doctrine (Stone 216.7 onward) — 2026-05-21` section with the 3-category table + locked tagged shapes + stone decomposition. Sonnet reads this for context; doesn't modify.
- **holon-rs `HolonAST::Symbol` docstring touch-up** — at `/home/watmin/work/holon/holon-rs/src/kernel/holon_ast.rs:52-71` — acknowledges dual-use (keyword + bare symbol + nil literal). Sonnet doesn't touch this either.

Both are orchestrator-direct work per `feedback_sonnet_no_realization_voice` and FM 7 (cross-repo cwd safety).

## Why this stone exists (lineage)

Read DESIGN.md "Encoding doctrine (Stone 216.7 onward)" section. The short version: arc 216 was about to close on collections-as-holons but the user surfaced — *what about Tuple? Option? Result? Instant? Uuid? Duration?* The variant audit from 216.5a had named these as "structurally-equal but NOT atomizable." Arc 216 expanded to close them per `feedback_no_known_defect_left_unfixed`. Tuple is the first piece — pure collection-category extension; same shape as Vec.

## Pattern template

Stone 216.2's Vec round-trip is the canonical template. Tuple's shape is IDENTICAL — positional-Bind Bundle with i64 keys — only the consumer-declared type differs:

- Vec<T>: variable arity; homogeneous element type T; decoded by inferring T from each Bind's value
- Tuple<T1, T2, ...>: fixed arity; heterogeneous element types; decoded by walking the type signature

The substrate encoding is the SAME shape. Sonnet's work is wiring the existing pattern to the Tuple variant.

## Pre-flight verified

- Stone 216.2 SHIPPED — Vec round-trip pattern at `src/runtime.rs` (`value_to_atom` Vec arm, `atom-value` reverse, HolonRepresentable for `Vec<T>` at `src/comms/mod.rs:190`)
- Stone 216.5a SHIPPED — `impl Hash for Value` at `src/runtime.rs:620+`; `Value::Tuple` classified as "structurally-not-atomizable" (gets structural Hash from 216.5a; NOT in is_atomizable)
- Stone 216.5d SHIPPED — `hashmap_key` gone; substrate uses `Value: Hash + Eq` natively
- Stone 216.6 SHIPPED — process-tier cascade validated end-to-end

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope (sonnet)

### Part A — Audit the Tuple shape

1. **Find Value::Tuple variant:**
   ```
   grep -n Value::Tuple src/runtime.rs
   ```
   Document the internal storage shape (probably `Arc<Vec<Value>>` or similar; verify).

2. **Find is_atomizable predicate (Tuple currently absent):**
   ```
   grep -n is_atomizable src/check.rs
   ```
   Identify where to add the Tuple arm.

3. **Find Vec round-trip template (216.2):**
   ```
   grep -n Value::Vec src/runtime.rs
   ```
   Locate the `value_to_atom` Vec arm — that's your translation source.

### Part B — Add Tuple atomization

4. **`is_atomizable` Tuple arm** at `src/check.rs` — `Tuple<T1, T2, ...>` is atomizable iff ALL element types are atomizable. Recursive predicate.

5. **`value_to_atom` Tuple arm** at `src/runtime.rs` — adjacent to Vec arm. Encode as positional-Bind Bundle:
   ```rust
   Value::Tuple(elements) => {
       HolonAST::bundle(
           elements.iter().enumerate()
               .map(|(i, elem)| Ok(HolonAST::bind(
                   HolonAST::i64(i as i64),
                   value_to_atom(elem)?
               )))
               .collect::<Result<_, _>>()?
       )
   }
   ```
   (sonnet's actual shape will depend on Vec arm structure; mirror it)

6. **`atom-value` reverse for Tuple** — given a consumer-declared type `Tuple<T1, T2, T3>` and a Bundle with 3 positional Binds, reconstruct `Value::Tuple([t1, t2, t3])`. Heterogeneous decode: per-position, decode each element to its declared type.

7. **`HolonRepresentable` impl for Rust tuples** at `src/comms/mod.rs`:
   - Add impls for fixed-arity tuples (start with 2-tuple, 3-tuple; can extend if needed)
   - `impl<T1, T2> HolonRepresentable for (T1, T2) where T1: HolonRepresentable, T2: HolonRepresentable`
   - to_holon_ast: collect as Bundle of positional Binds
   - from_holon_ast: validate Bundle shape; decode each Bind by position
   - Sonnet picks the arity ceiling (likely 2-5 covers practical cases; document)

### Part C — Probes

8. **Probe suite** at `tests/probe_arc216_stone7_tuple_roundtrip.rs` (~10 probes):
   - Probe 1: 2-tuple primitives — `(i64, String)` round-trips
   - Probe 2: 3-tuple primitives — `(bool, i64, String)` round-trips
   - Probe 3: Heterogeneous decode — Bundle with `[Bind(0, I64), Bind(1, String)]` → `(i64, String)` via consumer-declared type
   - Probe 4: Nested tuple — `((i64, i64), String)` round-trips
   - Probe 5: Tuple containing Vec — `(Vec<i64>, String)` round-trips
   - Probe 6: Tuple containing HashSet — `(HashSet<i64>, String)` round-trips
   - Probe 7: is_atomizable predicate — `Tuple<i64, String>` admits; `Tuple<i64, Fn>` rejects
   - Probe 8: HolonAST shape verification — positional Bind keys are 0..n-1
   - Probe 9: HolonRepresentable cascade — Rust `(String, i64)` compile-time check + round-trip
   - Probe 10: Process-tier IPC — `pair::<(String, i64)>()` send + recv round-trips (depends on Probe 9 working)

### Part D — Verification + SCORE

9. **Run prior probe suites — verify no regressions:**
   ```
   cargo test --release --test probe_arc216_stone6_process_collection_roundtrip -p wat
   cargo test --release --test probe_arc216_stone5c_hashmap_native_storage -p wat
   cargo test --release --test probe_arc216_stone5b_hashset_native_storage -p wat
   cargo test --release --test probe_arc216_stone2_vector_roundtrip -p wat
   ```

10. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.7.md` — scorecard matching EXPECTATIONS row count; document HolonRepresentable arity ceiling chosen; deltas; verification summary; elapsed time.

## NOT your scope

- DESIGN.md inscription — orchestrator already did
- holon-rs Symbol docstring — orchestrator already did (cross-repo cwd risk)
- Option / Result tagged literals — Stone 216.8
- Instant / Uuid / Duration — Stone 216.9
- INSCRIPTION + closure — Stone 216.10
- Refactoring Vec / HashSet / HashMap code (settled foundation)

## STOP triggers

- **STOP-1: Value::Tuple shape unexpected** — if internal storage is something other than `Arc<Vec<Value>>` (e.g., heterogeneous container), surface for orchestrator before extending
- **STOP-2: HolonRepresentable for Rust tuples requires macro** — if fixed-arity impls become unwieldy (8+ arities), surface; might need a macro helper as substrate addition
- **STOP-3: probe substitution** — fix the impl or surface; do not substitute test subjects
- **STOP-4: any existing probe regresses** — surface
- **STOP-5: 90 min elapsed**

## Verification

One per line:

```
cargo build --release
cargo test --release --test probe_arc216_stone7_tuple_roundtrip -p wat
cargo test --release --test probe_arc216_stone6_process_collection_roundtrip -p wat
cargo test --release --test probe_arc216_stone5c_hashmap_native_storage -p wat
cargo test --release --test probe_arc216_stone5b_hashset_native_storage -p wat
cargo test --release --test probe_arc216_stone5a_value_hash -p wat
cargo test --release --test probe_verify_hashset_of_vector_gap -p wat
cargo test --release --test probe_arc216_stone4_predicate_composition -p wat
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip -p wat
cargo test --release --test probe_arc216_stone2_vector_roundtrip -p wat
cargo test --release --test probe_arc216_stone1_hashset_roundtrip -p wat
cargo clippy --release -- -D warnings
```

## When you finish

Report: pass count out of EXPECTATIONS row count, deltas, verification summary, elapsed time, HolonRepresentable arity ceiling chosen, anything surfaced via STOPs.

Don't commit. Orchestrator commits after review.
