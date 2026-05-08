# Arc 163 — Slice 3f EXPECTATIONS

**Drafted 2026-05-07.** Pre-spawn predictions for substrate primitive-path
FQDN sweep.

## Independent prediction

**Mode A.** ~50-80 minutes wall-clock.

Mechanical sweep parallel to slice 3e. ~155 substrate sites + 5
canonicalize arms flip + iteration from cargo test diagnostics.

Slice 3e baseline: 848 → 0 in 7 iterations. Slice 3f scope ~30%
larger (155 vs 118 substrate writes) but the discipline is settled —
sonnet (or executor) iterates from diagnostics.

## Hard scorecard

| Row | Pass criterion |
|---|---|
| R1 | Workspace pre-fix: 2041 passed / 0 failed (slice 3e baseline) |
| R2 | Workspace post-fix: 2041 passed / 0 failed (or higher passing) |
| R3 | `cargo build --release` exits clean throughout |
| R4 | Audit grep `":i64"\|":f64"\|":bool"\|":String"\|":u8"` (with `.into()` context) returns 0 Bucket A live writes |
| R5 | parse_type_inner primitive arms FLIP from DOWNGRADE (`:wat::core::i64 → :i64`) to UPGRADE (`:i64 → :wat::core::i64`) |
| R6 | Value::type_name primitive arms flipped FQDN (slice 3e reverted them; slice 3f finishes the work) |
| R7 | Container-head canonicalize-upgrade arm at types.rs:1683 STAYS (slice 3h retires both together) |
| R8 | NO test fixture wat-source updates (slice 3g scope) |
| R9 | Sonnet's report includes per-phase counts + waterfall + at minimum 2 honest deltas |

## Path classifications

- **Mode A**: clean sweep, all rows pass, audit confirms. ~50-80 min.
- **Mode B**: sweep lands but with self-correction (touched a
  Rust-language identifier outside-quotes). Acceptable; flag in report.
- **Mode C**: build doesn't compile, OR audit grep R4 returns > 0,
  OR test count regressed, OR slice 3g scope violated. Stop + report.

## Honest deltas to flag

- Sites where bare-form path is in a Rust-only context (e.g.,
  `expected: ":i64"` in a Rust-deps marshal site that names the
  underlying Rust type) — those may stay bare; classify case-by-case.
- Tests that hardcode bare-form path strings in expected-output
  assertions: list them; orchestrator may update tests separately.
- If slice 3f surfaces an unexpected category (mirror of slice 3e's
  wat-macros codegen finding): name the category.

## Time-box

2× upper-bound = 160 min. ScheduleWakeup pacing if needed.

## What "done" looks like

After this slice, substrate-internal storage for ALL wat type
references — both container heads (slice 3e) AND primitive paths
(slice 3f) — is FQDN. Source FQDN flows through unchanged. Source
bare forms still rejected by walker. Reading any substrate file
shows `":wat::core::i64"`/`"wat::core::Vector"` etc. consistently —
no mixed convention. The "internal looks like our form / bare short
form" inconsistency is closed for substrate.

Slice 3g then sweeps user-source bare primitive fixtures.
Slice 3h retires the upgrade arms and gates arc 163 closure.
