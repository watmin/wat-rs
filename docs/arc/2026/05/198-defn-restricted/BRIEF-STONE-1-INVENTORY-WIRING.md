# Arc 198 Slice 2 Stone 1 BRIEF — inventory wiring + RestrictionEntry struct + setup iteration

**Arc:** 198 slice 2, Stone 1 of 4 (decomposed from the originally-monolithic slice-2 BRIEF — see `BRIEF-RUST-ATTRIBUTE.md` SUPERSEDED note).
**Task:** #328
**Predecessors:** arc 198 slice 1 (commit `24d3b0d`) — wat-side `def-restricted` / `defn-restricted` mechanism is complete. Arc 170 Stone B (commit `2a071f0`) — ad-hoc walker rule for `*_join-result` is in place.
**Successors:**
- Stone 2 — mint `#[restricted_to(...)]` proc-macro attribute that emits `inventory::submit!` entries (depends on this stone's `RestrictionEntry` struct)
- Stone 3 — apply attribute to `eval_kernel_*_join_result`
- Stone 4 — delete Stone B's ad-hoc walker rule

## Goal

Add substrate infrastructure for Rust-side restriction declarations via the `inventory` crate. This stone is SUBSTRATE-ONLY — no proc-macro, no user-facing attribute, no migration. Just the wiring that subsequent stones plug into.

**Three pieces:**

1. **`RestrictionEntry` struct** — a Rust struct holding `wat_name: &'static str` + `prefixes: &'static [&'static str]`. Lives in the wat crate near CheckEnv (probably `src/check.rs` or a new `src/restriction_entry.rs`).
2. **`inventory::collect!(RestrictionEntry)`** — registers the iter target so any crate can `inventory::submit!` entries.
3. **Setup-time iteration** — after `env.register` calls complete (or after `sym` → `env` mirror clone), iterate `inventory::iter::<RestrictionEntry>` and populate `sym.defined_value_restrictions` (or `env.defined_value_restrictions` — sonnet decides which side matches arc 198 slice 1's wat-side population pattern).

**Verification:** ONE test that manually `inventory::submit!`s a `RestrictionEntry` for a probe fn, runs substrate setup, and verifies the entry landed in `sym/env.defined_value_restrictions`.

## Why this exists

Arc 198 slice 2 was originally one monolithic BRIEF that bundled proc-macro + inventory + migration + rule deletion. Predicted 180-300 min — the signal that decomposition was overdue. Sonnet was killed in reading phase to avoid one-shot multi-piece change.

Per `feedback_iterative_complexity`: small funcs; prove each stepping stone. Stone 1 is the smallest first proof — the substrate ground that Stones 2-4 build on.

## Decay disclosure (orchestrator → sonnet)

Orchestrator has had multiple substrate-fact failures across this session. **Sonnet has FULL AUTHORITY on substrate-internal discovery** — exact `RestrictionEntry` struct location (new file vs adjacent to CheckEnv), inventory wiring API, where setup-time iteration lands (post-register vs post-mirror), Cargo.toml updates. Do NOT trust orchestrator claims without grep verification.

## Substrate state pointers (verified)

- `src/check.rs:1654` — `CheckEnv.defined_value_restrictions: HashMap<String, Vec<String>>` (arc 198 slice 1 storage; what you populate)
- `src/check.rs:1696` — `env.defined_value_restrictions = sym.defined_value_restrictions.clone();` (the mirror clone direction; sym → env)
- `src/check.rs:1806` — insert API on env
- `src/runtime.rs:1028` — `SymbolTable.defined_value_restrictions` (the source-of-truth side that mirrors to env)
- `src/runtime.rs:1720` (and others) — examples of how arc 198 slice 1 inserts into `sym.defined_value_restrictions` during `register_defines`
- `Cargo.toml` (root) — add `inventory` dep
- Probe-side: substrate-internal code that runs setup is in the freeze flow somewhere; sonnet to locate

## Implementation protocol (per `feedback_test_first` + `feedback_iterative_complexity`)

1. **Read existing substrate state.** All pointers above. Pay special attention to:
   - How arc 198 slice 1 populates `sym.defined_value_restrictions` during `register_defines` (the pattern to mirror)
   - Where the substrate setup sequence completes (so iteration can run AFTER all built-ins are registered but BEFORE check phase begins)

2. **Write 1 test FIRST** in `tests/wat_arc198_slice2_stone_1_inventory_wiring.rs`:
   - Use `inventory::submit!` at the test-file top-level (or wherever appropriate) to register a probe `RestrictionEntry { wat_name: ":probe::test::fn", prefixes: &[":wat::"] }`
   - Run substrate setup (e.g., `startup_from_source` with minimal wat source)
   - Assert that `sym.defined_value_restrictions.get(":probe::test::fn")` returns `Some(vec![":wat::".to_string()])`
   - RUN; CONFIRM test fails (RestrictionEntry doesn't exist + no iteration happens yet)

3. **Add `inventory` to Cargo.toml.** Root `Cargo.toml` (the wat crate) needs `inventory = "0.3"` (or whatever current version).

4. **Define `RestrictionEntry` struct.** Place it where it composes cleanly with CheckEnv — probably `src/check.rs` adjacent to `CheckEnv.defined_value_restrictions`, OR a new file `src/restriction_entry.rs` re-exported from the crate root.

   ```rust
   pub struct RestrictionEntry {
       pub wat_name: &'static str,
       pub prefixes: &'static [&'static str],
   }
   inventory::collect!(RestrictionEntry);
   ```

5. **Wire setup-time iteration.** Locate the substrate setup sequence (where `env.register` calls happen for built-ins, e.g., the `register_builtin_types` function or similar). Add iteration AFTER all built-ins are registered:

   ```rust
   for entry in inventory::iter::<RestrictionEntry> {
       sym.defined_value_restrictions.insert(
           entry.wat_name.to_string(),
           entry.prefixes.iter().map(|s| s.to_string()).collect(),
       );
   }
   ```

   (Or insert into env directly if that's more honest given the existing mirror direction. Sonnet decides; document choice in SCORE.)

6. **Build clean.** `cargo build --release --workspace --tests`.

7. **Run test.** All green.

8. **Workspace verification.** `cargo test --release --workspace --no-fail-fast`. Failure count ≤ baseline (4 pre-existing).

9. **Write SCORE.**

## Constraints (HARD)

- DO NOT commit. Orchestrator commits atomically after independent verification.
- Operate ONLY in `/home/watmin/work/holon/wat-rs/`. Anchor cwd; absolute paths route correctly.
- DO NOT touch INSCRIPTIONs / past SCOREs / DEFERRAL-VIOLATIONS / SUPERSEDED BRIEFs / AUDIT / recovery doc / INTERSTITIAL / past STONE BRIEFs / SCOREs / EXPECTATIONS docs / this BRIEF / this EXPECTATIONS / superseded slice-2 monolithic BRIEF + EXPECTATIONS.
- DO NOT mint proc-macro attribute yet — that's Stone 2.
- DO NOT apply attribute to `*_join-result` fns — that's Stone 3.
- DO NOT delete Stone B's ad-hoc walker rule — that's Stone 4.
- DO NOT touch existing `def-restricted` / `defn-restricted` forms (arc 198 slice 1) — they remain unchanged.
- DO NOT modify Stone B's tests — Stone 4 handles updates.
- DO NOT use any path containing `.claude/worktrees/`.
- DO NOT use `--no-verify` / `--no-gpg-sign` / skip hooks. NEVER use destructive git commands.

## Scorecard (5 rows YES/NO with evidence)

| Row | What | Evidence |
|-----|------|----------|
| A | `inventory` crate added to wat crate `Cargo.toml` | `grep "^inventory" Cargo.toml` shows the dep |
| B | `RestrictionEntry` struct defined + `inventory::collect!` registered | `grep -nA 5 "struct RestrictionEntry\|inventory::collect" src/` shows both |
| C | Setup-time iteration populates `defined_value_restrictions` from `inventory::iter` | grep shows the iteration in setup flow |
| D | 1 new test passes — probe `inventory::submit!` lands in `sym/env.defined_value_restrictions` after setup | `cargo test --release -p wat --test wat_arc198_slice2_stone_1_inventory_wiring` → green |
| E | Workspace test failure count ≤ baseline (4 pre-existing: lifeline flake, t6, totally_bogus, startup_error) | full workspace cargo test failures ≤ 4 |

## STOP triggers

- `inventory` crate doesn't compose with the substrate freeze flow (e.g., static-init ordering issues) → STOP and surface
- Setup-time iteration can't find a clean landing spot (sequence ordering ambiguous) → STOP and surface
- New crate dependency causes downstream build failures → STOP and surface
- Migration breaks existing tests (SHOULDN'T HAPPEN — change is purely additive) → STOP and investigate

## Workspace baseline (commit `8f794ff`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 pre-existing target failures (lifeline flake, t6 unquote, totally_bogus, startup_error)

Post-Stone-1 target:
- ≥ baseline + 1 pass (the new inventory-wiring test)
- ≤ 4 failed (no regressions; this is purely additive)

## Time-box

60 min predicted. Hard stop 90 min. If approaching stop, write partial SCORE describing state-at-stop.

## On completion

Write `docs/arc/2026/05/198-defn-restricted/SCORE-STONE-1-INVENTORY-WIRING.md`:
- 5 rows YES/NO with grep-able evidence
- Honest deltas: RestrictionEntry struct location chosen, iteration landing point chosen (sym vs env vs both), inventory crate version chosen, workspace test count vs baseline
- Calibration record (predicted vs actual)

Return final summary: rows passed/failed + RestrictionEntry location + iteration landing + workspace test count delta + path to SCORE.

You are launching now. T-minus 0.
