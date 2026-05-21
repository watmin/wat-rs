# BRIEF — Arc 216 Stone 216.1 — HashSet round-trip

**Stone:** mint bidirectional round-trip for `HashSet<T>` through `HolonAST::Bundle`. Smallest collection case (no Bind shape; bare atoms; cleanest pattern). First stone in the lineage; sets the template for 216.2 (Vector) and 216.3 (HashMap).
**Type:** Sonnet Mode A.
**Time budget:** 45-60 min target; 75 min STOP.
**Depends on:** arc 216 DESIGN (commit `bd1fd2a`); arc 215 (both stones); arc 214 Stone C (HolonRepresentable trait).
**Unblocks:** Stones 216.2 (Vector), 216.3 (HashMap depends on 216.1 + 216.2 for nested cases), 216.4 (consolidated atomizable predicate).

## Goal

Extend `value_to_atom` (runtime.rs:12762) to accept `Value::wat__std__HashSet(s)` → produces `HolonAST::Bundle(vec![T_holon, T_holon, ...])`. Mint reverse direction: `:wat::core::atom-value` extracts `HashSet<T>` from a `HolonAST::Bundle` of bare atoms (consumer declares T via `-> :T` annotation). Add `HolonRepresentable` trait impl for `HashSet<T>`. Add check.rs atomizable-predicate entry.

Per DESIGN Q2 + Q3: Bundle of bare atoms = set-shape. No dedupe at Bundle level (Bundle is the algebraic primitive); dedupe enforced at HashSet construction on reverse trip (HashSet insert is idempotent).

## Pre-flight verified

- `value_to_atom` at `src/runtime.rs:12762` — extension point; three existing dispatch arms (primitives, HolonAST, WatAST) per arc 215 dig
- `Value::wat__std__HashSet(s)` variant pattern exists across runtime.rs (length, contains?, etc. — well-established)
- `HolonRepresentable` trait at `src/comms/mod.rs:90`; `impl HolonRepresentable for String` at line 107 is the pattern template
- `:wat::core::atom-value` exists at runtime.rs (arc 057 extraction primitive); extends to new HolonAST shapes
- Baseline tests green (Stones 4.1-4.3 + arc 215 stones + P1/P2 + arc 169)

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- `cargo test` is the verification path

## Your scope

1. **Extend `value_to_atom` for HashSet** in `src/runtime.rs:12762`:
   - Add a match arm: `Value::wat__std__HashSet(s) => HolonAST::bundle(s.iter().map(value_to_atom).collect::<Result<_, _>>()?)` (or equivalent — sonnet picks the cleanest composition)
   - Each element atomizes recursively via value_to_atom (T must already be atomizable per the recursive predicate)

2. **Extend `:wat::core::atom-value` reverse direction** to extract HashSet:
   - When the consumer's `-> :T` annotation declares `T = :wat::core::HashSet<U>` AND the HolonAST is `Bundle(items)` with all items being bare atoms (not Binds), reconstruct `HashSet<U>` by extracting each atom to U and inserting
   - Dedupe happens naturally via HashSet insert
   - Type mismatch (wrong shape or wrong element type) returns None

3. **Add HolonRepresentable trait impl** in `src/comms/mod.rs`:
   - Mirror `impl HolonRepresentable for String` (line 107) pattern
   - `impl<T> HolonRepresentable for HashSet<T> where T: HolonRepresentable + Hash + Eq + Send + 'static`
   - `to_holon`: collect items as bundle (matches value_to_atom path)
   - `from_holon`: match Bundle shape; extract atoms; insert into HashSet (matches atom-value reverse)

4. **Add check.rs atomizable-predicate entry**:
   - Find where Atom's T-constraint is checked (per arc 215 Q4 verdict — atomizable set is {primitives, HolonAST, WatAST}; arc 216 extends)
   - Add: `T = :wat::core::HashSet<T'>` is atomizable iff `T'` is atomizable
   - Recursive check; the predicate composes per DESIGN Q6

5. **Probe matrix** — `tests/probe_arc216_stone1_hashset_roundtrip.rs` with ~10 probes:
   - Probe 1: Forward — `(value_to_atom #{1 2 3})` → HolonAST::Bundle of three atoms
   - Probe 2: Reverse — `(atom-value <bundle> -> :wat::core::HashSet<wat::core::i64>)` → HashSet<i64>{1,2,3}
   - Probe 3: Empty set round-trip — `#{}` → Bundle([]) → `#{}`
   - Probe 4: Single element — `#{42}` → Bundle of one atom → `#{42}`
   - Probe 5: Multi-T types — works for `HashSet<i64>`, `HashSet<String>`, `HashSet<bool>`, `HashSet<keyword>`
   - Probe 6: Dedupe semantic — reverse trip with duplicate atoms in Bundle still produces a set with unique elements
   - Probe 7: Nested set — `HashSet<HashSet<i64>>` round-trips (requires Stone 216.1 to be recursive — which it is via the predicate)
   - Probe 8: Check passes — `(:wat::holon::Atom my-hashset)` where my-hashset is `HashSet<i64>` type-checks cleanly
   - Probe 9: Check fails — `(:wat::holon::Atom non-atomizable-set)` where T is not atomizable fails at check with diagnostic naming the offending T
   - Probe 10: HolonRepresentable cascade — Rust-side test that `HashSet<String>` implements HolonRepresentable (compile-time check)

6. **WAT-CHEATSHEET update** (`docs/WAT-CHEATSHEET.md`):
   - Brief mention in the atomizable-set section that HashSet<T> is now atomizable (for atomizable T)
   - Reference arc 216 DESIGN

7. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.1.md`:
   - ~15-row scorecard matching EXPECTATIONS
   - Mode declaration (A)
   - Honest deltas section
   - PASS/FAIL per row with citation

## NOT your scope

- Vector round-trip — Stone 216.2
- HashMap round-trip — Stone 216.3
- Consolidated atomizable predicate (if not all done piecemeal here) — Stone 216.4
- Sandbox-scope walker validation — Stone 216.5
- INSCRIPTION — Stone 216.6
- WARD-PASS, INTERSTITIAL — orchestrator post-ship
- Commit + push — orchestrator commits after reviewing SCORE

## STOP triggers

- STOP-1: HolonRepresentable trait surface (mod.rs:90) doesn't compose cleanly for generic T parametric impls — flag if blanket-impl conflicts surface
- STOP-2: `value_to_atom`'s existing error model (TypeMismatch for unhandled variants) doesn't compose cleanly with recursive atomization — flag if needed
- STOP-3: any existing test fails — surface
- STOP-4: 75 min elapsed

## Verification

Single commands per line (firewall-friendly):

```
cargo build --release
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

Report:
- Final PASS count out of 15
- Honest deltas
- Verification summary
- Elapsed time
- Anything discovered

Don't commit. Orchestrator commits after reviewing SCORE.
