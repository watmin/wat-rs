# BRIEF — Arc 216 Stone 216.4 — Atomizable predicate consolidation + verification

**Stone:** verify the recursive `is_atomizable` predicate (introduced piecemeal during 216.1, extended bonus-style in 216.1's Delta 6, confirmed in 216.2/216.3) is coherent, complete, documented as the canonical mechanism, and exercised by composite probes that compose all three collection types recursively.
**Type:** Sonnet Mode A.
**Time budget:** 30-45 min target; 60 min STOP.
**Depends on:** Stones 216.1 (`b478ff4` — HashSet + predicate function + bonus Vector/HashMap arms), 216.2 (`e4a63ed` — Vector confirmed), 216.3 (`fdc5031` — HashMap confirmed).
**Unblocks:** Stone 216.5 (sandbox walker validation), Stone 216.6 (INSCRIPTION + closure).

## Goal

Stone 216.4 is the **verification stone**, not a feature stone. The predicate has already been pre-landed; this stone (a) audits it, (b) consolidates the documentation surface, (c) adds composite probes that exercise the recursive predicate across all three collection types, and (d) inscribes a SCORE doc that honestly records the predicate's current state.

Per DESIGN Q6, the predicate is:
```
atomizable(T) :=
  T ∈ {primitives, HolonAST, WatAST}     // arc 215 baseline
  OR T = HashMap<K, V>  ∧ atomizable(K) ∧ atomizable(V)
  OR T = Vector<T'>      ∧ atomizable(T')
  OR T = HashSet<T'>     ∧ atomizable(T')
```

Per Stone 216.1 SCORE Row 4 + Delta 6, this lives at `src/check.rs:3600` as `fn is_atomizable(ty: &TypeExpr) -> bool` and includes all three collection arms (pre-emptively added in 216.1; confirmed correct in 216.2 + 216.3).

## Pre-flight verified

- Stone 216.1 SCORE Delta 6: predicate includes Vector + HashMap arms (pre-emptive)
- Stone 216.2 SCORE Row 4: confirmed `wat::core::Vector` arm at `src/check.rs:3642`
- Stone 216.3 SCORE Row 5: confirmed `wat::core::HashMap` arm at `src/check.rs:3644`
- All three collections round-trip cleanly (216.1/216.2/216.3 probe suites: 10 + 12 + 14 = 36 PASS)
- Baseline tests green (all 10 probe suites + 824 lib unit tests; one pre-existing failure `wat_arc170_slice_1f_alpha_helpers` tracked as task #413; not introduced by 216.x)

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

1. **Audit `fn is_atomizable`** in `src/check.rs:3600`:
   - Confirm all three collection arms (HashSet, Vector, HashMap) recurse correctly into type arguments
   - Confirm the primitive baseline (arc 215: i64, String, bool, keyword, byte, HolonAST, WatAST, char) is complete
   - Confirm the function handles type aliases correctly (e.g., `:wat::program::Env` typealias from Slice 4 Stone 4.1 → `HashMap<keyword, HolonAST>` → atomizable)
   - Document any inconsistencies; fix if minimal; flag if structural

2. **Consolidate comments** at the predicate site:
   - Remove any "Stone N future" comments that are now stale (Stones 216.1/216.2/216.3 all shipped)
   - Add a single canonical doc-comment on `fn is_atomizable` that names the four atomizable categories: primitives, HolonAST, WatAST, collections-of-atomizable

3. **Verify the special-case arm in `infer_list`** (Stone 216.1 added it):
   - `:wat::holon::Atom | :wat::holon::leaf` arm
   - Inspect; confirm it correctly applies `is_atomizable(resolved)` after inferring the arg type
   - Confirm the diagnostic on failure names the offending position (per DESIGN Q6)

4. **WAT-CHEATSHEET consolidation pass**:
   - Single canonical "Atomizable types" section listing all four categories
   - Remove per-stone "future" markers that should now read "shipped"
   - Reference `fn is_atomizable` as the canonical mechanism (line citation)
   - Atomizable composition examples: `Atom<HashMap<keyword, Vector<HashSet<i64>>>>` passes; `Atom<HashMap<keyword, Function<...>>>` fails

5. **Composite probe suite** — `tests/probe_arc216_stone4_predicate_composition.rs` with ~6 probes:
   - Probe 1: `:wat::holon::Atom (:wat::std::HashMap :wat::core::keyword (:wat::std::Vector :wat::core::i64))` — HashMap-of-Vector at type-check passes
   - Probe 2: `:wat::holon::Atom (:wat::std::Vector (:wat::std::HashSet :wat::core::i64))` — Vector-of-HashSet at type-check passes
   - Probe 3: `:wat::holon::Atom (:wat::std::HashSet (:wat::std::Vector :wat::core::i64))` — HashSet-of-Vector at type-check passes
   - Probe 4: Triple-nested — `:wat::holon::Atom (:wat::std::HashMap :wat::core::keyword (:wat::std::Vector (:wat::std::HashSet :wat::core::i64)))` — all three collections nested; type-check passes
   - Probe 5: Negative — `:wat::holon::Atom (:wat::std::Vector :wat::core::Function)` (or similar non-atomizable T) — type-check fails with diagnostic naming non-atomizable position
   - Probe 6: Negative — `:wat::holon::Atom (:wat::std::HashMap :wat::core::Function :wat::core::i64)` — non-atomizable K fails

   Probes should construct values at the WAT surface and verify `(:wat::holon::Atom v)` compiles + runs (or fails at check with a TypeMismatch naming the non-atomizable type).

6. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.4.md` — scorecard with row count matching EXPECTATIONS; honestly document that the predicate code was pre-landed and this stone's work was verification + composition probes + doc consolidation.

## NOT your scope

- Sandbox-walker validation — Stone 216.5
- INSCRIPTION — Stone 216.6
- Predicate refactor beyond comment + doc consolidation — only fix structural inconsistencies if found; do not gold-plate
- WARD-PASS, INTERSTITIAL — orchestrator post-ship

## STOP triggers

- STOP-1: predicate found incoherent (e.g., missing a primitive type, wrong recursion) — surface; fix scope grows
- STOP-2: composite probe surfaces a runtime bug not previously visible — surface; flag whether it's a 216.4 fix or a follow-up arc
- STOP-3: any existing test fails — surface
- STOP-4: 60 min elapsed

## Verification

Single commands per line:

```
cargo build --release
cargo test --release --test probe_arc216_stone4_predicate_composition -p wat
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip -p wat
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

Report: pass count out of EXPECTATIONS row count, deltas, verification summary, elapsed time, predicate audit findings.

Don't commit. Orchestrator commits after review.
