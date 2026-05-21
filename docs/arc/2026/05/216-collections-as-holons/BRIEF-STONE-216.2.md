# BRIEF — Arc 216 Stone 216.2 — Vector round-trip

**Stone:** mint bidirectional round-trip for `Vector<T>` (i.e., `Vec<T>` at the Rust layer) through `HolonAST::Bundle` of positional-Binds. Second collection case; introduces the Bind shape; sets the template for 216.3 (HashMap; same Bind shape with arbitrary K).
**Type:** Sonnet Mode A.
**Time budget:** 60-75 min target; 90 min STOP.
**Depends on:** arc 216 Stone 216.1 (commit `b478ff4`) — `value_to_atom` extension pattern, `atom-value` reverse pattern, `HolonRepresentable` trait impl pattern, `is_atomizable` predicate machinery all established.
**Unblocks:** Stone 216.3 (HashMap round-trip — combines positional-binds insight with arbitrary K).

## Goal

Extend `value_to_atom` to accept `Value::wat__core__Vector(v)` → produces `HolonAST::Bundle(vec![Bind(Atom(i64(0)), T_holon_0), Bind(Atom(i64(1)), T_holon_1), ...])`. Mint reverse: `:wat::core::atom-value` extracts `Vec<T>` from a `Bundle` of positional-Binds (consumer declares T via `-> :T`). Add `HolonRepresentable` impl for `Vec<T>`. Add `is_atomizable` entry for `Vector<T>`.

Per DESIGN Q2: Vector = "Bundle of Binds with integer keys" — positional encoding via 0..n-1 indices in the Bind keys.

## Pre-flight verified

- Stone 216.1 shipped (`b478ff4`): pattern templates fully established
  - `value_to_atom` HashSet arm at `src/runtime.rs` (per SCORE-STONE-216.1)
  - `atom-value` Bundle→HashSet path (per SCORE-STONE-216.1)
  - `impl HolonRepresentable for HashSet<T>` at `src/comms/mod.rs` (per SCORE-STONE-216.1)
  - `is_atomizable` predicate with Atom special-case at `src/check.rs` (per SCORE-STONE-216.1)
- Read Stone 216.1's SCORE for exact line numbers + the helper-function shapes — your work mirrors that pattern with Bind-shape replacing bare-atom-shape
- Baseline tests green (all 8 probe suites + 824 lib unit tests)

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

1. **Extend `value_to_atom` for Vector** in `src/runtime.rs`:
   - Add a match arm next to the HashSet arm (Stone 216.1): `Value::wat__core__Vector(v) => HolonAST::bundle(v.iter().enumerate().map(|(i, item)| Ok(HolonAST::bind(HolonAST::Atom(Arc::new(HolonAST::I64(i as i64))), value_to_atom(item)?))).collect::<Result<_, _>>()?)` (or sonnet's cleaner composition)
   - Each element atomizes recursively via value_to_atom
   - Each (index, element) pair becomes a Bind with i64 key

2. **Extend `:wat::core::atom-value` reverse direction** for Vector:
   - When consumer's `-> :T` annotation declares `T = :wat::core::Vector<U>` AND the HolonAST is `Bundle(items)` with all items being `Bind(Atom(I64(i)), v)` AND the i64 keys are sequential `0..items.len()-1`, reconstruct `Vec<U>` by extracting each Bind's value to U (in key order) and collecting
   - Validate the positional-bind invariant: keys must be 0..n-1; out-of-order or missing keys → None (wrong shape)
   - Type mismatch (wrong shape or wrong element type) → None
   - Note: HashSet's reverse expected bare atoms; Vector's reverse expects Binds with i64-Atom keys. Sonnet decides whether one unified atom-value handles both via shape-dispatch or whether separate paths are cleaner.

3. **Add HolonRepresentable trait impl** in `src/comms/mod.rs`:
   - Mirror Stone 216.1's HashSet impl pattern
   - `impl<T> HolonRepresentable for Vec<T> where T: HolonRepresentable + Send + 'static`
   - `to_holon`: collect items as Bundle of positional-binds
   - `from_holon`: match Bundle; verify positional invariant; extract atoms in order; collect into Vec

4. **Extend `is_atomizable` predicate** in `src/check.rs`:
   - Add: `T = :wat::core::Vector<T'>` is atomizable iff `T'` is atomizable
   - Mirrors HashSet's entry from Stone 216.1

5. **Probe matrix** — `tests/probe_arc216_stone2_vector_roundtrip.rs` with ~12 probes:
   - Probe 1: Forward — `(value_to_atom [1 2 3])` → Bundle of three Binds with sequential i64 keys
   - Probe 2: Reverse — `(atom-value <bundle> -> :wat::core::Vector<wat::core::i64>)` → `Vec<i64>[1,2,3]`
   - Probe 3: Empty vec round-trip — `[]` → Bundle([]) → `[]`
   - Probe 4: Single element — `[42]` → Bundle of one Bind → `[42]`
   - Probe 5: Multi-T types — works for `Vec<i64>`, `Vec<String>`, `Vec<bool>`, `Vec<keyword>`
   - Probe 6: Order preservation — round-trip preserves element order (i64 key sequence enforced)
   - Probe 7: Nested vector — `Vec<Vec<i64>>` round-trips
   - Probe 8: Mixed nesting — `Vec<HashSet<i64>>` round-trips (composes with Stone 216.1)
   - Probe 9: Check passes — `(:wat::holon::Atom my-vec)` for atomizable T type-checks
   - Probe 10: Check fails — `(:wat::holon::Atom non-atomizable-vec)` fails at check
   - Probe 11: HolonRepresentable cascade — Rust compile-time check `Vec<String>: HolonRepresentable`
   - Probe 12: Reverse-shape validation — Bundle with non-sequential i64 keys (e.g., [Bind(0,a), Bind(2,b)]) → None on extract (positional invariant violated)

6. **WAT-CHEATSHEET update** — extend atomizable-set section to include `Vector<T>` (in addition to Stone 216.1's HashSet entry)

7. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.2.md` — 17-row scorecard

## NOT your scope

- HashMap round-trip — Stone 216.3
- Consolidated atomizable predicate refactor (if not done piecemeal) — Stone 216.4
- Sandbox-scope walker validation — Stone 216.5
- INSCRIPTION — Stone 216.6
- WARD-PASS, INTERSTITIAL — orchestrator post-ship

## STOP triggers

- STOP-1: Bind round-trip surfaces type-coercion subtleties (Atom keys vs raw i64 values) — flag if needed
- STOP-2: positional-bind invariant validation has edge cases (e.g., what if a Vector has more than i64::MAX elements? practically irrelevant but documented)
- STOP-3: any existing test fails — surface
- STOP-4: 90 min elapsed

## Verification

Single commands per line:

```
cargo build --release
cargo test --release --test probe_arc216_stone2_vector_roundtrip -p wat
cargo test --release --test probe_arc216_stone1_hashset_roundtrip -p wat
cargo test --release --test probe_arc214_slice4_stone3_env_dig_trio -p wat
cargo test --release --test probe_arc214_slice4_stone2_env_get_trio -p wat
cargo test --release --test probe_arc214_slice4_stone1_program_env_typealias -p wat
cargo test --release --test probe_arc215_stone2 -p wat
cargo test --release --test probe_arc215_collection_literal_inference -p wat
cargo test --release --test probe_brace_map_literal -p wat
cargo test --release --test probe_hashmap_ctor_vector_symmetric -p wat
cargo clippy --release -- -D warnings
```

## When you finish

Report: pass count out of 17, deltas, verification summary, elapsed time, anything discovered.

Don't commit. Orchestrator commits after review.
