# EXPECTATIONS — Stone 243.5 (orchestrator predictions, pre-spawn)

Falsifiable predictions, recorded BEFORE sonnet's flight, scored against an independent re-run after. Deltas are honest data, not failures.

## Site counts (from crawl @ HEAD `aedee4f5`)

| Prediction | Value | Basis |
|---|---|---|
| `register_subtype` call sites to update | 2 | crawl: types.rs:407 + types.rs:1421 (grep was exhaustive across `src/`) |
| `register_subtype` signature changes | 1 | types.rs:446 |
| Runes retired | 2 | struere(host-constraint) @441–445; conformare(spanless-by-domain) @1557–1560 |
| `TypeError`/`TypeErrorKind` carve block | ~246 lines | types.rs:1429–1674 |
| `parse_defstruct` decomposition source | ~380 lines | types.rs:1901–2281 |
| New files | 2 | `src/types/error.rs`, `src/types/defstruct.rs` |
| `src/types.rs` module-decl lines added | ~4 | `pub mod error;` + re-export + `pub(crate) mod defstruct;` + re-export |
| External consumer churn (imports) | 0 | the `pub use` re-export preserves `crate::types::*` paths (mirrors check.rs:50) |

## Green-state prediction

- `cargo build -p wat`: clean.
- Probe `probe_arc243_stone5_register_subtype_span`: **compiles + passes** (flips from today's E0061 disconfirm).
- `cargo test -p wat` lib: **895/0/1** (unchanged — this is a structural carve + a 2-site span thread; no behavior change to type logic).
- Integration: green EXCEPT the one banked `probe_8_atom_round_trip` (HashSet debt, unrelated).
- Clippy: unchanged from baseline (~897 pre-existing; the carve adds none).

## Risk predictions (where sonnet might legitimately STOP)

1. **LOW** — a third `register_subtype` caller hiding behind a macro/trait the grep missed. Crawl says no (the `grep -rn` was clean), but the build is the final word. If found → STOP expected.
2. **LOW-MED** — `parse_defstruct` shares mutable state across its concerns such that clean decomposition would change behavior. If so → STOP expected (decomposition is not redesign).
3. **LOW** — `error.rs` Display bodies reference a `crate::types`-private item that doesn't re-export cleanly. Mitigated: Display impls are self-contained (verified at 243.3); risk is a stray `use`.

## Predicted shape of the SCORE

Per-movement COMPLETE ×5; cascade table (2 files created, ~4 lines added to types.rs, 2 call sites, 2 runes deleted); probe compiles+passes; lib 895/0/1 + 1 banked. If sonnet reports more than ~2 surprises, that's a calibration signal worth reading, not rubber-stamping.

## Not in this stone (scope fence — score MUST NOT drift into these)

CheckError (243.6), the doctrine rewrite (243.4), the other `parse_*` decl fns, any new arc. If the SCORE touches these, it's scope creep.
