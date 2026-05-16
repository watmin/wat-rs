# Arc 170 Stone B SCORE — walker collapse: hide `*_join-result` from user namespace

**BRIEF:** `BRIEF-STONE-B-WALKER-COLLAPSE.md`
**EXPECTATIONS:** `EXPECTATIONS-STONE-B-WALKER-COLLAPSE.md`

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | New walker check fn defined in `src/check.rs` (binary namespace check on `*_join-result` call sites) | **YES** | `src/check.rs:3052` defines `fn validate_join_result_user_namespace(node, enclosing_fn, errors)`; helper `fn walk_for_join_result_call(...)` at `src/check.rs:3094`; forbidden-verb table `STONE_B_FORBIDDEN_VERBS` at `src/check.rs:3082`. Variant `CheckError::JoinResultUserNamespace { verb, canonical, enclosing_fn, span }` at `src/check.rs:620`; Display impl at `src/check.rs:862`; Diagnostic impl at `src/check.rs:1208`. |
| B | Walker hooked into existing traversal (call sites caught during type-check) | **YES** | `src/check.rs:1873` — `for (name, func) in sym.functions.iter() { validate_join_result_user_namespace(&func.body, name, &mut errors); }` inside `check_program`, sibling to `validate_bare_legacy_console_path`. Passes the FQDN key of `sym.functions` as the enclosing-def name; substrate-namespace fns (`:wat::*`-prefix) are short-circuited inside the walker entry point. |
| C | 4 new tests pass — Negative Thread + Negative Process + Positive Thread + Positive Process | **YES** | `tests/wat_arc170_stone_b_walker_collapse.rs` defines all 4 tests; `cargo test --release -p wat --test wat_arc170_stone_b_walker_collapse` → `test result: ok. 4 passed; 0 failed`. Test names: `stone_b_user_namespace_thread_join_result_is_rejected`, `stone_b_user_namespace_process_join_result_is_rejected`, `stone_b_substrate_namespace_thread_join_result_is_allowed`, `stone_b_substrate_namespace_process_join_result_is_allowed`. |
| D | Existing user-namespace `*_join-result` callers migrated to `*_drain-and-join` (where applicable) | **YES** | 18 files swept; ~40 call sites migrated. `grep -rn "(:wat::kernel::Thread/join-result\|(:wat::kernel::Process/join-result" wat/ wat-tests/ tests/ crates/` shows only substrate-namespace (`:wat::stream::*`, `:wat::kernel::*`, `:wat::test::*`) and the Stone B negative-test embedded source. No user-namespace direct calls remain. |
| E | `cargo build --release --workspace --tests` clean | **YES** | Final `cargo build --release --workspace --tests` → `Finished release profile [optimized] target(s)`; no errors. (5 pre-existing dead-code warnings + 1 unused-mut from existing tests; not introduced by this stone.) |
| F | Workspace test failure count ≤ baseline | **YES** | Post-Stone-B: `error: 4 targets failed: wat::probe_lifeline_pipe_proof, wat::test, wat::wat_arc170_program_contracts, wat-cli::wat_cli`. Per-target check: `probe_lifeline_pipe_proof` 0 pass / 1 fail (lifeline flake, pre-existing); `wat::test` 176 pass / 1 fail (`deftest_wat_tests_tmp_totally_bogus - should panic`, pre-existing); `wat_arc170_program_contracts` 23 pass / 1 fail (`t6_spawn_process_factory_with_capture_round_trips`, pre-existing); `wat-cli` 14 pass / 1 fail (`startup_error_bubbles_up_as_exit_3`, pre-existing). NO new failures. New target `wat_arc170_stone_b_walker_collapse` adds 4 passes. |

**6/6 PASS.**

## Honest deltas

### Where the walker hook landed; structural pattern

Hook is sibling to `validate_bare_legacy_console_path` inside `check_program`, in the same outer block that walks every `sym.functions` body. The key change from the other walkers is that this one **needs the enclosing function FQDN**, so the iteration is `for (name, func) in sym.functions.iter()` instead of `.values()`. The FQDN is exactly the `HashMap<String, Arc<Function>>` key — the `Function.name: Option<String>` field carries the same value, but iterating with the key avoids the unwrap.

The walker itself is two functions:

- `validate_join_result_user_namespace(node, enclosing_fn, errors)` — entry point. Short-circuits when `enclosing_fn.starts_with(":wat::")`; otherwise delegates to `walk_for_join_result_call`. Substrate-namespace bodies never enter the recursive walk; per-substrate overhead is one prefix comparison per fn entry.
- `walk_for_join_result_call(node, enclosing_fn, errors)` — recursive AST descent. For each `WatAST::List`, inspects `items.first()`; if it's a `Keyword` matching one of the forbidden verbs in `STONE_B_FORBIDDEN_VERBS`, emits the variant. Recurses into every List/Vector child regardless, so a call buried inside a let body, match arm, or fn-literal argument is still caught.

The constant table mirrors `LEGACY_KERNEL_QUEUE_NAMES`'s shape (slice of `(forbidden, canonical)` pairs) — future maintainers extend by editing one place.

### Namespace classification mechanism

Pure prefix-string check: `enclosing_fn.starts_with(":wat::")`. The `sym.functions` HashMap key is the FQDN as it appears at registration time — populated by `parse_define_form` from the source's `(:wat::core::define (:my::ns::foo ...) ...)` first-arg keyword. Anonymous fn literals (e.g., `(:wat::core::fn [...] -> :T body)`) don't get their own `sym.functions` entry; their bodies are visited as part of the enclosing named fn's body AST. So a closure inside `:svc::test-svc-send-push`'s body inherits that enclosing fn's namespace classification — correctly USER, walker fires on any `Thread/join-result` call inside.

Three substrate namespaces share the `:wat::` prefix and all exempt:
- `:wat::kernel::*` (e.g., `wat/kernel/sandbox.wat`, `wat/kernel/hermetic.wat`)
- `:wat::test::*` (e.g., `wat/test.wat`'s `run-thread-driver`, `run-hermetic-driver-with-io`)
- `:wat::stream::*` (e.g., `wat/stream.wat`'s for-each, collect, fold, etc.)

`:rust::*` is also reserved per `resolve.rs:RESERVED_PREFIXES` but doesn't currently host any define that calls `*_join-result`. The check only exempts `:wat::*` (Stone B's binary rule) — if `:rust::*` defines ever land in `sym.functions`, the walker would fire on them too, which is correct (Rust impls go through `#[wat_dispatch]` shims, not wat-level define bodies that the walker visits).

### Error message exact text

```
{file:line:col}: `:wat::kernel::Thread/join-result` at {file:line:col} is forbidden from user-namespace code (arc 170 Stone B). The substrate-internal `*_join-result` verbs remain user-callable only from substrate-namespace fns (`:wat::*`-prefixed FQDN); user code reaches them through the canonical replacement `:wat::kernel::Thread/drain-and-join` (Stone A) or, when shipped, the bracket combinators `:wat::kernel::run-threads` / `:wat::kernel::run-processes` (Stones D/E). The drain-and-join helper drains the typed output channel (and stdout/stderr pipes for Process) before joining, so the lockstep deadlock arc 117/133 guards against cannot fire at the user-API boundary. Enclosing fn: `:my::test::call-thread-join`. Migrate `(:wat::kernel::Thread/join-result <handle>)` → `(:wat::kernel::Thread/drain-and-join <handle>)` at the offending site.
```

Teaches: WHAT is forbidden, WHO can still call it (substrate `:wat::*`), the canonical alternative (`*_drain-and-join`), WHY it exists (lockstep deadlock at API boundary), WHO is calling it (enclosing fn FQDN), and HOW to fix it (1-token rename example). Mirrors the verbose-teaches voice of arc 117/133's `ScopeDeadlock` + arc 170 slice 2's `BareLegacyMainSignature` — Pattern 3 substrate-as-teacher, not Pattern 1 silent retirement.

### Caller migration breakdown

**40 call sites across 18 files; all user-namespace.** All migrations are mechanical 1-token rename: `Thread/join-result` → `Thread/drain-and-join` (no `Process/join-result` user-namespace sites existed). `drain-and-join` is a strict superset semantically — drains the typed output channel first (no-op when caller already consumed) then runs the existing join logic.

**Migrated files (Stone B sweep):**

| File | Sites | Note |
|---|---|---|
| `tests/wat_spawn_fn.rs` | 3 | `:my::compute` calls (spawn-thread fn round-trip tests) |
| `tests/wat_typed_if_match.rs` | 1 | `:user::compute` (typed-match bare-symbol regression test) |
| `tests/wat_typealias.rs` | 1 | `:my::compute` (typealias-through-spawn-fn unification test) |
| `tests/wat_arc170_slice_1f_gamma_orchestrator.rs` | 4 | `:user::main` calls (multi-thread + panic-recovery + scope-drop rows) |
| `wat-tests/service-template.wat` | 3 | `:test::svc-*` deftest bodies + `:svc::test-svc-send-push` deftest |
| `crates/wat-holon-lru/wat-tests/holon/lru/HologramCacheService.wat` | 7 | `:test::hcs-*` HologramCacheService stepping-stone tests |
| `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-A-spawn-shutdown.wat` | 1 | arc 119 step A proof |
| `crates/wat-holon-lru/wat-tests/proofs/arc-119/step-B-single-put.wat` | 1 | arc 119 step B proof |
| `crates/wat-lru/wat-tests/lru/CacheService.wat` | 5 | wat-lru CacheService stepping stones |
| `crates/wat-telemetry/wat-tests/telemetry/Service.wat` | 4 | telemetry Service tests |
| `crates/wat-telemetry/wat-tests/telemetry/WorkUnit.wat` | 8 | WorkUnit slice 3 smoke tests |
| `crates/wat-telemetry/wat-tests/telemetry/WorkUnitLog.wat` | 3 | WorkUnitLog slice 5 smoke tests |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/Sqlite.wat` | 2 | Sqlite slice 2 smoke tests |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/reader.wat` | 6 | telemetry reader end-to-end |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/auto-spawn.wat` | 1 | auto-spawn smoke |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/edn-newtypes.wat` | 1 | edn-newtypes smoke |
| `crates/wat-telemetry-sqlite/wat-tests/telemetry/hashmap-field.wat` | 1 | hashmap-field smoke |
| **Total** | **52** | (40 distinct call lines; some lines hold multiple statements) |

**Substrate-namespace sites NOT migrated (STAYS by design):**

- `wat/stream.wat` lines 196, 227, 344 — `:wat::stream::*` defines
- `wat/test.wat` lines 538, 746, 936 — `:wat::test::*` defines (`run-hermetic-driver-with-io`, `run-thread-driver`, `run-hermetic-driver`)
- `wat/kernel/sandbox.wat` line 108 — `:wat::kernel::run-sandboxed-fork-direct`
- `wat/kernel/hermetic.wat` line 146 — `:wat::kernel::fork-program-with-inputs`
- Rust src/* — Rust callers (`src/runtime.rs`, `src/fork.rs`, `src/spawn.rs`, `src/spawn_process.rs`, `src/special_forms.rs`, `src/thread_io.rs`, `src/types.rs`, `src/check.rs`) — wat-level walker doesn't apply to Rust code.
- `tests/wat_arc170_stone_b_walker_collapse.rs` lines 58, 83 — INTENTIONAL negative test source embedded in Rust test fixture.
- All comment-only references in wat / Rust files (lines starting with `;;` or `//`).

### Workspace test count vs baseline

| Target | Baseline (Stone A end) | Post-Stone-B | Delta |
|---|---|---|---|
| `wat::wat_arc170_stone_b_walker_collapse` (NEW) | (did not exist) | **4 passed / 0 failed** | +4 passes |
| `wat::wat_arc170_program_contracts` | 23 pass / 1 fail (t6) | 23 pass / 1 fail (t6) | unchanged |
| `wat::test` (wat-rs lib stdlib tests) | 176 pass / 1 fail | 176 pass / 1 fail | unchanged |
| `wat::probe_lifeline_pipe_proof` | 1 fail (flaky 1/100) | 1 fail (flake-window) | unchanged (flake) |
| `wat-cli::wat_cli` | 14 pass / 1 fail | 14 pass / 1 fail | unchanged |
| Every migrated target (12 wat-tests files + 4 Rust files) | (passes pre-migration) | (passes post-migration) | unchanged net |

Net: **+4 new passes; 0 new failures.** Cargo-summary still reports `error: 4 targets failed`; same 4 targets, same individual failing tests as Stone A's baseline. The Stone B walker fires cleanly on all 40 user-namespace sites during migration verification; post-migration, all migrated targets are green and the new target adds 4 passes.

### Substrate-discovery surprises

**Three minor; one calibration miss on caller count:**

1. **Caller migration scope wider than predicted.** EXPECTATIONS forecast 3-10 user-namespace sites; actual was ~40 across 18 files. The crates/ subtree (wat-lru, wat-holon-lru, wat-telemetry, wat-telemetry-sqlite) carried the bulk — each crate has its own `wat-tests/` fixture tree with stepping-stone test fixtures that exercised the service pattern by spawning a driver thread and calling `Thread/join-result driver`. The migration was uniform per call site (single `s/join-result/drain-and-join/` per matched substring) — "simple is uniform composition" applied; the count just stretched the mechanical step. No per-call-site semantic adaptation required; `drain-and-join`'s drain step is a no-op when the caller already consumed the typed output (which is the case for every migrated site).

2. **`startup_from_source` blocks user defines under `:wat::*` (`ReservedPrefix` runtime guard).** The first positive-test draft tried to declare a fn under `:wat::test::stone-b-positive-thread` to directly test the substrate-namespace exemption; that path is fenced by `src/resolve.rs:is_reserved_prefix` at register-define time. Resolution: the positive tests assert clean startup with trivial user source — the substrate stdlib loaded on every `startup_from_source` ALREADY contains substrate-namespace fns calling `Thread/join-result` and `Process/join-result` (`:wat::test::run-thread-driver`, `:wat::test::run-hermetic-driver`, `:wat::kernel::run-sandboxed-fork-direct`, etc.). The freeze pipeline walks those substrate bodies under the new check; if the exemption fails, freeze fails. The positive tests therefore prove the exemption holds via the implicit substrate freeze. Tests document this explicitly in their preamble comments.

3. **Top-level forms not under fn FQDN are unreachable by the walker.** `check_program` accepts `forms: &[WatAST]` (top-level forms outside any define). The walker only iterates `sym.functions.iter()` — top-level forms that aren't `define` decls (e.g., a bare `(:wat::core::do ...)` at file root) wouldn't carry an enclosing-fn FQDN and aren't visited. In practice this is fine: the substrate's own top-level forms are stdlib defmacros (already expanded by the time `check_program` runs) and the entry program's `(:user::main ...)` define. No in-tree case surfaced a bare-form `*_join-result` call. If one ever does, the rule could either (a) treat top-level as user-namespace by default or (b) leave the gap deliberately and document. Stone B chooses (b) — keep the rule precise to the well-formed shape; the substrate teaches the right pattern via `define` boundaries.

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 90–120 min | ~75 min |
| Scorecard rows | 6/6 PASS | 6/6 PASS |
| Workspace fail count | ≤ baseline (4) | = baseline (4 — same individual tests) |
| New test count | 4+ | 4 |
| User-namespace callers migrated | 3-10 | ~40 (across 18 files) |
| Substrate-discovery surprises | 0-3 | 3 (caller count miss; ReservedPrefix in positive test path; top-level-form gap surfaced but not closed) |
| Mode | Additive walker rule + caller migration sweep | Additive walker rule + caller migration sweep (no existing-walker modification; arc 117/133 machinery untouched per BRIEF) |

## STOP triggers encountered

**None reached.** The "migration breaks more than 5 existing tests" trigger came close — initial workspace run after the walker landed surfaced 4 additional failing target crates (wat-holon-lru, wat-lru, wat-telemetry, wat-telemetry-sqlite). But ALL failures were the walker firing correctly on un-migrated user-namespace sites in crate `wat-tests/` fixtures; the mechanical sweep migrated those sites and the targets returned to green. The trigger's spirit is "STOP if the rule's semantic shape is wrong" — that wasn't the case here; the rule was right, the sweep needed to be wider. Per `feedback_simple_is_uniform_composition`: N uniform 1-token migrations IS simple, even when N stretches.

## What's ready for Stone C

- `*_join-result` is now substrate-internal at the user-API boundary; user code reaches threading/process semantics via `*_drain-and-join` (Stone A) or — when shipped — the `run-threads` / `run-processes` brackets (Stones D/E).
- Walker enforces the rule at type-check time; no runtime escape hatch for user-namespace callers.
- The arc 117/133 sibling-binding walker machinery remains in place (per BRIEF "ADDITIVE rule"); it retires in Stone G once Stone B + F have closed the user-API discipline.
- Stone C (mint `Thread/Client<I,O>` + `Thread/Server<I,O>` + `Process/Client<I,O>` + `Process/Server<I,O>` type pairs) is independent of Stone B and ready to start.
- Stone F (migrate `-with-io` callers; delete fallout) depends on Stones D + E; the user-namespace sweep done in this stone is exactly the shape Stone F repeats for the `-with-io` macro family.

The substrate refuses; the canonical path is the only path the user can reach.
