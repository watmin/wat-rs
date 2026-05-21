# BRIEF — Arc 216 Stone 216.5d — DELETE `hashmap_key`; the poison is purged

**Stone:** the endgame of the antidote sequence. `hashmap_key` no longer has a reason to exist — Stone 216.5b moved HashSet to native `HashSet<Value>` storage, Stone 216.5c moved HashMap to native `HashMap<Value, Value>` storage. Both use `Value`'s native `Hash + Eq` from Stone 216.5a. This stone audits remaining callers, refactors any stragglers to native `Value::Hash`, deletes `fn hashmap_key` entirely with all 9 arms, decides the fate of `value_is_hashable`, and updates the WAT-CHEATSHEET. **After this stone, the canonical-key crutch is structurally unrepresentable in the substrate.**
**Type:** Sonnet Mode A.
**Time budget:** 60-90 min target; 105 min STOP.
**Depends on:** Stone 216.5a (`e404056`, impl Hash for Value), Stone 216.5b (`ff5f86d`, HashSet refactor), Stone 216.5c (`b98d42a`, HashMap refactor + value_is_hashable unification).
**Unblocks:** Stone 216.6 (sandbox-walker validation), Stone 216.7 (INSCRIPTION — arc 216 closes with the substrate impeccable).

## Why this stone exists

Read DESIGN.md "Antidote stones (216.5a-d)" section. Stone 216.5d is the final removal — the poison is gone after this stone, not just neutralized.

## The transformation

```rust
// Before this stone:
pub fn hashmap_key(op: &str, v: &Value) -> Result<String, RuntimeError> {
    match v {
        Value::String(s) => Ok(format!("S:{}", s)),
        Value::i64(n) => Ok(format!("I:{}", n)),
        // ... 7 more arms ...
        Value::Vec(xs) => /* recursive */,        // Stone 216.5 throw-away
        Value::wat__std__HashMap(m) => /* recursive */,  // Stone 216.5 throw-away
        Value::wat__WatAST(ast) => /* recursive */,      // Stone 216.5 throw-away
        other => Err(RuntimeError::TypeMismatch { ... }),
    }
}

// After this stone:
// (nothing; the function is gone)
```

## Pre-flight verified

- Stone 216.5a SHIPPED — `impl Hash + PartialEq + Eq for Value` (foundation)
- Stone 216.5b SHIPPED — HashSet storage native; `value_is_set_hashable` defensive guard
- Stone 216.5c SHIPPED — HashMap storage native; `value_is_hashable` unified; 20 caller sites swept
- All probe suites GREEN at commit `b98d42a`
- `cargo clippy` — 111 pre-existing errors; 0 new from antidote sequence

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

### Part A — Audit + refactor remaining `hashmap_key` callers

1. **Comprehensive grep:** `grep -rn "hashmap_key" src/ tests/`. Expected callers after 216.5b+c:
   - Internal recursion within `fn hashmap_key` itself (dies with the function)
   - Any straggler from the 18-site audit that wasn't swept by 216.5b/c

2. **For each straggler caller:**
   - Determine what it's hashing and why
   - Refactor to use native `Value::Hash` via `std::collections::hash_map::DefaultHasher` (or similar) inline
   - OR refactor to use the new native HashSet/HashMap storage directly
   - Surface any caller whose refactor is non-obvious (STOP)

### Part B — DELETE the function

3. **Delete `fn hashmap_key`** at `src/runtime.rs:9330+` entirely:
   - The function declaration
   - All 9 match arms (String, i64, f64, bool, keyword, HolonAST, Uuid, HashSet, Vec, HashMap, WatAST)
   - The doc-comment block (the long block documenting canonical-key schemes)
   - The `other =>` TypeMismatch diagnostic

4. **Delete the THREE throw-away arms** added in Stone 216.5 (they die with the function — listed for completeness in the SCORE):
   - Vec arm (length-prefix scheme)
   - HashMap arm (sorted-pairs scheme)
   - WatAST arm (Debug-string DefaultHasher)

5. **Update any imports** that referenced `hashmap_key` — remove them.

### Part C — `value_is_hashable` decision

6. **Decide the fate of `value_is_hashable` + `value_is_set_hashable` + `value_is_key_hashable`** (the unified predicate + two wrappers from 216.5c):
   - **Option α (recommended):** KEEP all three. They're defense-in-depth for Rust-level code paths (closure_extract.rs, edn_shim.rs) that don't go through check.rs's `is_atomizable` predicate. Their job is preventing `unreachable!()` panics from `impl Hash for Value` on opaque-handle Values. Independent of `hashmap_key`'s existence.
   - **Option β:** RETIRE if check.rs's `is_atomizable` is sufficient — but this would expose the unreachable!() arms to any caller bypassing check (closure_extract, edn_shim, future Rust code). Risky.
   - Sonnet's call; document in SCORE with rationale.

### Part D — WAT-CHEATSHEET update

7. **Update `docs/WAT-CHEATSHEET.md`:**
   - The "Hashable types" subsection (added in Stone 216.5) — REWRITE to describe the new canonical mechanism: `impl Hash for Value` mirroring HolonAST; `is_atomizable` predicate at `src/check.rs:3623` is the check-time gate; `value_is_hashable` at `src/runtime.rs` is the runtime defense-in-depth
   - Remove all references to `hashmap_key` (the function is gone)
   - Remove the canonical-key scheme documentation (the schemes don't exist any more)
   - Keep the "Atomizable types" subsection (the predicate is unchanged)

### Part E — Probes

8. **No new probe file needed.** The deletion is validated by EXISTING probes staying green:
   - `probe_verify_hashset_of_vector_gap` — confirms the original arc 216.5 bug doesn't regress (the canonical-key crutch is gone; the gap can't reopen because the mechanism doesn't exist)
   - `probe_arc216_stone5_hashmap_key_coverage` — wait, this tests `hashmap_key` directly. **DELETE this test file** as part of this stone (its subject — the function — no longer exists; the coverage it tested is now provided by 216.5b + 216.5c probe matrices)
   - All other 216.x probe suites stay green
   - Add 1-2 probes if useful to assert the deletion: e.g., a Rust-level test that confirms `hashmap_key` is not defined (won't compile if it is) — but this is automatic via the build itself. Skip unless sonnet sees value.

9. **Update `tests/probe_verify_hashset_of_vector_gap.rs`** — update the doc-comment to reflect "the gap is closed; this probe is now historical evidence; the canonical-key crutch that caused the gap no longer exists." Keep the test (still passes; documents the gap that was there).

### Part F — Documentation

10. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5d.md` — scorecard matching EXPECTATIONS row count; document:
    - Straggler caller count (how many remained after 216.5b+c; refactor approach for each)
    - `value_is_hashable` decision (Option α vs β) with rationale
    - Line count deleted (delta from removing hashmap_key + arms + docs)
    - WAT-CHEATSHEET deltas
    - Probe file deletion (`probe_arc216_stone5_hashmap_key_coverage.rs`)

11. **DESIGN.md** — no further forward-correction needed; the stepping stones inscribed for 216.5a-d are now complete; 216.7 INSCRIPTION will close the arc.

## NOT your scope

- **Sandbox-walker validation** — Stone 216.6
- **INSCRIPTION + closure** — Stone 216.7
- **Refactoring is_atomizable predicate** — out of scope (the Tuple/Option/Result observation from 216.5a is a future arc)
- **Any architectural change beyond removing hashmap_key + the 3 throw-away arms**

## STOP triggers

- **STOP-1: straggler caller refactor non-obvious.** If a `hashmap_key` caller doesn't have an obvious native-Hash translation, surface; orchestrator decides whether to extend scope or defer.
- **STOP-2: probe regression.** If deleting `hashmap_key` breaks an existing probe (other than the one being deleted), STOP. The deletion is wrong or incomplete; surface the regression.
- **STOP-3: `value_is_hashable` retirement risk.** If you're considering Option β (retiring the guards), DOUBLE-CHECK that every Hash-impl-reaching path goes through check.rs. If unsure, choose Option α.
- **STOP-4: probe deletion surprise.** If `probe_arc216_stone5_hashmap_key_coverage.rs` has probes that DON'T just test `hashmap_key` (e.g., they test downstream user-facing behavior), surface; we may want to keep those probes adapted rather than delete the whole file.
- **STOP-5: any existing probe fails** — surface.
- **STOP-6: 105 min elapsed.**

## Verification

Single commands per line:

```
cargo build --release
cargo test --release --test probe_arc216_stone5c_hashmap_native_storage -p wat
cargo test --release --test probe_arc216_stone5b_hashset_native_storage -p wat
cargo test --release --test probe_arc216_stone5a_value_hash -p wat
cargo test --release --test probe_verify_hashset_of_vector_gap -p wat
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

(Note: `probe_arc216_stone5_hashmap_key_coverage` is NOT in this list because the test file is deleted as part of this stone — its subject no longer exists.)

## When you finish

Report: pass count out of EXPECTATIONS row count, deltas, verification summary, elapsed time, straggler caller count, `value_is_hashable` decision (α/β), line count deleted, *the poison is purged*.

Don't commit. Orchestrator commits after review.
