# BRIEF — Arc 216 Stone 216.5a — `impl Hash for Value` + `impl PartialEq + Eq for Value`

**Stone:** mint the antidote molecule. `Value` becomes natively hashable + structurally equal, mirroring `HolonAST` in holon-rs. NO callers touched in this stone; the new mechanism just EXISTS. Subsequent stones (216.5b, 216.5c, 216.5d) consume it to retire the canonical-key crutch (`hashmap_key`).
**Type:** Sonnet Mode A.
**Time budget:** 60-90 min target; 105 min STOP.
**Depends on:** Stone 216.5 (`8a6c12f`) — canonical-key extension + 12-probe matrix + caller audit (the work to be retired by 216.5d).
**Unblocks:** Stone 216.5b (HashSet storage refactor), 216.5c (HashMap storage refactor), 216.5d (`hashmap_key` deletion).

## Why this stone exists

Read DESIGN.md "Antidote stones (216.5a-d)" section first — names the poison + the antidote + the stepping stones.

The short version: `hashmap_key` is a String-canonical-key serialization crutch that exists because `Value` doesn't implement `Hash`. The crutch has metastasized through 18 call sites. Stone 216.5 extended the crutch to close a runtime gap, but the crutch itself is the wrong shape. `holon-rs`'s `HolonAST` already shows the right pattern: per-variant payload hash + `std::mem::discriminant` tagging + `f64::to_bits()` + zero allocation. This stone applies that pattern to `Value`.

## Template — read this Rust file

`/home/watmin/work/holon/holon-rs/src/kernel/holon_ast.rs:158-232` — the canonical pattern for `impl PartialEq + Eq + Hash` on a structural enum with float variants. Mirror it.

## Goal

1. **`impl PartialEq + Eq for Value`** with NaN-safe equality:
   - All variants compare structurally (recurse via PartialEq on payloads)
   - `Value::f64(a)` vs `Value::f64(b)` → `a.to_bits() == b.to_bits()` (NaN-safe; standard pattern)
   - Variants with f64 fields (none currently, but in case) → same pattern
   - Non-atomizable variants (Fn, ProgramHandle, etc.) — see strategy below

2. **`impl Hash for Value`** with discriminant tagging:
   - `std::mem::discriminant(self).hash(state)` first (prevents `Bool(true) == I64(1)` collisions)
   - Per-variant payload hashing (recurse via Hash on payloads)
   - `Value::f64(x)` → `x.to_bits().hash(state)` (NaN-safe)
   - Recursive variants (HashSet, HashMap, Vec, HolonAST, WatAST) compose for free via std lib's `Hash` impls on `Vec<T>`, `HashSet<T>`, `HashMap<K,V>`, plus existing `Hash` on HolonAST
   - Non-atomizable variants → see strategy below

3. **Non-atomizable variant strategy (Option A per DESIGN verdict):**
   - For variants where structural Hash/Eq is undefined (Fn, function values, ProgramHandle, channel handles, etc.):
     - `Hash::hash`: `unreachable!("Value::{variant} is not atomizable; is_atomizable predicate at check time should have rejected this. If you see this panic, the predicate has drifted.")`
     - `PartialEq::eq`: pointer equality where the variant wraps `Arc<_>`, or `unreachable!()` if no sensible equality exists
   - The `is_atomizable` predicate at `src/check.rs:3623` is the static guarantee; `unreachable!()` is the runtime assertion of the same invariant

## Pre-flight verified

- `holon-rs/src/kernel/holon_ast.rs:158-232` — the template (HolonAST's PartialEq + Eq + Hash impls)
- `src/runtime.rs` Value enum — sonnet audits the variant list as first step
- `src/check.rs:3623` — `fn is_atomizable` — the static guarantee that gates the unreachable!() path
- Stone 216.5 probe matrix (12 probes + verify-probe) — all GREEN; 216.5a must not regress them
- All 9 arms in `hashmap_key` at `src/runtime.rs:9330` — unchanged by this stone

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope

### Part A — Audit (don't extend yet)

1. **Find the Value enum definition.** `grep -n "pub enum Value" src/runtime.rs`. Read every variant.

2. **Check current trait impls on Value.** Does it derive `PartialEq` / `Eq`? Does it have manual impls? Is `Hash` impl'd anywhere?

3. **Classify variants** into:
   - **Atomizable** (per `is_atomizable` predicate): String, i64, f64, bool, keyword, HolonAST, WatAST, Uuid, HashSet, HashMap, Vec (and any others the predicate accepts)
   - **Non-atomizable**: function values (Fn variants), ProgramHandle, channel handles, anything else not in the atomizable set

   Document the classification in the SCORE doc.

### Part B — `impl PartialEq + Eq for Value`

4. **Replace the current PartialEq derive (if present) with a manual impl** mirroring HolonAST's pattern:
   - Match on `(self, other)` tuple
   - Per atomizable-variant pair: structural comparison (recurse via PartialEq on payloads)
   - f64 cases: `a.to_bits() == b.to_bits()`
   - Non-atomizable-variant pairs: `unreachable!()` OR pointer equality (`Arc::ptr_eq`) — sonnet picks per honesty; documents
   - `(_, _) => false` for cross-variant pairs

5. **`impl Eq for Value {}`** — Eq is a marker trait; safe per the NaN-bit-pattern equality

### Part C — `impl Hash for Value`

6. **Mirror HolonAST's Hash impl exactly:**
   ```rust
   impl Hash for Value {
       fn hash<H: Hasher>(&self, state: &mut H) {
           std::mem::discriminant(self).hash(state);
           match self {
               Value::String(s) => s.hash(state),
               Value::i64(n) => n.hash(state),
               Value::f64(x) => x.to_bits().hash(state),
               Value::bool(b) => b.hash(state),
               Value::wat__core__keyword(k) => k.hash(state),
               Value::holon__HolonAST(h) => h.hash(state),  // HolonAST already Hash
               Value::wat__core__Uuid(u) => u.hash(state),
               Value::wat__std__HashSet(s) => /* recursive */,
               Value::wat__std__HashMap(m) => /* recursive */,
               Value::Vec(v) => v.hash(state),  // std lib Vec<T>: Hash composes
               Value::wat__WatAST(ast) => /* see Part D */,
               // Non-atomizable variants:
               _ => unreachable!("Value::{variant} is not atomizable; is_atomizable predicate at check time should have rejected this."),
           }
       }
   }
   ```

7. **For HashSet/HashMap variants** — their current storage is `Arc<HashMap<String, Value>>` / `Arc<HashMap<String, (Value, Value)>>` (the canonical-key crutch). 216.5b/c will refactor the storage. For 216.5a, hash the current storage form:
   - `Value::wat__std__HashSet(s)` → iterate s.values() (the Values themselves, not the String keys); collect into a sorted Vec by recursive Value Hash; hash. This is deterministic + matches the canonical-key crutch's set semantics.
   - `Value::wat__std__HashMap(m)` → iterate m.values() (which gives `(k_val, v_val)` tuples); collect into sorted Vec of (k_hash, v_hash); hash.
   - **Important:** the canonical-key String is NOT used. The new Hash impl bypasses the crutch entirely (which is the point).

### Part D — `Value::wat__WatAST` consideration

8. **WatAST currently does not implement Hash** (per Stone 216.5's audit; Debug-string DefaultHasher was used as workaround). Decide:
   - **Option D1:** `impl Hash for WatAST` directly (mirror HolonAST's pattern at the WatAST enum). Adds a small new impl but matches the pattern. Preferred if WatAST's variants are well-defined.
   - **Option D2:** Use Debug-string DefaultHasher in the Value::wat__WatAST arm (mirrors Stone 216.5's choice; less clean but localized).

   Sonnet picks; documents. D1 is cleaner if scope allows.

### Part E — Probes

9. **Rust-level probe suite** at `tests/probe_arc216_stone5a_value_hash.rs` (~10 probes):
   - Probe 1: Self-equality — `assert_eq!(hash(&v), hash(&v))` for each atomizable variant
   - Probe 2: Discriminant tagging — `hash(&Value::bool(true)) != hash(&Value::i64(1))` (different variants, same payload)
   - Probe 3: NaN-safety — `Value::f64(f64::NAN) == Value::f64(f64::NAN)` (bit-pattern equality); hash same
   - Probe 4: Recursive composition — `HashSet<Value>` literal builds + queries; `HashMap<Value, Value>` literal builds + queries (Rust-level, no WAT)
   - Probe 5: HolonAST nesting — `Value::holon__HolonAST(...)` hashes consistently via existing HolonAST::Hash
   - Probe 6: Vec composition — `Value::Vec(vec![Value::i64(1), Value::i64(2)])` hashes; reverse-order Vec hashes DIFFERENTLY (order preserved)
   - Probe 7: HashSet composition — two HashSet Values with same elements (different insertion order) hash IDENTICALLY (set semantics; sort-then-hash)
   - Probe 8: HashMap composition — two HashMap Values with same pairs (different insertion order) hash IDENTICALLY (map semantics; sort-then-hash)
   - Probe 9: Deep nesting — `Value::Vec(vec![Value::wat__std__HashMap(...)])` hashes consistently
   - Probe 10: Non-atomizable panic — construct a Fn Value (if possible at this layer); assert `hash(&fn_value)` panics with the expected `unreachable!()` message. Use `std::panic::catch_unwind` to assert the panic; document if Fn construction isn't accessible at this test layer.

### Part F — NO caller refactor

10. **Do NOT touch `hashmap_key`.** Do NOT touch `eval_hashset_ctor`, `eval_hashmap_ctor`, or any of the 18 callers. This stone is foundation-only.

11. **Verify existing tests still pass** — the 216.5 probe matrix + all prior probes stay green. The new Hash + PartialEq impls coexist with the canonical-key crutch.

### Part G — Documentation

12. **SCORE doc** at `docs/arc/2026/05/216-collections-as-holons/SCORE-STONE-216.5a.md` — scorecard matching EXPECTATIONS row count; document the variant classification (atomizable vs non-atomizable list); document the WatAST decision (D1 vs D2); document the panic-vs-Arc::ptr_eq decision for non-atomizable PartialEq.

13. **No WAT-CHEATSHEET update** — `hashmap_key` documentation stays as-is (it's still the canonical mechanism until 216.5d deletes it).

## NOT your scope

- **Storage refactor of HashSet/HashMap** — Stone 216.5b/c
- **Deletion of `hashmap_key`** — Stone 216.5d
- **ANY caller refactor** — all 18 sites untouched in this stone
- **WatAST PartialEq+Eq change** — if WatAST already implements them, leave as-is; only add Hash if you choose D1
- **WAT-surface probes** — Rust-level only

## STOP triggers

- **STOP-1: caller refactor.** If you find yourself wanting to "improve" a HashSet or HashMap caller because the new Hash impl makes it cleaner, STOP. That's 216.5b/c scope. Leave it alone.
- **STOP-2: storage refactor temptation.** If you want to change `Value::wat__std__HashSet`'s storage from `HashMap<String, Value>` to `HashSet<Value>` because "it would be so easy now," STOP. That's 216.5b. Leave it.
- **STOP-3: probe substitution.** If a probe fails because the impl doesn't quite work, STOP. Do not substitute a different probe; fix the impl or surface the constraint.
- **STOP-4: non-atomizable variant ambiguity.** If a variant's atomizability isn't obvious, surface it; do NOT silently put it in the unreachable!() arm if it might be reachable.
- **STOP-5: existing test fails.** Surface.
- **STOP-6: 105 min elapsed.**

## Verification

Single commands per line:

```
cargo build --release
cargo test --release --test probe_arc216_stone5a_value_hash -p wat
cargo test --release --test probe_arc216_stone5_hashmap_key_coverage -p wat
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

## When you finish

Report: pass count out of EXPECTATIONS row count, deltas (NOT substitutions), verification summary, elapsed time, variant classification, WatAST D1/D2 decision, non-atomizable PartialEq strategy.

Don't commit. Orchestrator commits after review.
