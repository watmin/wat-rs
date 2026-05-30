# SCORE — Stone 243.3 — TypeError Pattern A retrofit

## Phase A — substrate refactor verified

**Mode:** A

### Per-step audit

| Step | Status | Notes |
|---|---|---|
| S1 — Refactor `src/types.rs` shape | COMPLETE | `pub enum TypeError` → `pub struct TypeError { pub span: Span, pub kind: TypeErrorKind }` + `pub enum TypeErrorKind`; span stripped from all 16 variants; rune placed on `CyclicSubtype`; `Display` impl updated to `match &self.kind` with `span_prefix(&self.span)` |
| S2 — Emitter cascade (types.rs) | COMPLETE | 79 emitter sites in `src/types.rs` updated; 1 site in `src/argspec/error.rs` (`From<ArgSpecError> for TypeError`) updated |
| S3 — Consumer cascade | COMPLETE | 16-arm span-extraction match in `src/function/parse.rs:154-172` collapsed to `e.span` (5 lines vs 17); WHY comment deleted; `tests/probe_arc237_stone1_typeunion_substrate.rs` 5 match arms updated; `src/check.rs` 4 `matches!` macros updated; `src/types.rs` internal test `matches!` macros updated |
| S4 — From impl span preservation | COMPLETE | `From<ArgSpecError> for TypeError` in `src/argspec/error.rs` updated to `TypeError { span, kind: TypeErrorKind::MalformedDecl { head, reason } }`; `From<TypeError> for StartupError` in `src/freeze.rs` passes the struct through intact — span preserved |
| S5 — Test cascade | COMPLETE | `tests/probe_arc237_stone1_typeunion_substrate.rs` updated; FM 2-bis probe `tests/probe_arc243_stone3_typeerror_pattern_a.rs` compiles + passes 3/0 |
| S6 — CONFORMARE.md update | COMPLETE | Concrete `TypeError` + `TypeErrorKind` code block added under "First applied example — TypeError (Stone 243.3)"; 16-arm collapse noted; arc reference wired |

### Cascade audit table

| File | Sites updated | Category |
|---|---|---|
| `src/types.rs` | 79 emitters + 7 `matches!` macros | Emitters + inline tests |
| `src/argspec/error.rs` | 1 From impl + import | Emitter (From conversion) |
| `src/function/parse.rs` | 1 consumer (16→5 lines) | Consumer (collapse) |
| `src/check.rs` | 4 `matches!` macros | Consumer (test asserts) |
| `tests/probe_arc237_stone1_typeunion_substrate.rs` | 5 match arms + import | Consumer (probe test) |
| `docs/CONFORMARE.md` | 1 section added | Documentation |

**Total emitter sites updated: ~80** (79 in types.rs + 1 in argspec/error.rs)
**Total consumer sites updated: ~10** (16-arm collapse + 9 matches/asserts)

### Honest deltas

- `src/types.rs`: net positive lines (struct + kind enum declaration overhead; constructor patterns slightly longer than enum direct)
- `src/function/parse.rs`: net -12 lines (17-line 16-arm match → 5-line struct literal)
- Pattern A structurally eliminates the class: `TypeError` without `span` is now uncompilable

### Trap-doors encountered + absorbed

| # | Trap-door | Resolution |
|---|---|---|
| T1 | Display impl needed `match &self.kind` with `self.span` | Mechanical; resolved during S1 |
| T2 | `tests/probe_arc237_stone1_typeunion_substrate.rs` match arms on old shape | Updated 5 arms + import to `TypeErrorKind` |
| T3 | `From<ArgSpecError> for TypeError` in argspec/error.rs | Updated to Pattern A constructor; span from `classify()` preserved |
| T4 | MalformedVariant span field was in middle of field list (not at end) | Stripped cleanly; outer struct span handles it |
| T5 | 16-arm match collapse — load-bearing UX win | Collapsed to `e.span` as designed; 17 lines → 5 |
| T6 | Sub-stone scope creep — RuntimeError, CheckError, ParseStep remain flat | NOT touched; TypeError only this stone |

### Final metrics

| Metric | Value |
|---|---|
| Lib tests | 890 / 0 |
| tests/function | 8 / 0 |
| FM 2-bis probe (`probe_arc243_stone3_typeerror_pattern_a`) | 3 / 0 |
| Workspace test-build | clean (exit 0) |
| Clippy warnings | 897 (unchanged from baseline) |

### Structural verification results (all green)

| Check | Result |
|---|---|
| `pub struct TypeError` present | 1 match at line 1421 |
| `pub enum TypeErrorKind` present | 1 match at line 1429 |
| `pub enum TypeError` GONE | 0 matches |
| `rune:conformare` on CyclicSubtype | 1 match at line 1549 |
| TypeError outer struct `span: Span` | confirmed (line 1422) |
| TypeError outer struct `kind: TypeErrorKind` | confirmed (line 1423) |
| TypeErrorKind variants: no `span:` fields | 0 matches |
| 16 variants in TypeErrorKind | 16 confirmed |
| 16-arm match GONE from parse.rs | 0 matches |
| BadRetType arm uses `e.span` | confirmed |
| WHY comment for 16-arm match GONE | 0 matches |
| From<TypeError> for StartupError preserves span | passes struct through intact |
| Display impl matches on `&self.kind` | confirmed |
| No backward-compat aliases | 0 matches |
| No deferral language in stone-touched code | 0 matches |
| CONFORMARE.md cites Stone 243.3 | 2 matches |

---

## Phase B — conformare re-cast attestation

Phase B conformare re-cast attestation pending orchestrator cast.
