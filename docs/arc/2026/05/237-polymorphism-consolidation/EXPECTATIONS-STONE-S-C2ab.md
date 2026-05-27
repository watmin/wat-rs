# EXPECTATIONS — Stone S-C.2ab (field names → RecordDef + re-route name-access)

Mode A: baseline-preserving (holonic answers identical via the new source) + the records-thread
probes green (after the mechanical recordtype 3-arg arity updates) + scope held.

## Scorecard

| # | Row | Verification | Expected |
|---|-----|--------------|----------|
| 1 | Build clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | Lib baseline | `cargo test --release --lib -p wat 2>&1 \| tail -3` | `827 passed; 0 failed` (1 ignored) |
| 3 | keyword-access (NEW source, same answers) | `probe_arc234_stone3c_keyword_accessor` | pass |
| 4 | record assoc (NEW source + parity) | `probe_arc234_stone3b_record_assoc` | pass |
| 5 | S-A1 (after recordtype 3-arg update) | `probe_arc237_sA1_assignable` | `6 passed; 0 failed` |
| 6 | S-B.1 (after 3-arg update) | `probe_arc237_sB1_recordtype` | `6 passed; 0 failed` |
| 7 | S-B.2 defrecord | `probe_arc237_sB2_defrecord_recordtype` | `5 passed; 0 failed` |
| 8 | defrecord surface | `probe_arc227_stone2_defrecord` | pass |
| 9 | scope | `git status --short` | src/{types,runtime}.rs + wat/Record.wat + the named test files + SCORE; NO holon-rs; NO base variant; NO macro split |

**Clippy NOT a ceiling concern.**

## Independent prediction

**Target band: 40–70 min Mode A. STOP-3: 90. STOP-4: 120.** The variable is the recordtype
3-arg ripple (grep all callers — S-A1/S-B.1 probes + macro + any other) and the macro's
field-name emission (reuse its existing extraction). Baseline-preserving by construction —
holonic keyword-access/assoc must give identical answers (parity); if any differ, the re-route
mis-resolved name→index → STOP.

## Risks / trap-doors

1. **Name→index order mismatch.** `field_names` order MUST equal declaration order ==
   `struct_form` order == the old `holon_form` Bundle order. If the macro emits names in a
   different order, accessors return the wrong field. The multi-field keyword-access/assoc
   regression case is the guard — ensure one exists.
2. **Breaking holonic assoc parity.** The re-route changes only the name→index *source*;
   holonic assoc must STILL rebuild both `struct_form` and `holon_form` (runtime.rs:16912 +
   16917-43). Don't drop the holon rebuild.
3. **Missed recordtype caller.** The 3-arg HARD CUT breaks every caller; grep `recordtype`
   across tests/ + wat/ — a missed one is a compile/parse error (rustc/the parser names it).
4. **Reaching for the base variant or macro split.** Out of scope (S-C.2c / S-C.3).
5. **A non-obvious error** → pivot to the diagnostic, don't guess (`feedback_nonintuitive_error_is_pivot`).

## SCORE

`SCORE-STONE-S-C2ab.md` (NEW). 9-row scorecard + the changes (RecordDef.field_names;
recordtype 3-arg + parse; macro emission; the 3 re-routed sites; recordtype-caller updates) +
honest deltas + working tree. Mirror SCORE-STONE-S-C1.
