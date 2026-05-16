> **SUPERSEDED 2026-05-16 — decomposed into 4 stones per `feedback_iterative_complexity`.** The original BRIEF below bundled too much (proc-macro + inventory wiring + 2 fn migrations + Stone B rule deletion + 4 test updates) and predicted 180-300 min, which IS the warning signal "this is too big." First sonnet launched on this BRIEF was killed in reading phase. Replaced by 4 stones:
>
> - `BRIEF-STONE-1-INVENTORY-WIRING.md` — substrate-only inventory + `RestrictionEntry` struct + setup iteration; no proc-macro yet
> - `BRIEF-STONE-2-PROC-MACRO-ATTRIBUTE.md` — mint `#[restricted_to(...)]` in wat-macros
> - `BRIEF-STONE-3-APPLY-TO-JOIN-RESULT.md` — annotate the 2 substrate fns; verify both walkers fire
> - `BRIEF-STONE-4-LOOP-CLOSURE.md` — delete Stone B's ad-hoc rule + update Stone B's tests
>
> Each stone ~30-90 min sonnet. Each provable in isolation. Original BRIEF preserved below as historical record.

---

# Arc 198 Slice 2 BRIEF — `#[restricted_to(...)]` proc-macro attribute (Rust-side complement)

**Arc:** 198 (continuation — slice 1 shipped wat-side `def-restricted` + `defn-restricted` at commit `24d3b0d`)
**Task:** #328 (reframed from "future arc" to arc 198 slice 2 per `feedback_stay_in_arc_until_inscribed`)
**Predecessor:** arc 170 Stone B's ad-hoc `validate_join_result_user_namespace` walker rule (commit `2a071f0`)

## Goal

Mint `#[restricted_to(...)]` proc-macro attribute in `crates/wat-macros/` that declares allowed-caller-prefix whitelists on Rust-side substrate primitives. Symmetric to arc 198 slice 1's wat-side `(def-restricted :name [...] value)` form — same `defined_value_restrictions` HashMap, same walker enforcement, same matching rules, different declaration surface (Rust attribute vs wat form).

Apply to `eval_kernel_thread_join_result` + `eval_kernel_process_join_result`. Loop closure: delete arc 170 Stone B's ad-hoc walker rule (`validate_join_result_user_namespace` + `CheckError::JoinResultUserNamespace` + its `check_program` hook). Update Stone B's 4 tests to assert on arc 198's `DefRestrictedCallerNotAllowed` error format.

## Why this exists

The user's framing 2026-05-16: *"we only operate on long term - if we think 5 users may exist that number is small enough to ensure they will."* Anticipated substrate-internal primitives needing restriction: `*_join-result` (2 now), bracket-internals when arc 170 Stones D/E land, Holon LRU service internals, telemetry internals, future ones. 5+ plausible — the proc-macro shape IS the long-term-superior form; building it now per `feedback_stay_in_arc_until_inscribed`.

Inventory-based auto-registration chosen over sibling-const-with-manual-plug-in per four-questions analysis (sibling const fails obvious + honest + good-UX; inventory wins YES YES YES YES — the `inventory` crate is complex underneath but exposes simple surface primitives per `feedback_simple_is_uniform_composition` updated principle).

## Form shape (settled)

```rust
#[restricted_to(":wat::")]
pub(crate) fn eval_kernel_thread_join_result(...) -> Result<Value, RuntimeError> {
    // body unchanged
}

#[restricted_to(":wat::", ":my::specific::fn")]
pub(crate) fn some_other_primitive(...) -> ... { ... }
```

**Prefix matching rules** (must match arc 198 slice 1 wat-side rules):
- Trailing `::` (e.g., `:wat::kernel::`) → namespace prefix match (caller FQDN starts with this)
- No trailing `::` (e.g., `:wat::kernel::specific-fn`) → exact FQDN match
- Empty prefix list `#[restricted_to()]` → no callers allowed (every call fails)

## Architecture

**Inventory-based auto-registration:**

1. `wat-macros` defines `#[restricted_to(prefix1, prefix2, ...)]` proc-macro attribute. Parses variadic string-literal args.
2. The attribute generates:
   - Original fn (unchanged)
   - An `inventory::submit! { RestrictionEntry { wat_name: ..., prefixes: &[...] } }` statement
3. `RestrictionEntry` struct lives in the `wat` crate (alongside CheckEnv) and uses `inventory::collect!` macro.
4. The attribute needs the WAT NAME to populate `wat_name` — but the Rust fn doesn't know its wat name. **Two options for sonnet to decide:**
   - **(a) Attribute carries wat name:** `#[restricted_to(wat_name = ":wat::kernel::Thread/join-result", from = [":wat::"])]` — full self-contained, but redundant if dispatch arm already declares the mapping
   - **(b) Attribute lives WITH the registration call:** different shape entirely — declarative macro at the `env.register` call site that ALSO emits inventory entry; not an attribute on the fn body
   - **Sonnet decides** based on what reads cleanest given the existing dispatch + registration architecture
5. After `env.register` calls run in setup, iterate `inventory::iter::<RestrictionEntry>` and call `env.defined_value_restrictions.insert(name, prefixes)` for each.

## Decay disclosure (orchestrator → sonnet)

Orchestrator has had multiple substrate-fact failures across this session. THIS BRIEF describes the TARGET SHAPE. **Sonnet has FULL AUTHORITY on substrate-internal discovery** — exact attribute syntax design (sub-decision (a) vs (b) above), inventory wiring, RestrictionEntry struct shape, setup iteration landing, codegen details, dependency additions. Do NOT trust orchestrator claims about substrate internals without grep verification.

## Substrate state pointers (verified)

- `crates/wat-macros/src/lib.rs` — proc-macro crate entry; current attribute is `#[wat_dispatch]`
- `crates/wat-macros/src/codegen.rs` — existing codegen patterns
- `crates/wat-macros/Cargo.toml` — add `inventory` dep if needed; `wat-macros` proc-macros generate code that runs in the consumer crate
- `Cargo.toml` (wat crate root) — add `inventory` dep here too (consumer side)
- `src/runtime.rs:16722` — `eval_kernel_thread_join_result` (the fn to annotate)
- `src/runtime.rs:16340` — `eval_kernel_process_join_result` (the fn to annotate)
- `src/check.rs:13170-13183` — Thread/join-result `env.register(...)` site
- `src/check.rs:13032-13046` — Process/join-result `env.register(...)` site
- `src/check.rs:1654` — `CheckEnv.defined_value_restrictions: HashMap<String, Vec<String>>` (arc 198 slice 1 storage)
- `src/check.rs:1806` — insert API
- `src/check.rs:3094` — `validate_join_result_user_namespace` (Stone B's ad-hoc walker — DELETE in this slice)
- `src/check.rs:1939` — Stone B's hook into `check_program` (DELETE)
- `src/check.rs:667+` — `CheckError::JoinResultUserNamespace` variant + Display + Diagnostic (DELETE)
- `src/check.rs:3094+` — arc 198's `DefRestrictedCallerNotAllowed` walker (KEEPS; will now fire for *_join-result too)
- `tests/wat_arc170_stone_b_walker_collapse.rs` — 4 tests that grep "drain-and-join" in error msg; need update to grep arc 198's error format
- `wat/core.wat:202-206` — existing `defn` defmacro (precedent pattern)
- `docs/arc/2026/05/198-defn-restricted/SCORE.md` — arc 198 slice 1's SCORE (read for architecture context)

## Implementation protocol (per `feedback_test_first` + `feedback_iterative_complexity`)

1. **Read substrate state.** All pointers above. Pay special attention to wat-macros codegen patterns + how `#[wat_dispatch]` parses + emits.

2. **Write tests FIRST.** Add to `tests/wat_arc198_slice2_restricted_to.rs`:
   - **Test 1 (negative):** user-namespace fn calls `:wat::kernel::Thread/join-result` → startup fails with `DefRestrictedCallerNotAllowed` error mentioning the restricted callee
   - **Test 2 (negative):** same shape for `Process/join-result`
   - **Test 3 (positive):** `:wat::*` namespace fn calls `Thread/join-result` → startup succeeds
   - **Test 4 (positive):** same for `Process/join-result`
   - **Test 5 (attribute mechanism):** a SECOND test substrate primitive in `tests/` or a probe fixture wrapped with `#[restricted_to(...)]` proves the attribute correctly registers restriction via inventory
   - RUN; CONFIRM tests fail (rule not yet in place via attribute — but Stone B's rule may still fire; that's OK — tests will pass during transition AND after; sonnet manages)

3. **Implement `#[restricted_to(...)]` attribute** in `crates/wat-macros/`:
   - Parser for variadic string args
   - Codegen that emits `inventory::submit!` entry alongside the fn
   - `RestrictionEntry` struct + `inventory::collect!` registration (probably in `wat` crate `src/` near CheckEnv)
   - Cargo dependencies as needed (`inventory` crate likely)

4. **Apply attribute** to `eval_kernel_thread_join_result` + `eval_kernel_process_join_result`. KEEP the existing `env.register(...)` calls; the attribute is ADDITIVE.

5. **Add setup-time iteration** in wherever the substrate registration sequence completes (likely a finalize/freeze step after `register_builtin_types` runs). Pseudo:
   ```rust
   for entry in inventory::iter::<RestrictionEntry> {
       env.defined_value_restrictions.insert(entry.wat_name.into(), entry.prefixes.iter().map(|s| s.to_string()).collect());
   }
   ```

6. **Verify both Stone B's walker and arc 198's walker fire for the *_join-result cases** (both rules catch the same violations during transition). Tests pass.

7. **Delete Stone B's ad-hoc rule** + `CheckError::JoinResultUserNamespace` variant + its hook in `check_program`. NOW only arc 198's walker fires for *_join-result.

8. **Update Stone B's 4 tests** in `tests/wat_arc170_stone_b_walker_collapse.rs` to grep arc 198's `DefRestrictedCallerNotAllowed` error format instead of Stone B's old "drain-and-join" suggestion text.

9. **`cargo build --release --workspace --tests` clean. Workspace test verification** — failure count ≤ baseline (4 pre-existing: lifeline flake, t6 unquote, totally_bogus, startup_error).

10. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/`. Anchor cwd; absolute paths route correctly.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / past STONE BRIEFs/EXPECTATIONS/SCOREs / arc 198 slice 1 BRIEF/EXPECTATIONS/SCORE / this BRIEF / this EXPECTATIONS.
- DO NOT touch arc 198 slice 1's wat-side `def-restricted` / `defn-restricted` forms — those are complete.
- DO NOT modify Stone A's `drain-and-join` helpers (`eval_kernel_*_drain_and_join`).
- DO NOT modify Stone B's user-namespace caller migrations (~40 sites already done).
- DO NOT update USER-GUIDE / docs.
- DO NOT use any path containing `.claude/worktrees/`.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks. NEVER use destructive git commands.

## Scorecard (7 rows YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `#[restricted_to(...)]` proc-macro attribute defined in `crates/wat-macros/` | grep crates/wat-macros/src/ for `restricted_to`; attribute signature accepts variadic string args |
| B | Inventory wiring: `RestrictionEntry` struct + `inventory::collect!` in wat crate + setup iteration | grep `RestrictionEntry\|inventory::iter\|inventory::submit` shows the full pipeline |
| C | Attribute applied to both `eval_kernel_*_join_result` fns | `grep -B 2 "fn eval_kernel_thread_join_result\|fn eval_kernel_process_join_result" src/runtime.rs` shows attribute above each |
| D | Setup-time iteration populates `env.defined_value_restrictions` for both *_join-result | grep shows the iteration; runtime smoke test confirms HashMap is populated post-setup |
| E | Stone B's ad-hoc rule DELETED (function + CheckError variant + hook) | `grep "validate_join_result_user_namespace\|JoinResultUserNamespace" src/check.rs` returns empty |
| F | 5+ tests pass: Stone B's 4 updated tests + at least 1 new attribute-mechanism test | `cargo test --release -p wat --test wat_arc170_stone_b_walker_collapse --test wat_arc198_slice2_restricted_to` all green |
| G | Workspace test failure count ≤ baseline (4 pre-existing: lifeline flake, t6, totally_bogus, startup_error) | full workspace cargo test failures ≤ 4 |

## STOP triggers

- `inventory` crate (or chosen registration mechanism) doesn't compose with the existing substrate freeze flow → STOP and surface alternatives
- Attribute on fn can't easily know the wat name without redundant declaration → STOP; surface; choose between (a) carry name in attribute or (b) move to declarative-macro-at-registration-site shape
- Proc-macro infrastructure in wat-macros doesn't extend cleanly to accept variadic string args → STOP and surface
- Migration breaks more than 5 existing tests → STOP; root-cause
- > 5 unexpected substrate-finding surfaces → STOP; scope may need decomposition

## Workspace baseline (commit `24d3b0d`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 pre-existing target failures unchanged

Post-slice-2 target:
- ≥ baseline + 1+ passes (new attribute-mechanism test)
- = baseline failures (no regressions; Stone B's tests still pass via arc 198's walker now)

## Time-box

3-5 hours predicted (proc-macro design + inventory wiring + 2 fn migrations + Stone B rule deletion + test updates). Hard stop 6 hours.

## On completion

Write `docs/arc/2026/05/198-defn-restricted/SCORE-RUST-ATTRIBUTE.md`:
- 7 rows YES/NO with grep-able evidence
- Honest deltas: attribute syntax chosen (sub-decision (a) name-carrier vs (b) callsite-macro), inventory wiring shape, RestrictionEntry struct definition location, setup iteration landing, Stone B rule deletion confirmation, test update count, workspace test count vs baseline
- Calibration record (predicted vs actual)

Return final summary message: rows passed/failed + sub-decision choice (a vs b) + workspace test count delta + path to SCORE.

You are launching now. T-minus 0.
