# SCORE — Stone 243.5 — mint `src/types/` home + carve `TypeError`; thread `register_subtype` span

**Verdict: WARDED — L1+L2=0** on both lifted residents (`src/types/error.rs`, `src/types/defstruct.rs`). Stamped `//! vigilatum: 2026-06-01T02:47:26Z — vigilia 9-spell L1+L2=0`.

Scored against the orchestrator's INDEPENDENT re-run (not sonnet's self-report) + a live read of both residents end-to-end (per `feedback_warded_means_annihilated`: a recorded CONVERGED is not a live cast; per `feedback_verify_sonnet_worktree_not_just_return`: git-state verified, no strays, sonnet did not commit).

## Movements (BRIEF M1–M5) — all COMPLETE

| M | What | Verified |
|---|---|---|
| M1 | mint `src/types/` home (NOT mv — `pub mod error;` + sibling files, mirroring `src/check.rs:49`) | `ls src/types/` = error.rs + defstruct.rs; flat types.rs stays |
| M2 | carve `TypeError`+`TypeErrorKind`+2 Display+`impl Error` → `types/error.rs` | read end-to-end; re-export via `pub use error::{TypeError, TypeErrorKind}`; ZERO consumer churn (workspace build clean) |
| M3 | thread `register_subtype(span)` + retire runes | 3-arg sig; emitter uses caller span; 2 call sites correct (407 threads real span, 1421 built-in seed `Span::unknown()` w/ comment); probe bites |
| M4 | decompose `parse_defstruct` → `types/defstruct.rs` | 5 named single-concern fns; behavior preserved; every error carries a real node span |
| M5 | cascade + probe green | independently re-run below |

## Runes retired — 3 (BRIEF predicted 2; sonnet found a third, honestly)

| Rune | Was | Disposition |
|---|---|---|
| `struere(host-constraint)` | types.rs:441–445 (register_subtype emitter) | DELETED — emitter now uses caller span; the excuse is false |
| `conformare(spanless-by-domain)` | types.rs:1557–1560 (CyclicSubtype doc) | DELETED — variant doc in error.rs now reads "The span is the caller-supplied declaration span, not a baked-in unknown" (the thesis, honestly inscribed) |
| `solvere(deferred-stone-243.5)` | types.rs:1896–1900 (parse_defstruct doc) | DELETED — the decomposition this deferral pointed at LANDED |

Independent grep: ZERO `deferred-stone-243.5` runes remain in `src/`. The surviving `struere(host-constraint)` at `types.rs:2188` is a DIFFERENT fn (`parse_type_expr`), correctly out of scope, correctly left. recensere should now confirm these three retirements (the runes it was watching are gone).

## Independent re-run (orchestrator, release)

| Gate | Result |
|---|---|
| `cargo build -p wat` | clean (pre-existing `list_span` warnings elsewhere — see Honest deltas; NOT this stone) |
| `cargo clippy` on `src/types/error.rs` + `src/types/defstruct.rs` | **ZERO** (R2 sweep fixed the 2 home L2s: `doc_lazy_continuation` + `type_complexity`→`ParsedStructMeta` alias) |
| probe `probe_arc243_stone5_register_subtype_span` | 1 passed / 0 failed (compiles + the span-survival assert bites) |
| probe `probe_arc237_sA_hierarchy` (cascade) | 10 passed / 0 failed |
| lib | 895 passed / 0 failed / 1 ignored |
| banked | `probe_arc216_stone5b_hashset_native_storage::probe_8_atom_round_trip` — pre-existing HashSet debt, unrelated, left |

## Honest deltas (independent re-run found what the SCORE-report did not)

1. **My FM 2-bis probe was partly toothless.** `Span::eq` returns `true` unconditionally (`src/span.rs:120` — structural-transparency by arc 016 doctrine), so my original `assert_eq!(err.span, span)` proved nothing. Sonnet caught it during the strike and corrected to `assert!(!err.span.is_unknown())` — the behavioral check that actually verifies the caller span survived. The Shadowdancer found the hole in the Inquisitor's trap; the probe now bites. (The `assert_eq!` line is retained as documentation of intent but is non-load-bearing; `is_unknown()` is the live assertion.)
2. **The `list_span` warning wall (~150 sites) is PRE-EXISTING, not this stone.** Spread across assertion.rs / edn_shim.rs / fork.rs / io.rs / runtime.rs / string_ops.rs / thread_io.rs / time.rs — every kernel-verb eval fn takes `list_span: &Span` and doesn't yet thread it into its RuntimeError. NONE in src/types.rs or src/types/. This is the RuntimeError span-thread debt (the 243.7+ rolling-audit error type), banked as its own task — NOT widened into here (scope fence held).
3. **3 runes retired, not the predicted 2** — the BRIEF undercounted; sonnet surfaced + closed the `solvere` deferral honestly. Calibration delta in the right direction (more eliminated than predicted).

## Scope fence (held)

Did NOT touch CheckError (243.6), the doctrine rewrite (243.4), the other `parse_*` decl fns, or open any new arc. `parse_defstruct` was in scope only as the named struere F3 deferral owner.

## What this unblocks

- arc 243's "zero exceptions" is now TRUE in code (the last `spanless-by-domain` rune is gone) → 243.4 (doctrine rewrite) can be written honestly.
- `src/types/` is now a warded home; future error-type carves (243.7+) have a second proven home pattern to mirror.
