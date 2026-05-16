# Arc 198 Slice 2 Stone 1 SCORE — `inventory` wiring + `RestrictionEntry` struct + setup-time iteration

**BRIEF:** `BRIEF-STONE-1-INVENTORY-WIRING.md`
**EXPECTATIONS:** `EXPECTATIONS-STONE-1-INVENTORY-WIRING.md`

## Scorecard

| Row | What | Status | Evidence |
|-----|------|--------|----------|
| A | `inventory` crate added to wat crate `Cargo.toml` | **YES** | `Cargo.toml:65` declares `inventory = "0.3"` in `[dependencies]` (under the existing `wat-edn`/`wat-macros` entries, with a comment block tying it to Stones 1-4). |
| B | `RestrictionEntry` struct defined + `inventory::collect!` registered | **YES** | New file `src/restriction_entry.rs` declares `pub struct RestrictionEntry { pub wat_name: &'static str, pub prefixes: &'static [&'static str] }` plus `inventory::collect!(RestrictionEntry);`. Module wired into `src/lib.rs` (alongside `pub mod resolve;` / `pub mod runtime;`). Public, so callers (and tests) can `inventory::submit!`. |
| C | Setup-time iteration populates `defined_value_restrictions` from `inventory::iter` | **YES** | `src/freeze.rs` adds step **6.8** in `startup_from_forms_post_config` — a `for entry in inventory::iter::<crate::restriction_entry::RestrictionEntry>` loop that inserts each entry into `symbols.defined_value_restrictions`. Positioned AFTER all `register_*` calls (so the map isn't stomped) and BEFORE step 7.5 / step 8 (so `CheckEnv::from_symbols` sees the unified map at check time). |
| D | 1 new test passes — probe `inventory::submit!` lands in `sym.defined_value_restrictions` after setup | **YES** | `tests/wat_arc198_slice2_stone_1_inventory_wiring.rs` declares `inventory::submit! { RestrictionEntry { wat_name: ":arc198::s2::s1::probe::test-fn", prefixes: &[":wat::kernel::"] } }` at module scope; the single test runs `startup_from_source` against a minimal `:user::main` source and asserts `frozen.symbols.defined_value_restrictions.get(":arc198::s2::s1::probe::test-fn") == Some(&vec![":wat::kernel::".to_string()])`. Run: `cargo test --release -p wat --test wat_arc198_slice2_stone_1_inventory_wiring` → `test result: ok. 1 passed; 0 failed`. |
| E | Workspace test failure count ≤ baseline (4 pre-existing) | **YES** | `cargo test --release --workspace --no-fail-fast` → `error: 3 targets failed: -p wat --test test, -p wat --test wat_arc170_program_contracts, -p wat-cli --test wat_cli`. Individual failures: `deftest_wat_tests_tmp_totally_bogus` (pre-existing), `t6_spawn_process_factory_with_capture_round_trips` (pre-existing), `startup_error_bubbles_up_as_exit_3` (pre-existing). The fourth baseline failure (`lifeline_pipe_zero_orphans_across_100_trials`, known flake) passed this run. 3 ≤ 4. **Zero new failures.** |

**5/5 PASS.**

## Honest deltas

### `RestrictionEntry` struct location

Chose **new file `src/restriction_entry.rs`**, public module re-exported from `src/lib.rs`. Considered placing the struct adjacent to `CheckEnv` inside `src/check.rs` (the BRIEF's first option), rejected because:

1. The struct is a substrate-public declaration mechanism — consumers (test code, future Stone-2 proc-macro output) `use wat::restriction_entry::RestrictionEntry`. Placing it in `check` would force a `use wat::check::RestrictionEntry`, which leaks the implementation detail that the walker (currently) lives in `check`.
2. The struct carries no behavior tied to `CheckEnv`; the wiring boundary is `inventory::iter` → `SymbolTable.defined_value_restrictions`, and that runs in `freeze.rs`, not `check.rs`. A standalone module composes more honestly with the actual data flow.
3. Module is intentionally tiny (~75 LOC with extensive docs); the doc comment carries the Stone 2/3/4 trail so future readers find the channel from the struct.

### Iteration landing point chosen — (α) populate `sym` (NOT `env`)

Chose **(α) populate `symbols.defined_value_restrictions`** in `freeze.rs` before `check_program` runs. (β) populating `env` directly was rejected — it would break the "sym is source of truth; env mirrors at `from_symbols`" pattern that arc 198 slice 1 established (and that arc 157's `defined_values` storage established before that). Slice 1's three insertion sites in `runtime.rs` (lines 1720, 2834, 2915) all write to `sym`; the iteration is a sibling Rust-side feed into the same destination, and the existing `CheckEnv::from_symbols` mirror at `src/check.rs:1696` propagates the unified map onward. One source of truth, one mirror clone — no parallel code paths.

Concretely the loop lives at `src/freeze.rs` as step "6.8" (between step 6.7 newtype-method registration and step 6b user-dispatch registration). The exact placement doesn't matter for correctness within steps 6–7.5 — only that it precedes step 8 — but conceptually it sits with the other "drain a registry into `symbols`" steps and outside the user-dispatch / config-flag propagation flow.

### `inventory` crate version chosen

`inventory = "0.3"`. Current stable major; aligns with BRIEF's hint. Resolved (via `Cargo.lock` after build) to whatever `0.3.x` is on crates.io as of build time; no version-specific feature requested.

### Test-first discipline

Per `feedback_test_first`: wrote `tests/wat_arc198_slice2_stone_1_inventory_wiring.rs` BEFORE any substrate change. First compile attempt failed with the two predicted errors:

```
error[E0432]: unresolved import `wat::restriction_entry`
error[E0433]: failed to resolve: use of unresolved module or unlinked crate `inventory`
```

Saw it fail, then added Cargo.toml dep + `restriction_entry.rs` + freeze.rs hook in that order, then watched the test go green. Verbose-and-honest beats clever-and-cute.

### Workspace test count vs baseline

| Target | Baseline (slice 1 end) | Post-Stone-1 | Delta |
|---|---|---|---|
| `wat::wat_arc198_slice2_stone_1_inventory_wiring` (NEW) | (did not exist) | **1 passed / 0 failed** | +1 pass |
| `wat::probe_lifeline_pipe_proof` | 1 fail (flake 1-2/100) | **1 pass this run** (flake didn't trigger) | -1 fail (flake-dependent; may return) |
| `wat::test` | 176 pass / 1 fail (`totally_bogus`) | 176 pass / 1 fail (`totally_bogus`) | unchanged |
| `wat::wat_arc170_program_contracts` | 23 pass / 1 fail (t6) | 23 pass / 1 fail (t6) | unchanged |
| `wat-cli::wat_cli` | 14 pass / 1 fail (`startup_error`) | 14 pass / 1 fail (`startup_error`) | unchanged |
| `wat::wat_arc198_def_restricted` | 5 passed | 5 passed | unchanged |
| Every other target | passes | passes | unchanged |

**Net: +1 new pass; -1 fail this run (flake passed); 0 new failures.** Workspace report: `error: 3 targets failed` (vs baseline's 4) — strictly fewer failures than baseline, with the difference attributable entirely to the known lifeline flake.

### Substrate-discovery surprises

**Zero. Smooth ride.** Predicted 0-2 in EXPECTATIONS; actual: 0. The arc 198 slice 1 substrate (`SymbolTable.defined_value_restrictions` storage + `CheckEnv::from_symbols` mirror + walker reading from `env.defined_value_restrictions`) was designed for exactly this kind of additive feed — Stone 1's iteration is one new line of plumbing landing in a slot that already had the right shape.

The `inventory` crate composed cleanly with the freeze flow. No static-init ordering issues (entries are collected at link time, ready before any function runs). No `'static` lifetime surprises (string literals and slice literals satisfy it naturally). No `Cargo.toml` workspace-level surprises (the wat crate's `[dependencies]` section was the natural home; no workspace-level shared-deps machinery needed). Docs-test on the example in the module's doc comment is `ignored` (it's an `ignore`-marked code block — would require a proc-macro to actually compile).

## Calibration record

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60 min | ~25 min |
| Scorecard rows | 5/5 PASS | 5/5 PASS |
| Workspace fail count | ≤ baseline (4) | 3 (flake passed this run; 3 baseline failures unchanged) |
| New test count | 1 | 1 |
| Iteration landing | (α) sym OR (β) env | (α) sym — chosen for consistency with arc 198 slice 1's "sym is source of truth; env mirrors" pattern |
| `RestrictionEntry` location | adjacent to CheckEnv OR new module | New module `src/restriction_entry.rs` re-exported from `src/lib.rs` — composes more honestly with the actual data flow (struct → inventory → symbol table → check env mirror) |
| Substrate-discovery surprises | 0-2 | 0 |
| Mode | Additive (pure substrate infrastructure) | Additive |

## STOP triggers encountered

**None reached.**

- "`inventory` crate doesn't compose with substrate freeze flow (static-init ordering)" — no issue; entries are collected at link time and `inventory::iter` returns the full registry on every call regardless of init order.
- "Setup iteration landing point ambiguous (5+ candidate hooks; sequence unclear)" — landing was obvious: between step 6.7 (last `register_*` on `symbols`) and step 7.5 (config-flag propagation). Comment block documents the position.
- "New dep causes downstream build failures" — `cargo build --release --workspace --tests` clean.
- "Migration breaks existing tests" — N/A (no migration); pre-existing tests unchanged. Arc 198 slice 1 tests still pass (5/5). Arc 170 Stone B tests still pass.

## What this enables

After Stone 1 ships:

- **Stone 2** mints a `#[restricted_to(...)]` proc-macro attribute. The macro body emits `inventory::submit!` against `wat::restriction_entry::RestrictionEntry`, deriving `wat_name` from the annotated fn's path (or accepting an explicit override) and lifting the attribute's prefix list into a `&'static [&'static str]`. Wat-side `def-restricted` and Rust-side `#[restricted_to(...)]` then share the same downstream storage; the walker (`validate_def_restricted_caller_namespace`) consults a unified whitelist regardless of declaration origin.
- **Stone 3** applies the attribute to `eval_kernel_*_join_result` substrate fns, replacing the hard-coded substrate-namespace exemption in arc 170 Stone B's walker.
- **Stone 4** retires Stone B's ad-hoc walker rule and the orphaned `JoinResultUserNamespace` `CheckError` variant once the generic mechanism covers `*_join-result`.

The substrate teaches its own boundaries; future readers don't grep the walker for hard-coded rules.

## Files touched

- `Cargo.toml` — added `inventory = "0.3"` to `[dependencies]` (with a comment block tying to Stones 1-4)
- `src/lib.rs` — `pub mod restriction_entry;` (one line, alphabetically positioned next to `pub mod resolve;` / `pub mod runtime;`)
- `src/restriction_entry.rs` — NEW. `RestrictionEntry` struct + `inventory::collect!` + module-level doc comment with Stone 2/3/4 trail
- `src/freeze.rs` — new step **6.8** in `startup_from_forms_post_config`: a `for entry in inventory::iter::<crate::restriction_entry::RestrictionEntry>` loop that drains the registry into `symbols.defined_value_restrictions`
- `tests/wat_arc198_slice2_stone_1_inventory_wiring.rs` — NEW. One test that asserts the wiring end-to-end via `inventory::submit!` → `startup_from_source` → `frozen.symbols.defined_value_restrictions.get(...)`
- `docs/arc/2026/05/198-defn-restricted/SCORE-STONE-1-INVENTORY-WIRING.md` — this file (NEW)
