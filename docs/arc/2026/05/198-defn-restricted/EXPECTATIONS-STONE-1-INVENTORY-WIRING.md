# Arc 198 Slice 2 Stone 1 EXPECTATIONS

**BRIEF:** `BRIEF-STONE-1-INVENTORY-WIRING.md`

## Independent prediction

**Runtime band:** 60 minutes sonnet.

Reasoning:
- `inventory` crate addition to Cargo.toml: trivial (1-3 lines)
- `RestrictionEntry` struct + `inventory::collect!`: ~10 LOC
- Setup-time iteration: ~10 LOC at the right location
- 1 test: ~60-90 LOC (setup + probe submit + assertion)
- Smaller than Stone A's 90-120 min band; this is the lighter substrate addition

**Time-box:** 90 min hard stop.

## SCORE methodology

5 rows YES/NO per BRIEF; per-row evidence patterns:

- **Row A** (Cargo.toml): `grep "^inventory" Cargo.toml` shows the dep
- **Row B** (struct + collect!): grep `RestrictionEntry\|inventory::collect` in `src/` shows both
- **Row C** (iteration): grep `inventory::iter::<RestrictionEntry>\|inventory::iter` shows the loop in setup flow
- **Row D** (test passes): `cargo test --release -p wat --test wat_arc198_slice2_stone_1_inventory_wiring` → green
- **Row E** (workspace baseline maintained): cargo test summed failed ≤ 4

## Honest deltas to watch for

- **Iteration landing point.** Two reasonable spots:
  - **(α) Populate `sym.defined_value_restrictions`** before the mirror clone runs → consistent with arc 198 slice 1's pattern (wat-side `def-restricted` also populates sym via `register_defines`)
  - **(β) Populate `env.defined_value_restrictions` directly** after mirror clone → more direct but breaks the "sym is source of truth" mental model
  - Sonnet's call. (α) feels right for consistency, but verify against the actual freeze flow.

- **Static-init ordering with inventory crate.** The `inventory` crate uses linkme-style auto-registration; entries are collected at link time. In tests, `inventory::submit!` calls in test files must be at module-scope to be collected. Test fixture needs to be designed accordingly.

- **`'static` lifetime requirements.** `inventory::submit!` requires the data to be `'static`. `RestrictionEntry { wat_name: &'static str, prefixes: &'static [&'static str] }` works because string literals are `'static`. Sonnet to verify no surprises.

- **Cargo.toml workspace vs crate level.** Adding `inventory` to root Cargo.toml might require workspace-level config or feature gating. Sonnet to discover.

- **Pre-existing test failures.** 4 pre-existing failures from arc 198 slice 1 baseline are unrelated to this stone. NEW failures should be zero.

## Workspace baseline (commit `8f794ff`)

- `cargo build --release --workspace --tests`: clean
- `cargo test --release --workspace --no-fail-fast`: 4 pre-existing target failures (lifeline flake, t6 unquote, totally_bogus, startup_error)

Post-Stone-1 target:
- ≥ baseline + 1 passed (1 new test)
- ≤ baseline failed (no regressions; additive change)

## Calibration record (to fill on completion)

| Metric | Predicted | Actual |
|---|---|---|
| Wall-clock runtime | 60 min | TBD |
| Scorecard rows | 5/5 PASS | TBD |
| Workspace fail count | ≤ baseline (4) | TBD |
| New test count | 1 | TBD |
| Iteration landing | (α) sym OR (β) env | TBD |
| RestrictionEntry location | adjacent to CheckEnv OR new module | TBD |
| Substrate-discovery surprises | 0-2 | TBD |
| Mode | Additive (pure substrate infrastructure) | TBD |
