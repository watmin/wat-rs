# BRIEF — Arc 233 Stone 233.2.j — migrate 5 producers + eval_inner TrackedValue cascade

## What we're doing

Flip `eval_inner` to return `Result<TrackedValue, RuntimeError>` and migrate the 5 producers to construct `TrackedValue::new(value, provenance)` directly (no more `Value::Tracked` wrapping at the producer site). This makes `Value::Tracked` UNREACHABLE at every constructor site, enabling Stone 233.2.k to delete the variant and Stone 233.2.l to seal the meta-class via proc-macro.

**The cascade:**

1. **eval_inner signature flip** — `src/runtime.rs:4514`:
   ```rust
   pub(crate) fn eval_inner(...) -> Result<TrackedValue, RuntimeError>
   ```
   ~30 internal `Ok(Value::...)` arms inside eval_inner wrap with `.into_tracked()` (use existing `TrackedValue::from(Value)` helper if `into_tracked()` doesn't exist yet — mint it as a one-liner on `impl Value`).

2. **5 producer constructor swaps** (Value::Tracked → TrackedValue::new):
   - `src/runtime.rs:7371` — `eval_keyword_from_string`
   - `src/runtime.rs:14420, 14428, 14438, 14449, 14456, 14463, 14470, 14477, 14484` — `eval_from_holon` (9 primitive arms)
   - `src/runtime.rs:14544, 14559, 14600, 14616, 14658` — `eval_from_holon` (5 nested classifier-wrap arms)
   - `src/runtime.rs:19788, 19865` — `eval_kernel_recv` + `eval_kernel_try_recv` (special — see below)
   - `src/edn_shim.rs:227` — `eval_edn_read`

3. **383 eval_inner caller sweep** in src/runtime.rs — `let v = eval_inner(...)?` becomes `let v = eval_inner(...)?.value_owned()` (or `.value()` if borrow is sufficient). Substrate-as-teacher per FM 15; cargo enumerates batches; iterate until 0 errors.

4. **eval boundary simplification** — `src/runtime.rs:4629`:
   ```rust
   pub fn eval(...) -> Result<TrackedValue, RuntimeError> {
       eval_inner(ast, env, sym)  // direct passthrough; no unwrap-and-rewrap
   }
   ```
   The 4-line match-and-rewrap at lines 4636-4639 is removed entirely.

5. **`ValueSnapshot::of_tracked(&TrackedValue) -> Self`** — new constructor in `impl ValueSnapshot` (sibling to existing `of`, `unavailable`, `described`). Reads `tv.value()` for type_name + render; reads `tv.provenance()` for provenance field. Existing `of(&Value)` stays + keeps `Provenance::Unknown` per Stone 233.2.a contract.

**Special case — recv/try-recv:** the producer wrap at `runtime.rs:19788` + `19865` is **inside** a nested `Value::Result(Arc::new(Ok(Value::Option(Arc::new(Some(tagged))))))` chain. The `tagged` slot needs Value (not TrackedValue) because the outer Option<T> carries Value. **Planned honest delta:** at this stone, recv/try-recv lose producer provenance because the wrap can't be TrackedValue (the surrounding Value::Option is structurally Value-typed). Document in SCORE; arc 233 Stone 233.2.e revisits via AST-derived provenance mechanism on the receive side.

For recv/try-recv specifically, the construction becomes:
```rust
// Before
let tagged = Value::Tracked { inner: Box::new(v), provenance: ... };
Ok(Value::Result(Arc::new(Ok(Value::Option(Arc::new(Some(tagged)))))))

// After
// Provenance lost at the value carrier (revisited in Stone 233.2.e).
// The Value::Tracked wrap is REMOVED entirely; bare v flows through.
Ok(Value::Result(Arc::new(Ok(Value::Option(Arc::new(Some(v)))))))
```

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.j.md`** (commit `064df14`) — sub-DESIGN; substrate-informed migration plan; four-questions verdict; planned honest delta at recv/try-recv. **Authoritative for shape decisions.**

2. **`tests/probe_stone_233_2_j_producer_migration.rs`** (commit `cf6d464`) — FM 2-bis probe. 5 contracts. **The probe IS the success criterion** — sonnet flips:
   - Probe 1 + 2 (behavioral): already pass; stay green
   - Probe 3 (static scan): 18 → 0 construction sites
   - Probe 4 (of_tracked): doesn't compile → exists + reads provenance
   - Probe 5 (eval simplification): unwrap arm present → arm removed

3. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.i.md`** — boundary flip precedent (commit `8164629`). The pattern sonnet now extends one layer deeper (eval_inner instead of eval).

4. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.h.md`** — TrackedValue mint precedent (commit `38acd60`). The TrackedValue API (`new`, `from`, `value`, `value_owned`, `provenance`) is the only surface used.

5. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.d.md`** — substrate-as-teacher cascade precedent. Same iteration shape per FM 15.

6. **`docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15** — substrate-as-teacher pattern. **Short BRIEF; sonnet iterates from compile errors.** Don't enumerate the 383 sites upfront; let cargo enumerate by failing.

## Implementation surface

1. **`Value::into_tracked()` helper** — confirm `TrackedValue::from(Value)` exists post-233.2.h; if it does, use it directly. If not, mint `impl Value { pub fn into_tracked(self) -> TrackedValue { TrackedValue::from(self) } }` as a one-line convenience.

2. **eval_inner signature** — flip to `Result<TrackedValue, RuntimeError>`. Wrap all leaf `Ok(Value::...)` arms (literals, etc.) with `.into_tracked()` or `TrackedValue::from(...)`.

3. **eval_inner cascade** — substrate-as-teacher iteration. `let v = eval_inner(...)?` becomes `let v = eval_inner(...)?.value_owned()`. Use `.value()` (borrow) where the caller doesn't need to consume.

4. **5 producer constructors** — replace each `Value::Tracked { inner: Box::new(v), provenance: p }` with `TrackedValue::new(v, p)` (use the values from existing call sites; don't change provenance content). For `eval_from_holon`'s 14 arms, mechanical sweep. For `recv`/`try-recv`, drop the wrap entirely (planned honest delta).

5. **eval boundary simplification** — `pub fn eval` becomes a 1-line passthrough. The freeze.rs surfaces (`eval_in_frozen`, `eval_digest_in_frozen`, `eval_signed_in_frozen`) likely need NO change if they delegate to `eval` — verify and adjust.

6. **`ValueSnapshot::of_tracked`** — mint in `impl ValueSnapshot` block. Existing `of(&Value)` stays. Internal sites that have TrackedValue migrate to of_tracked incrementally; OUT OF SCOPE for this stone — only ADD the constructor.

## What does NOT change

- **`Value::Tracked` variant body** — STAYS in this stone. The variant + `.inner()` + `.provenance()` helpers + Hash/Eq/Display unreachable match arms remain until Stone 233.2.k retires them.
- **5 producer return TYPES** at the fn signature level — they continue to return `Result<Value, RuntimeError>` (the wrap site changes, not the signature). Note: eval_inner returns TrackedValue, so internal eval_X functions called BY eval_inner stay Value-typed; the cascade is ONLY at eval_inner's signature + its leaf-arm wraps.
- **External RAISE sites that currently use `ValueSnapshot::of(&Value)`** — stay as-is. Migration to of_tracked is out of scope.

## Out of scope (affirmative scope-bounding)

- **Value::Tracked variant retirement** — Stone 233.2.k.
- **#[wat_value] proc-macro structural seal** — Stone 233.2.l.
- **AST-derived provenance on let-bindings + literals** — Stone 233.2.e.
- **Migrating ALL ValueSnapshot::of sites to of_tracked** — incremental work; this stone only ADDS the constructor.
- **Internal eval_<name> signature flips** — they stay returning Value; ONLY eval_inner flips.
- **holon-rs** — NOT touched.
- **HARD CUT** — no parallel API; no deprecation aliases.

## Verification flow

```bash
cargo test --release --test probe_stone_233_2_j_producer_migration 2>&1 | tail -5    # 5/5 PASS post-stone
cargo build --release -p wat 2>&1 | tail -5                                          # 0 errors
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                      # ≥ 827 passed; 0 failed
cargo test --release --test probe_eval_signature_returns_tracked_value 2>&1 | tail -3 # 3/3 PASS (regression guard)
cargo test --release --test probe_tracked_value_mint_contract 2>&1 | tail -3          # 6/6 PASS
cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 | tail -3 # 1/1 PASS
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3  # 8/8 PASS
cargo test --release --test probe_value_tracked_transparency 2>&1 | tail -3           # 8/8 PASS
cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 | tail -3 # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"           # ≤ 54
git -C /home/watmin/work/holon/holon-rs/ status --short                               # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors NOT tracing to the cascade
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** **240 min elapsed** (per Stone 233.2.j sub-DESIGN calibration: 90-150 Mode A; 240 STOP)
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 54
- **STOP-6:** scope creep — touching Value::Tracked variant body (still exists per 233.2.k scope), proc-macro work (233.2.l), or migrating ALL ValueSnapshot::of sites to of_tracked
- **STOP-7:** probe still has failures post-stone (any of 5 contracts not PASS)
- **STOP-8:** existing arc 233 probes regress
- **STOP-9:** cascade exceeds time-box — surface partial state for orchestrator (do NOT bridge / workaround)

Per FM 2-bis: STOP triggers are REJECTION criteria; never permission-to-defer.

## Trap-door audit

- **NO new parallel API.** Use existing TrackedValue surface (`new`, `from`, `value`, `value_owned`, `provenance`).
- **NO retirement of Value::Tracked variant.** Stone 233.2.k owns. Match arms in Hash/Eq/Display impls stay unreachable.
- **NO of_tracked sweep at RAISE sites.** Only ADD the constructor; migration sites are incremental.
- **recv/try-recv provenance loss is INTENTIONAL** for this stone. Document in SCORE; arc 233 Stone 233.2.e revisits.
- **Internal eval_<name> fns stay returning Value.** Only eval_inner flips signature.
- **The cascade is substrate-as-teacher (FM 15).** Don't enumerate 383 sites upfront; let cargo enumerate by failing. Iterate one error batch per round.
- **`.value()` vs `.value_owned()`** — pick per call site. `.value()` borrows for pattern-match; `.value_owned()` consumes for ownership transfer. Sonnet's call.
- **Boundary wrap removal in eval()** — verify the `Value::Tracked { inner, provenance }` match arm at line 4636 is removed entirely; eval becomes direct passthrough. Probe 5 asserts.

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases
- Per `feedback_inscription_immutable`: SCORE is a NEW file (`SCORE-STONE-233.2.j.md`)
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification
- This is the **BIG cascade** per Stone 233.2.j sub-DESIGN (3.6× the 233.2.i call-site count, narrower file scope). Expect substantial wall-clock; substrate-as-teacher iteration is the working pattern.
- The probe at `tests/probe_stone_233_2_j_producer_migration.rs` IS the success criterion. Flip all 5 contracts.

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.j.md` — sub-DESIGN (commit `064df14`)
- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.g.md` — Shape A pivot that mandated TrackedValue
- `tests/probe_stone_233_2_j_producer_migration.rs` — FM 2-bis probe (commit `cf6d464`)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.i.md` — boundary flip precedent (commit `8164629`)
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.h.md` — TrackedValue mint precedent (commit `38acd60`)
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15 — substrate-as-teacher
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF
- `scratch/FAILURE-ENGINEERING.md` — annihilation-not-patch doctrine driving this stone
- `feedback_sonnet_writes_substrate` — protocol
