# BRIEF — Arc 233 Stone 233.2.k — Value::Tracked variant retirement + Environment stores TrackedValue

## What we're doing

Retire `Value::Tracked` entirely. The Stone 233.2.j Phase 5 exemption (bind_let_binding re-wrap) dissolves PERMANENTLY by flipping `Environment.bindings` storage from `HashMap<String, Value>` to `HashMap<String, TrackedValue>` (Option A from the sub-DESIGN). After this stone, the wrapping-variant trap-door class has zero living instances in source; Stone 233.2.l seals the meta-class via `#[wat_value]` proc-macro.

**The cascade:**

1. **Environment storage flip** — `src/runtime.rs:1267` + `1304`:
   ```rust
   bindings: HashMap<String, Value>,    // before
   bindings: HashMap<String, TrackedValue>,  // after
   ```

2. **Environment API signature flips:**
   - `Environment::lookup(&self, name: &str) -> Option<Value>` → `Option<TrackedValue>` (`src/runtime.rs:1288`)
   - `EnvironmentBuilder::bind(name, value: Value)` → `bind(name, tv: TrackedValue)`
   - `bindings.insert` at line 1310 receives `TrackedValue` directly
   - 6 known lookup callers update (probably already calling `.into_tracked()` post-lookup — that step becomes redundant)

3. **`bind_let_binding` simplification** — `src/runtime.rs:6195`+:
   ```rust
   // After 233.2.k
   LetBinding::Single { name, rhs } => {
       let tv = eval_inner(rhs, scope, sym)?;
       Ok(scope.child().bind(name, tv).build())  // direct; no re-wrap
   }
   LetBinding::Destructure { names, rhs } => {
       let value = eval_inner(rhs, scope, sym)?.value_owned();
       let elements = destructure_tuple(&value, names.len(), ":wat::core::let")?;
       let mut builder = scope.child();
       for (name, elem) in names.iter().zip(elements) {
           // Destructure slots get Unknown provenance — each slot has its own
           // origin which we'd need separate tracking for (out of scope for
           // 233.2.k; arc 233.2.e revisits if/when destructure provenance becomes
           // load-bearing).
           builder = builder.bind(name, TrackedValue::from(elem));
       }
       // ... rest of destructure handling ...
   }
   ```

4. **DELETE `Value::Tracked` variant** from `pub enum Value { ... }` definition (around `src/runtime.rs:613`).

5. **DELETE helpers** on `impl Value`:
   - `Value::inner()` at lines 1166-1170
   - `Value::provenance()` at lines 1178-1183
   - `Value::into_tracked()` at lines 1188-1201

6. **Remove dead match arms** (variant no longer exists; compile error otherwise):
   - `src/runtime.rs:1011` — Hash impl `unreachable!` arm
   - `src/runtime.rs:1159` — `type_name()` arm
   - `src/runtime.rs:17940` — `render_value()` `unreachable!` arm
   - `src/edn_shim.rs:1696` — `value_to_edn_with` passthrough arm
   - `src/closure_extract.rs:1733` — closure_extract handling arm
   - Any Eq/PartialEq Value::Tracked arms

7. **Strip `.inner()` call sites** — ~19 sites. With Value never wrapped, `.inner()` was a no-op (returned self when not Tracked). Replace `v.inner()` with `v` (or strip entirely if used inline like `match v.inner() { ... }` → `match v { ... }`).

8. **Replace `.into_tracked()` call sites** — ~26 sites. The helper's job was "extract Provenance from Tracked OR wrap with Unknown"; post-retirement only the second case matters; that's `TrackedValue::from(value)`. Mechanical replace.

9. **DELETE `tests/probe_value_tracked_transparency.rs`** — the 233.2.a probe tests the retired variant's transparency contracts. Per HARD CUT + inscription-immutable: probes for retired surface get deleted, not refactored. The file goes away entirely.

10. **Remove probe-3-exempt mechanism** from `tests/probe_stone_233_2_j_producer_migration.rs`:
    - Delete the `// #[probe-3-exempt: ...]` marker on the bind_let_binding line (the line itself goes away in Phase 3 above)
    - Delete the exemption detection logic in probe_3 (the `if line.contains("#[probe-3-exempt") { continue; }` block)
    - Update assertion message to remove the exemption-mechanism mention
    - The probe's intent (zero construction sites) is preserved; the exemption is gone because no construction sites remain.

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.k.md`** (commit `f830de8`) — sub-DESIGN; Option A verdict (Environment stores TrackedValue) over B/C; four-questions verdict; substrate-informed cascade scope. **Authoritative for shape decisions.**

2. **`tests/probe_stone_233_2_k_variant_retired.rs`** (commit `f43c577`) — FM 2-bis probe. 5 contracts. **The probe IS the success criterion** — sonnet flips:
   - Probe 1: many active Value::Tracked refs → 0
   - Probe 2: Value enum has Tracked variant → doesn't
   - Probe 3 (behavioral regression guard): passes both pre + post; verifies the Option A structural fix holds the same property the Phase 5 re-wrap did
   - Probe 4: Environment.lookup returns Option<Value> → Option<TrackedValue> (compile-shape)
   - Probe 5: Value helpers exist → deleted

3. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.j.md`** — 233.2.j shipment record (commit `c16419e`). Reference for the Phase 5 exemption this stone dissolves; reference for the dispatch_keyword_head split sonnet did (Value-typed dispatch table is mostly unchanged; only the Environment side flips).

4. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.j.md`** — context on Phase 5's emergence + recovery plan.

5. **`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15** — substrate-as-teacher. Short BRIEF; cargo enumerates; iterate.

## Implementation surface

- **Environment.bindings type flip** (2 instances: Environment + EnvironmentBuilder)
- **Environment.lookup signature flip** (+ 6 callers — most already do `.into_tracked()` post-lookup, that call becomes redundant once lookup returns TrackedValue directly)
- **EnvironmentBuilder.bind signature flip** (accept TrackedValue)
- **bind_let_binding simplification** (remove re-wrap; bind TrackedValue directly; destructure wraps each element with `TrackedValue::from`)
- **Value::Tracked variant DELETE**
- **Value::inner() DELETE**
- **Value::provenance() DELETE**
- **Value::into_tracked() DELETE**
- **Dead match arms cleanup** (Hash, type_name, render_value, value_to_edn_with, closure_extract, any Eq/PartialEq)
- **.inner() call-site sweep** (~19 sites; strip or replace inline)
- **.into_tracked() call-site sweep** (~26 sites; replace with TrackedValue::from)
- **DELETE tests/probe_value_tracked_transparency.rs** (probes retired variant)
- **Remove probe-3-exempt mechanism** in probe_stone_233_2_j (exemption expired)

## What does NOT change

- **TrackedValue struct** + its `new`/`from`/`value`/`value_owned`/`provenance` methods — KEEPS unchanged
- **ValueSnapshot::of_tracked** — KEEPS unchanged
- **ValueSnapshot::of(&Value)** — KEEPS unchanged (continues to return Provenance::Unknown for bare Value)
- **eval boundary** — unchanged (already a direct passthrough post-233.2.j)
- **eval_inner signature** — unchanged (`Result<TrackedValue, _>`)
- **5 producer signatures** — unchanged (already `Result<TrackedValue, _>` post-233.2.j)
- **dispatch_keyword_head + dispatch_keyword_head_value split** — unchanged
- **runtime_def_values HashMap<String, Value>** at line 1494 — stays as Value (different concern; defmacro-level, not provenance-carrying)
- **holon-rs** — NOT touched

## Out of scope (affirmative scope-bounding)

- **Stone 233.2.l #[wat_value] proc-macro structural seal** — sub-DESIGN at `57eced2`; lands after 233.2.k
- **Stone 233.2.e AST-derived provenance** for destructure slots / recv / try-recv — separate stone; doesn't gate 233.2.k
- **runtime_def_values storage** — separate concern; not provenance-carrying in let-binding sense
- **holon-rs** — STOP-4
- **HARD CUT** — no deprecation alias for any deleted helper or variant

## Verification flow

```bash
cargo test --release --test probe_stone_233_2_k_variant_retired 2>&1 | tail -5    # 5/5 PASS post-stone
cargo build --release -p wat 2>&1 | tail -5                                       # 0 errors
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                   # ≥ 827 passed; 0 failed
cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 | tail -3 # 5/5 PASS (exemption now removed)
cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 | tail -3 # 3/3 PASS
cargo test --release --test probe_tracked_value_mint_contract 2>&1 | tail -3      # 6/6 PASS
cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 | tail -3 # 1/1 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3 # 8/8 PASS (let-binding probes 6/7/8 stay green via Option A)
cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 | tail -3 # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"       # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                           # empty
ls tests/probe_value_tracked_transparency.rs 2>&1 | grep -c "No such"             # 1 (file DELETED)
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors NOT tracing to the cascade
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **180 min elapsed** (per sub-DESIGN calibration: 60-120 Mode A; 180 STOP)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 54
- **STOP-6:** scope creep — touching 233.2.l proc-macro, 233.2.e AST-derived provenance, runtime_def_values storage
- **STOP-7:** probe still has failures post-stone (any of 5 contracts not PASS)
- **STOP-8:** existing arc 233 probes regress (especially 233.1 probes 6/7/8 — load-bearing for Option A correctness)
- **STOP-9:** cascade exceeds time-box — surface partial state per `feedback_partial_state_grading` (do NOT bridge / workaround)

Per FM 2-bis: STOP triggers are REJECTION criteria; never permission-to-defer.

## Trap-door audit

- **Value::Tracked variant DELETE is structural** — once gone, the pattern-match trap-door class has no living instance.
- **bind_let_binding re-wrap dissolves PERMANENTLY** via Environment TrackedValue storage. No #[ignore] markers, no deferral, no convention enforcement.
- **HARD CUT discipline**: `tests/probe_value_tracked_transparency.rs` (from 233.2.a) tests retired surface — DELETED, not refactored.
- **.inner() call sites** — most are `match v.inner() { Value::X(...) => ... }`. Post-stone, `match v { Value::X(...) => ... }` works equivalently because Value is never Tracked-wrapped. Strip `.inner()` from these.
- **.into_tracked() call sites** — `something.into_tracked()` becomes `TrackedValue::from(something)`. Mechanical.
- **Watch for runtime_def_values** at line 1494 — if a producer's value ever flows into runtime_def_values (unlikely; this is defmacro-tier), the same provenance-loss issue surfaces. STOP-6 if so — surface to orchestrator; don't bridge.
- **Destructure path** — current behavior wraps each slot with Provenance::Unknown via TrackedValue::from. This is consistent with destructured tuple semantics (slot origin ≠ tuple origin); not a regression.
- **Verify probe_diagnostic_value_snapshot_in_errors probes 6/7/8 stay GREEN** — these test producer provenance survives let-bindings. Pre-stone they pass via Phase 5 re-wrap. Post-stone they MUST pass via Environment storing TrackedValue. If they regress, the Option A fix is incomplete; STOP and surface.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases for any deleted helper or variant
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-233.2.k.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification.
- This is the **STONE WHERE THE CLASS DIES**. After this stone, Value::Tracked does not exist in source. Stone 233.2.l seals the meta-class so future re-introduction compile-errors.
- The probe at `tests/probe_stone_233_2_k_variant_retired.rs` IS the success criterion. Flip all 5 contracts.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.k.md` — sub-DESIGN (commit `f830de8`)
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.l.md` — proc-macro seal that depends on 233.2.k landing (commit `57eced2`)
- `tests/probe_stone_233_2_k_variant_retired.rs` — FM 2-bis probe (commit `f43c577`)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.j.md` — establishes the Phase 5 exemption this stone dissolves
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.j.md` — context on Phase 5 emergence
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15 — substrate-as-teacher
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF
- `scratch/FAILURE-ENGINEERING.md` — the doctrine driving Option A choice
- `feedback_no_known_defect_left_unfixed` — disqualifies Option B's #[ignore] approach
- `feedback_sonnet_writes_substrate` — protocol
- `feedback_partial_state_grading` — discipline if STOP-3 fires
