# EXPECTATIONS — Stone 241.6 — Phase 2 opens: metadata-map storage on `def`

Independent scorecard for orchestrator-side verification. No vigilia phase (legacy flat substrate per DESIGN D7 default). Commit on SCORE-green.

## Phase A — Scorecard (11 rows)

| Row | Claim | Verification command | Expected result |
|-----|---|---|---|
| 1 | Probe contracts 01-03 PASS (def + defn with metadata) | `cargo test --release --test probe_arc241_stone6_def_metadata_map contract_0[1-3]` | 3 passed; 0 failed |
| 2 | Probe contracts 04-05 PASS (regression: no-metadata) | `cargo test --release --test probe_arc241_stone6_def_metadata_map contract_0[45]` | 2 passed; 0 failed |
| 3 | Probe contract 06 PASS (empty `{}` rejected) | `cargo test --release --test probe_arc241_stone6_def_metadata_map contract_06` | 1 passed; 0 failed |
| 4 | Probe whole-suite 6/6 | `cargo test --release --test probe_arc241_stone6_def_metadata_map` | 6 passed; 0 failed |
| 5 | Stone 241.5 probe preserved 8/8 | `cargo test --release --test probe_arc241_stone5_defclause_rest_dispatch` | 8 passed; 0 failed |
| 6 | Stone 241.4 canonical probe preserved 15/15 | `cargo test --release --test probe_arc241_stone1_argspec_canonical` | 15 passed; 0 failed |
| 7 | Stone 241.2 + 241.3 probes preserved | `cargo test --release --test probe_arc241_stone2_fn_parser_migration --test probe_arc241_stone3_defclause_parser_migration` | 10+6 passed; 0 failed |
| 8 | 237.8b Gate 1 PASS preserved | `cargo test --release --test probe_arc237_8b_defclause_arithmetic gate_1_defclause_supports_rest_binder` | 1 passed; 0 failed |
| 9 | Lib baseline preserved | `cargo test --release --lib -p wat` | 834 passed; 0 failed (or higher) |
| 10 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0; 0 errors |
| 11 | Clippy delta ≤ 0 | `cargo clippy --release 2>&1 \| grep -cE "^warning:"` | ≤ 904 |

## Structural verification (5 rows)

| Verification | Command | Expected |
|---|---|---|
| 4-item def discrimination present in try_parse_fn_shape_def | `grep -A 30 "^fn try_parse_fn_shape_def" src/runtime.rs \| grep -c "items.len() == 4\|items.len() >= 3"` | ≥ 1 (the new branch) |
| `:wat::core::HashMap` head detection | `grep -n "wat::core::HashMap" src/runtime.rs \| head -5` | ≥ 1 match in def parsers |
| SymbolTable.binding_metadata exists | `grep -n "binding_metadata" src/runtime.rs src/check.rs \| head -3` | ≥ 1 match |
| defn macro updated | `grep -rn "metadata\b" wat/*.wat \| head -5` | metadata-flow comment OR macro body shows metadata threading |
| `src/argspec/*` UNCHANGED | `git diff src/argspec/` | empty diff |

## Independent prediction (runtime band)

**Target band: 25-45 min Mode A.**
**Upper bound: 50 min (STOP-3).**

**Mode B triggers**:
- Stone 241.6 probe < 6/6 PASS
- Lib < 834
- Files outside discipline touched
- src/argspec/* or src/lib.rs modified
- Stone 241.x probes regress
- New variants/types/fields beyond binding_metadata
- defn macro inheritance broken
- Empty `{}` rejection broken
- Clippy > 904

**Mirror precedent: Stone 241.5** (~10 min Mode A; multi-site coordinated edit with macro touch). Stone 241.6 has similar shape (substrate + macro extension) but is parser-level rather than dispatch-level.

## Trap-door risks

| # | Risk | Detection | Resolution |
|---|---|---|---|
| **T1** | defn macro not found (lives in unexpected location) | sonnet's investigation | Acceptable; surface in honest delta + sonnet picks correct file |
| **T2** | SymbolTable extension breaks existing serialization / freeze paths | lib regression | Re-brief; the extension should be additive |
| **T3** | Empty `{}` already rejected at parser level | contract 06 passes pre-stone (verified: passes at HEAD) | No def-level work needed; document |
| **T4** | Plain-value def parser missed (only fn-shape touched) | contract 01/02 may fail | Re-brief; both paths must be extended |
| **T5** | Metadata flows into Function.metadata accidentally (mixing binding vs value layer) | grep Function struct for metadata field | Re-brief; metadata is BINDING-level, not value-level |
| **T6** | def-restricted breaks via shared parser path | regression on arc 203 tests | Re-brief; def-restricted MUST stay untouched |
| **T7** | Macro expansion order conflict with metadata threading | defn-with-metadata test fails on expansion | sonnet investigates expansion order; re-brief if needed |
| **T8** | Test cascade larger than zero | lib regression diagnostics | Document as honest deltas |
| **T9** | Sonnet introduces namespaced home (e.g., src/metadata/) | git diff shows new directory | Acceptable IF cleanly bounded; vigilia gate would apply (cast Phase B); STOP-6 if forces broader changes |
| **T10** | TypeExpr/HolonAST representation issue with metadata values | parser/check errors on map values | sonnet investigates; surface |

## Pre-spawn baseline checks

1. **Stone 241.6 probe at HEAD = 3 PASS / 3 FAIL** (verified — 4-item def fails; defn with metadata fails on signature parsing).
2. **All other Stone 241.x probes at current PASS counts**.
3. **Lib = 834 PASS / 0 FAIL**.
4. **237.8b Gate 1 = PASS**.
5. **Clippy = 904**.

## What completion looks like

### Phase A — SCORE scorecard + structural verification

After sonnet returns Mode A:
- 11/11 scorecard rows verify
- 5/5 structural rows verify
- `SCORE-STONE-241.6.md` written with verbatim results + honest deltas
- PHASE 2 OPENS inscription: storage shipped; reflection (241.7) queued

### Phase B — NOT cast (legacy flat substrate per DESIGN D7 default)

UNLESS sonnet introduces a namespaced home (e.g., `src/metadata/`) — then vigilia gate applies on that home.

### Phase C — Commit + push

- Atomic commit covers: substrate files touched, defn macro file, probe, SCORE doc
- Push to origin
- Stone 241.7 (metadata-of reflection verb) opens next

## Calibration history reference

| Stone | Class | Surface delta | Predicted | Actual | Status |
|---|---|---|---|---|---|
| 241.1-241.4 | Phase 1 parser shape | various | various | various | Phase 1 capstone |
| 241.5 | Runtime dispatch + Gate 1 | +190 net | 20-40 min | ~10 min | UNDER |
| 241.6 (this) | Metadata-map storage | ~+215 net | 25-45 min | TBD | — |

Per Phase 1 calibration: substrate tests assert structurally; cascade likely zero. defn macro touch is the main unknown.

## What this unblocks

**Stone 241.7** — `:wat::runtime::metadata-of` reflection verb (reads `binding_metadata`). Per FORM-COLLAPSE-NOTES locked verdicts: return `Option<HashMap<Keyword, HolonAST>>` encoded as `#wat.core/Some {...}` / `#wat.core/None nil`.

**Phase 2 complete** at Stone 241.7. Phase 3 (HARD CUTs at Stones 241.8-241.10) then proceeds. Phase 4 (INSCRIPTION at 241.11) closes the arc.
