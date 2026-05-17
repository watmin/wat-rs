# EXPECTATIONS — Arc 203 Slice 1: substrate primitive minting

**BRIEF:** `BRIEF-SLICE-1.md`
**Drafted:** 2026-05-16, pre-spawn.

## Independent prediction

**Runtime band:** 60-90 min sonnet.

Reasoning:
- ~1-2 new code paths in src/check.rs (infer_struct_restricted; keyword recognition)
- ~1-2 new code paths in src/runtime.rs (register_runtime_defs_form arm; register_struct_methods extension)
- 6 tests in new tests/wat_arc203_struct_restricted.rs
- Mechanism is fully analogous to arc 198 slice 1 (~75 min actual) but more involved on the runtime-side (struct synthesis path is heavier than def's simple binding path)
- No new walker code, no new error variants, no inventory wiring
- arc 198 pattern is established and well-documented (INSCRIPTION lives at docs/arc/2026/05/198-defn-restricted/INSCRIPTION.md)

**Time-box:** 120 min hard stop.

## Scorecard prediction

| Row | Predicted | Confidence |
|-----|-----------|------------|
| A — form parses for worked example | YES | high |
| B — constructor restriction fires | YES | high (mirror of arc 198 mechanism) |
| C — per-field restriction fires | YES | high (same mechanism applied per accessor) |
| D — public accessors unrestricted | YES | high (no entry in HashMap = no restriction) |
| E — empty sections honored | YES | medium-high (some shape-parsing edge cases; sonnet will iterate if needed) |
| F — workspace baseline preserved | YES | high (purely additive substrate primitive) |

**6/6 PASS predicted; ~80% confidence overall.** Higher than arc 198 slice 1 (~70% pred) because we have arc 198's worked pattern as a template; lower than mechanical sweeps because this is genuine substrate addition.

## Honest deltas predicted (to watch for in SCORE)

### Likely surfaces

1. **`register_struct_methods` signature** — current shape is `(types, sym)`; arc 203 needs to ALSO know per-struct restrictions. Likely fork into a `register_struct_methods_with_restrictions` companion OR thread the restrictions through `TypeDef::Struct` itself. Sonnet picks; surface which.

2. **Restricted-section shape parsing** — per-field whitelists nested inside the restricted section are a slightly novel parse shape (each entry is `([wlist] field <- :T)` triple — not a flat name-type pair). The substrate's existing field-list parser may need a variant or a parallel parser. Surface if the parser needed adjustment.

3. **Where to register at freeze** — arc 198 wires `def-restricted` registration via `register_runtime_defs_form` for the def case. The struct case is more nested (struct decl → register_struct_methods → synthesizes /new + /<field>). The right injection point is probably AT register_struct_methods (after method synthesis, register the restrictions). Surface the chosen injection point.

4. **TypeDef extension** — does `TypeDef::Struct` need to carry the restrictions metadata, or do we side-table them at registration time? If sonnet extends TypeDef, surface the variant change; if side-table, surface where the state lives until register_struct_methods consumes it.

### Less likely surprises

5. **CheckEnv vs SymbolTable mirroring** — arc 198 wrote restrictions to both. Arc 203 needs the same mirror. Sonnet should follow arc 198's two-write pattern; surface if there's a mismatch.

6. **Whitelist prefix-matching semantics inheritance** — arc 198's rules: trailing `::` = namespace prefix; no trailing `::` = exact FQDN. Arc 203 uses the same rules (the walker is reused). Sonnet should verify the prefix entries are extracted to strings exactly like arc 198 does; surface if there's a discrepancy.

## Workspace baseline (verified 2026-05-16 just before this spawn)

`cargo test --release --workspace --no-fail-fast` baseline: clean except 3 pre-existing stable failures:
- `deftest_wat_tests_tmp_totally_bogus`
- `startup_error_bubbles_up_as_exit_3`
- `t6_spawn_process_factory_with_capture_round_trips`

(Lifeline flake may add ±1 per run per recovery doc § 7 calibration; treat as variance not regression.)

Post-slice-1 target:
- Pass count: ≥ baseline + 6 (six new arc 203 tests pass)
- Fail count: ≤ 3 + lifeline-variance

## Calibration record (filled at completion)

| Metric | Predicted | Actual | Delta |
|---|---|---|---|
| Wall-clock runtime | 60-90 min | TBD | TBD |
| Scorecard rows | 6/6 PASS | TBD | TBD |
| Workspace fail count | ≤ 3 + lifeline | TBD | TBD |
| New test count | 6 | TBD | TBD |
| Substrate↔assumption gaps surfaced | 1-3 (register_struct_methods extension, section-shape parsing, TypeDef carrying restrictions all candidates) | TBD | TBD |
| BRIEF corrections suggested | 0-2 | TBD | TBD |
| STOP-triggers fired | 0-1 | TBD | TBD |

**Calibration summary (post-SCORE):** TBD.
