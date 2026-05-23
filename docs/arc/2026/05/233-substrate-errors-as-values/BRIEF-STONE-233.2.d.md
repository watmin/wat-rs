# BRIEF — Arc 233 Stone 233.2.d — substrate-symmetry uniform `list_span` threading

## What we're doing

Thread `list_span: &Span` uniformly across all dispatch arms in `dispatch_keyword_head` (`src/runtime.rs:4623`) that delegate into the eval layer (`eval_*` fns). The sub-DESIGN locks the canonical signature template. The FM 2-bis probe (commit `2ff3d56`) is the load-bearing assertion: pre-stone FAILS with **133 violations of 382 arms**; post-stone PASSES (0 violations, 6 exempt unchanged, all 376 eval_*-calling arms compliant).

After this stone: substrate-symmetry doctrine holds structurally. Every dispatch arm carries call-site coordinates into the eval layer. Foundation for Stone 233.2.e (AST-derived provenance — `SymbolBound`'s `head_span` needs uniform `list_span` availability).

## Design substrate (READ FIRST; MANDATORY)

1. **`docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.d.md`** — sub-DESIGN. Doctrine, canonical signature template, affirmative scope-bounding, trap-door audit.

2. **`tests/probe_substrate_symmetry_list_span_threading.rs`** (commit `2ff3d56`) — FM 2-bis disconfirming probe. Pre-stone output:

   ```
   substrate-symmetry: 133 of 382 dispatch arms call into eval_* without threading `list_span`.
   Counts: 243 compliant; 6 exempt (no eval_* call); 133 violations.
   ```

   Post-stone target: `test result: ok. 1 passed; 0 failed`. **The probe IS the success criterion** — sonnet's success is "flip this probe from FAIL to PASS."

3. **`docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.c.md`** — the precedent stone. Stone 233.2.c's `eval_edn_read` signature plumb is the canonical one-arm preview. The dispatch arm `":wat::edn::read" => crate::edn_shim::eval_edn_read(args, list_span, env, sym)` is the exact template shape to replicate.

4. **`src/runtime.rs:4623`** — `dispatch_keyword_head` function start; signature already has `list_span: &Span` in scope.

## Canonical signature template (the convention)

Called `eval_<name>` fn:

```rust
fn eval_<name>(
    args: &[WatAST],
    list_span: &Span,    // structural invariant — always threaded
    env: &Environment,
    sym: &SymbolTable,
) -> Result<Value, RuntimeError>
```

Dispatch arm:

```rust
":wat::ns::verb" => eval_<name>(args, list_span, env, sym),
```

**Position convention:** `list_span` is the 2nd parameter (after `args`, before `env`). If an existing compliant arm has `list_span` in a different position (e.g., `eval_apply` passes it 4th), **leave it as-is**; non-canonical position on already-compliant arms is out of scope for this stone (surface as honest delta).

## Implementation surface

1. Run `cargo test --release --test probe_substrate_symmetry_list_span_threading` — confirm 133 violations.
2. Per FM 15 substrate-as-teacher: start `cargo build --release -p wat`; the compiler enumerates failures. For each:
   - Update dispatch arm to pass `list_span` (or `list_span.clone()` if the called fn takes owned `Span`)
   - Update called `eval_*` fn signature to accept `list_span: &Span` at canonical position
   - If the called fn has callers OTHER than `dispatch_keyword_head` (recursive eval, helper invocations), update those callers too — the compiler names them
3. Iterate `cargo build` until clean.
4. Run the probe — verify PASS.
5. Run verification cascade — verify no regression.

**Unused-parameter handling:** Rust does NOT warn on unused fn parameters by default. No `#[allow]` needed; just thread the param and leave it unused if the body doesn't need it yet. Stone 233.2.e populates use sites; this stone is plumbing only.

## Out of scope (affirmative scope-bounding)

- **Eval fn body refactors** — pure signature changes + dispatch arm updates. Bodies stay untouched.
- **Renaming `list_span`** — name settled per Stone 233.2.c precedent. No synonyms (`call_span` / `form_span` / `list_call_span`).
- **The 6 exempt arms** — pure inline arms (e.g., `Ok(Value::Unit)`, `Err(RuntimeError::...)`) that don't dispatch into eval_*. Substrate-symmetry doctrine doesn't apply.
- **The 243 already-compliant arms** — they follow the convention. Don't touch their bodies; don't reposition `list_span` if it's already there in non-canonical position.
- **Non-dispatch routing paths** — special forms with their own routing (not via `dispatch_keyword_head`) are independent.
- **AST-derived provenance** (Stone 233.2.e)
- **Errors-as-EDN** (Stone 233.3)
- **holon-rs** — NOT touched
- **HARD CUT** — no deprecation aliases
- **New behavioral semantics** — pure plumbing only

## Verification flow

```
cargo test --release --test probe_substrate_symmetry_list_span_threading 2>&1 | tail -5   # PASS post-stone
cargo build --release -p wat 2>&1 | tail -5                                                # 0 errors
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3                            # ≥ 827 passed
cargo test --release --test probe_diagnostic_value_snapshot_in_errors 2>&1 | tail -3       # 8/8 PASS
cargo test --release --test probe_value_tracked_transparency 2>&1 | tail -3                # 8/8 PASS
cargo test --release --test probe_diagnostic_dynamic_keyword_invocation 2>&1 | tail -3     # 8/8 PASS
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"                # ≤ 52
git -C /home/watmin/work/holon/holon-rs/ status --short                                     # empty
```

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors (errors NOT tracing to substrate-symmetry plumbing) — surface and STOP
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** 150 min elapsed
- **STOP-4:** holon-rs touched
- **STOP-5:** new clippy warning above 52 baseline
- **STOP-6:** scope creep — eval fn body refactors, parameter renaming, or touching already-compliant arms
- **STOP-7:** substrate-symmetry probe still FAILS
- **STOP-8:** existing arc 233 probes (Stones 233.1 / 233.2.a / 233.2.b / 233.2.c / 232.0a) regress

Per FM 2-bis: STOP triggers are REJECTION criteria; **never permission-to-defer slots**. If any STOP fires: ship NOTHING beyond the clean-stoppable state; surface as honest delta in SCORE.

## Trap-door audit

- **NO scope expansion.** Plumbing only.
- **NO renaming.** `list_span` is settled.
- **NO touching the 6 exempt arms.** They're correct as-is.
- **NO touching the 243 already-compliant arms** beyond what the compile errors force (recursive ripple).
- **DO NOT touch holon-rs.** STOP-4.
- **Substrate-as-teacher loop.** Sonnet's worklist is the compiler's error output. Don't enumerate 133 arms upfront; let cargo enumerate.
- **Ripple count is data.** If updating an eval_* signature ripples to N non-dispatch callers, log N in SCORE per fn; the ripple count is honest information about the call graph.

### Specific trap from pre-spawn audit

The dispatch table contains arms like `:wat::core::HashMap::*`, `:wat::time::*`, `:wat::io::*`, `:wat::kernel::*`, `:wat::edn::write*`, `:wat::core::map/foldl/foldr/filter/sort-by`, `:wat::holon::Hologram/*` — most violation arms are in these clusters. Verify the probe FAILS in a fresh shell BEFORE starting (probe runs in 0.02s; cheap cross-check).

## Scope reminders

- Mode `model: "sonnet"` (orchestrator sets explicitly per FM 12)
- HARD CUT — no aliases
- Per `feedback_inscription_immutable`: SCORE is a new file
- Per `feedback_no_broken_commits`: do NOT commit. Orchestrator commits after independent verification

## Cross-references

- `docs/arc/2026/05/233-substrate-errors-as-values/DESIGN-STONE-233.2.d.md` — sub-DESIGN
- `docs/arc/2026/05/233-substrate-errors-as-values/SCORE-STONE-233.2.c.md` — `eval_edn_read` precedent
- `tests/probe_substrate_symmetry_list_span_threading.rs` — FM 2-bis probe (commit `2ff3d56`)
- `docs/arc/2026/04/109-kill-std/INVENTORY.md` § P — original gap-surfacing
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15 — substrate-as-teacher
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — probe-before-BRIEF
- `feedback_sonnet_writes_substrate` — protocol
- `feedback_inscription_immutable` — SCORE is new file
- `feedback_no_broken_commits` — no commit by sonnet
