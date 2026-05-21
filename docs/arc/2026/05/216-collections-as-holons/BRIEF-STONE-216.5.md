# BRIEF — Arc 216 Stone 216.5 — `hashmap_key` full coverage (substrate fix)

**Stone:** close the predicate→runtime gap that Stone 216.4 surfaced. Make `is_atomizable(T) → hashmap_key` always honors the contract: every atomizable type must be hashable through `hashmap_key`. After this stone, `HashSet<Vector<T>>`, `HashSet<HashMap<K,V>>`, `HashMap<Vector<T>, V>`, `HashMap<HashMap<K,V>, V>`, `HashMap<WatAST, V>` all round-trip cleanly at the WAT surface. Arc 216's thesis ("class of failure eliminated: values that look HolonRepresentable but silently aren't at runtime") becomes TRUE on the branch.
**Type:** Sonnet Mode A.
**Time budget:** 60-90 min target; 105 min STOP.
**Depends on:** Stones 216.1 (`b478ff4`, HashSet + pre-emptive predicate arms), 216.2 (`e4a63ed`, Vector), 216.3 (`fdc5031`, HashMap), 216.4 (`987e13c`, predicate consolidation — surfaced this gap).
**Unblocks:** Stone 216.6 (sandbox-walker validation), Stone 216.7 (INSCRIPTION + closure).

## Why this stone exists (lineage)

Read the DESIGN.md "Forward-correction (Stone 216.5 onward)" section BEFORE starting. The short version:

- Stone 216.1 Delta 6 pre-emptively added Vector + HashMap arms to `is_atomizable` "for future stones." SCORE called this "predicate is slightly ahead of the runtime" — that was an error report, not honest documentation.
- Stones 216.2/216.3 did not audit the cross-product (HashSet<Vector<T>>, HashMap<Vector,V>, etc.).
- Stone 216.4 (verification) hit `HashSet<Vector<i64>>` failing at runtime and SUBSTITUTED the probe's type to `HashSet<HashSet<i64>>` to make it pass. The substitution was logged as Delta 2 and the gap was labeled "follow-up arc."
- Orchestrator caught the substitution + label in post-ship review. The arc 216 thesis was found false on the branch. Stone 216.5 fixes the foundation.

A runtime verification probe is already on disk and currently RED:
- `tests/probe_verify_hashset_of_vector_gap.rs` — run it FIRST; see it fail with `TypeMismatch { op: ":wat::core::HashSet", expected: "hashable value (primitive, HolonAST, or HashSet<T>)", got: "wat::core::Vector" }`. That is the bug. When it goes green, the fix is real.

## Goal

Three new arms in `hashmap_key` at `src/runtime.rs:9330`:

1. **`Value::Vec(v)`** → canonical key `"Vec:[k1,k2,k3,...]"` (order preserved; recursive `hashmap_key` per element; deterministic separator escaping or hash-based fallback for collision safety — sonnet picks)
2. **`Value::wat__std__HashMap(m)`** → canonical key `"Map:{(k1=v1),(k2=v2),...}"` (sorted by k for determinism since HashMap has no order; both K and V recursive)
3. **`Value::wat__WatAST(ast)`** → mirror HolonAST's `Hash`+DefaultHasher pattern at lines 9337-9343

Plus: update the `other =>` diagnostic message to enumerate the new accepted set honestly. The current message lies by omission.

## Pre-flight verified

- `hashmap_key` at `src/runtime.rs:9330-9363` — current arms: String, i64, f64, bool, keyword, HolonAST, Uuid, HashSet. Missing: Vec, HashMap, suspected WatAST.
- `is_atomizable` at `src/check.rs:3623` — arms include Vector, HashMap, HashSet, plus WatAST in primitive baseline.
- `tests/probe_verify_hashset_of_vector_gap.rs` — failing probe demonstrates the gap end-to-end.
- All 216.x SCOREs already shipped + readable.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

### Part A — Audit + extend `hashmap_key`

1. **Run the failing probe first:** `cargo test --release --test probe_verify_hashset_of_vector_gap -p wat -- --nocapture`. Capture the red output. This is the bug you're fixing.

2. **Audit `hashmap_key` vs `is_atomizable`:** every atomizable type needs a `hashmap_key` arm. Verify the gap matches what's claimed above (Vec, HashMap, WatAST). If you find OTHER gaps, surface them BEFORE adding arms — do not silently extend.

3. **Add the three arms** with canonical-key schemes:
   - **Vec:** `Value::Vec(v)` — `"Vec:[k1,k2,...]"`. Order preserved. Recursive `hashmap_key` for each element. Pick a separator/escape scheme that's collision-safe (commas in strings? quote them? OR use length-prefix? OR fall back to hash-of-serialized-form? Sonnet picks; documents the choice in the doc-comment).
   - **HashMap:** `Value::wat__std__HashMap(m)` — `"Map:{(k1=v1),(k2=v2),...}"`. Sort by k for determinism. Both K and V recursive.
   - **WatAST:** `Value::wat__WatAST(ast)` — mirror the HolonAST pattern at lines 9337-9343 (DefaultHasher; `"W:{hash}"`).

4. **Update the `other =>` diagnostic message** to enumerate the new accepted set: `"hashable value (primitive, HolonAST, WatAST, HashSet<T>, Vec<T>, or HashMap<K,V>)"` (or whatever the actual final set is — match reality).

### Part B — Audit all `hashmap_key` call sites

5. **Grep for callers:** `grep -n "hashmap_key(" src/`. Known: `eval_hashset_ctor:9468`, `eval_hashmap_ctor:9419`. Verify ALL callers benefit from the fix uniformly. If any caller has its own pre-filter or special-case that would BLOCK the new arms from helping, surface it.

### Part C — Probe matrix

6. **Symmetric probe suite** at `tests/probe_arc216_stone5_hashmap_key_coverage.rs` (~12 probes):
   - Probe 1: `HashSet<Vector<i64>>` round-trip (the failing probe's positive twin)
   - Probe 2: `HashSet<HashMap<keyword, i64>>` round-trip
   - Probe 3: `HashSet<WatAST>` round-trip (if WatAST values can be constructed at WAT surface; if not, skip with documented reason)
   - Probe 4: `HashMap<Vector<i64>, String>` round-trip (Vector as K)
   - Probe 5: `HashMap<HashMap<keyword, i64>, String>` round-trip (HashMap as K)
   - Probe 6: `HashMap<WatAST, String>` round-trip (if constructible)
   - Probe 7: Nested — `HashSet<Vector<HashSet<i64>>>` round-trip
   - Probe 8: Nested — `HashMap<Vector<i64>, HashSet<i64>>` round-trip
   - Probe 9: Dedupe semantics — `HashSet<Vector<i64>>` with two equal-content Vectors collapses (canonical key matches)
   - Probe 10: Diagnostic — the new `other =>` message enumerates the full accepted set
   - Probe 11: Collision safety — two Vecs with content `["a", "b,c"]` and `["a,b", "c"]` produce DIFFERENT canonical keys (catches naive comma-join schemes)
   - Probe 12: HolonRepresentable cascade — `HashSet<Vec<String>>::from_holon_ast` round-trip at Rust level

7. **Flip `tests/probe_verify_hashset_of_vector_gap.rs` to green** — same probe; same assertion; just no longer panics. Keep the file as documentation of the gap that existed.

### Part D — Reland 216.4 Probe 3

8. **`tests/probe_arc216_stone4_predicate_composition.rs` Probe 3** — flip from sonnet's `HashSet<HashSet<i64>>` substitution back to the original BRIEF's `HashSet<Vector<i64>>`. Update the doc-comment to acknowledge the original substitution + the 216.5 reland. Confirm 11/11 still passes.

### Part E — Documentation

9. **WAT-CHEATSHEET update** — extend the atomizable-types section with a NEW subsection ("Hashable types") naming the symmetric runtime contract: atomizable T → hashable through `hashmap_key`. Cross-reference the canonical-key schemes.

10. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5.md` — scorecard matching EXPECTATIONS row count; honestly document any judgment calls (separator scheme, WatAST constructibility at WAT surface, etc.).

## NOT your scope

- Sandbox-walker validation — Stone 216.6
- INSCRIPTION + closure — Stone 216.7
- Any pre-emptive code beyond the three arms + their direct support
- Any refactor of `hashmap_key` beyond the three new arms + the diagnostic message

## STOP triggers (sharpened post-216.4)

- **STOP-1 (NEW): pre-emptive code beyond stone's scope.** If you find yourself wanting to add an arm or branch "for the next stone," STOP. Either write the failing test that the next stone will make pass, or pull the code OUT of this stone. Do not ship code without a passing test.
- **STOP-2 (NEW): probe substitution.** If a probe in your matrix fails because the runtime can't support what the probe is testing, STOP. Do NOT substitute a different type to make the probe pass. Return to orchestrator with the surfaced gap; orchestrator decides scope. (Stone 216.4 Delta 2 was the failure mode this trigger guards against.)
- **STOP-3: new substrate gap surfaced.** If audit reveals MORE missing arms than {Vec, HashMap, WatAST}, STOP. Surface them; orchestrator decides whether to absorb or open a new stone.
- **STOP-4: collision-safety scheme has subtle bugs.** If the canonical-key scheme you pick has hard-to-prove collision safety, surface the concern; consider hash-based fallback.
- **STOP-5: any existing test fails** — surface.
- **STOP-6: 105 min elapsed.**

## Verification

Single commands per line:

```
cargo build --release
cargo test --release --test probe_verify_hashset_of_vector_gap -p wat
cargo test --release --test probe_arc216_stone5_hashmap_key_coverage -p wat
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

Report: pass count out of EXPECTATIONS row count, deltas (substitutions are STOP triggers, not deltas — surface them via STOP), verification summary, elapsed time, audit findings (additional gaps if any), canonical-key scheme choice.

Don't commit. Orchestrator commits after review.
