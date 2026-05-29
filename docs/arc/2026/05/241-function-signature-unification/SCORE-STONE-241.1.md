# SCORE — Stone 241.1 — mint canonical `parse_argspec_triples` at `src/argspec/`

**Date:** 2026-05-28
**Status:** COMPLETE — 10/10 probe PASS. Vigilia convergence pending (Phase B, orchestrator-side).

---

## Scorecard

| # | Row | Command | Result |
|---|-----|---------|--------|
| 1 | Contract 1 PASS (empty argspec, no ret) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_01_empty_argspec_no_ret_type_expected` | 1 passed; 0 failed |
| 2 | Contract 2 PASS (single fixed param, no ret) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_02_single_fixed_param_no_ret` | 1 passed; 0 failed |
| 3 | Contract 3 PASS (multiple fixed + ret) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_03_multiple_fixed_params_with_ret` | 1 passed; 0 failed |
| 4 | Contract 4 PASS (ret-only signature) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_04_ret_only_signature` | 1 passed; 0 failed |
| 5 | Contract 5 PASS (non-Symbol at name slot) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_05_non_symbol_at_name_slot` | 1 passed; 0 failed |
| 6 | Contract 6 PASS (missing `<-` arrow) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_06_missing_arrow_token` | 1 passed; 0 failed |
| 7 | Contract 7 PASS (non-Keyword at type slot) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_07_non_keyword_at_type_slot` | 1 passed; 0 failed |
| 8 | Contract 8 PASS (missing `->` when ret expected) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_08_missing_ret_arrow_when_expected` | 1 passed; 0 failed |
| 9 | Contract 9 PASS (trailing items) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_09_trailing_items_after_ret` | 1 passed; 0 failed |
| 10 | Contract 10 PASS (rest-binder rejected) | `cargo test --release --test probe_arc241_stone1_argspec_canonical contract_10_rest_binder_rejected_when_disallowed` | 1 passed; 0 failed |
| 11 | Probe whole-suite 10/10 | `cargo test --release --test probe_arc241_stone1_argspec_canonical` | 10 passed; 0 failed |
| 12 | Lib baseline preserved | `cargo test --release --lib -p wat` | 834 passed; 0 failed; 1 ignored |
| 13 | Workspace test-build clean | `cargo build --release --tests --workspace` | exit 0; 0 errors |
| 14 | Clippy delta = 0 | `cargo clippy --release --lib -p wat -- -D warnings 2>&1 \| grep -c "warning"` | 54 (= pre-stone baseline; 0 new) |
| 15 | Files touched match discipline | `git diff --name-only` (pre-commit) | `src/argspec/mod.rs`, `src/argspec/parse.rs`, `src/argspec/error.rs`, `src/lib.rs`, `SCORE-STONE-241.1.md` |
| 16 | Arc 237/238 probes no regression | `cargo test --release --test probe_arc237_8b_defclause_arithmetic --test probe_arc237_stone5_conforms ...` | All PASS; 0 failures |

---

## Final API shape

```rust
// src/argspec/mod.rs re-exports:
pub use error::ArgSpecError;
pub use parse::{parse_argspec_triples, ArgSpec, ParseOptions};

// Canonical parser signature:
pub fn parse_argspec_triples(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
    options: ParseOptions,
) -> Result<ArgSpec, ArgSpecError>;

pub struct ArgSpec {
    pub fixed_params: Vec<(String, TypeExpr)>,
    pub rest_param: Option<(String, TypeExpr)>,  // None in 241.1; 241.4 populates
    pub ret_type: Option<TypeExpr>,              // None when include_ret_type = false
}

pub struct ParseOptions {
    pub include_ret_type: bool,   // fn = true; defclause = false
    pub allow_rest_binder: bool,  // 241.4 only; always false in 241.1
}

pub enum ArgSpecError {
    NameNotSymbol        { span: Span, head: String },
    MissingArrow         { span: Span, head: String },
    TypeNotKeyword       { span: Span, head: String },
    MalformedTypeKeyword { span: Span, head: String, inner: Box<TypeError> },
    MissingRetArrow      { span: Span, head: String },
    RetTypeNotKeyword    { span: Span, head: String },
    TrailingItems        { span: Span, head: String, count: usize },
    IncompleteSignature  { span: Span, head: String },
    RestBinderNotSupported { span: Span, head: String },
}

// From<> impls (call-boundary converters):
impl From<ArgSpecError> for RuntimeError { ... }
impl From<ArgSpecError> for CheckError  { ... }
impl From<ArgSpecError> for TypeError   { ... }
```

---

## Line counts per file

| File | Lines |
|------|-------|
| `src/argspec/mod.rs` | 47 |
| `src/argspec/parse.rs` | 219 |
| `src/argspec/error.rs` | 253 |
| `src/lib.rs` delta | +1 line (`pub mod argspec;` before `pub mod comms;`) |
| `tests/probe_arc241_stone1_argspec_canonical.rs` | 186 (committed `e0d1d054`; not a stone deliverable) |
| **Net new** | **519 lines** |

---

## Clippy delta

**0 new warnings.** Pre-stone baseline: 54 (via `cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"`). Post-stone: 54. No `src/argspec/` file contributes any clippy warning.

---

## Lib baseline confirmation

**834 passed; 0 failed; 1 ignored.** Pre-stone baseline was 834 PASS / 0 FAIL (verified before spawn at commit `e0d1d054`). Post-stone identical — pure additive, no existing behavior touched.

---

## Cascade depth

**Zero.** Pure additive stone: new module + probe already committed. A1/A2/A3/A4 untouched at their current file:line locations. No existing call site modified. Cascade begins at 241.2.

---

## Honest deltas from BRIEF sketch

- **`ArgSpecError` does not derive `Clone`**: `TypeError` (held in `MalformedTypeKeyword::inner: Box<TypeError>`) derives only `Debug`, not `Clone`. Deriving `Clone` on `ArgSpecError` would require `Clone` on `TypeError`. Rather than adding `Clone` to `TypeError` (out-of-scope substrate change) or switching to `Arc<TypeError>` (premature), `ArgSpecError` derives only `Debug`. The probe's 10 contracts don't require `Clone` on `ArgSpecError`; downstream migration (241.2/3) converts eagerly via `From<>` rather than cloning errors.

- **`ArgSpec` derives `Clone` correctly**: `TypeExpr` already derives `Clone` (confirmed `src/types.rs:67`); `Vec<(String, TypeExpr)>` and `Option<TypeExpr>` clone cleanly.

- **`From<>` impls are fully wired** (not `todo!()` or `unimplemented!()`): each variant maps to the appropriate native error class per AUDIT.md's per-site invariants table. `RuntimeError::MalformedForm`, `CheckError::MalformedForm`, `TypeError::MalformedDecl` — exact variants used at A1/A4, A3, and B1/B2 sites respectively.

- **`is_bare_symbol` private helper**: named exactly (not `is_symbol_named`). Inline `matches!` pattern — zero allocation. Reachable for all three structural tokens (`"<-"`, `"->"`, `"&"`).

- **BRIEF sketch `idx + 2 >= args_vec.len()`**: the BRIEF's condition `idx + 2 >= args_vec.len()` detects an incomplete triple when fewer than 3 items remain. This is correct: if `idx + 2 == args_vec.len() - 1` there are exactly 2 items left (not 3); shipment condition is `idx + 2 < args_vec.len()` i.e. `idx + 2 <= args_vec.len() - 1`. The `>=` form in the BRIEF (and reproduced here) fires when there are 0, 1, or 2 items remaining — all incomplete.

---

## Working tree on return

```
?? src/argspec/
 M src/lib.rs
?? docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.md
```

No other Rust files modified. STOP-4 (holon-rs) not touched. STOP-5 (Rust outside `src/argspec/*` + `src/lib.rs`) not violated. A1/A2/A3/A4 untouched (STOP-6 clean).

---

## Vigilia Convergence (Phase B, 2026-05-28 mid-day; orchestrator-inscribed)

Per `feedback_namespaced_home_vigilia_gate`: vigilia cast post-SCORE on `src/argspec/*` + `tests/probe_arc241_stone1_argspec_canonical.rs`. Eight spells in parallel — intueri, solvere, purgare, struere, sequi, temperare, complectens, vocare.

**Aggregate: DIVERGED — 4 L1 findings + ~12 L2 findings (significant cross-spell overlap on 3-4 sites).**

### L1 findings (require amend OR rune-accept per grimoire)

| # | Site | Spell(s) | Finding | Direction |
|---|---|---|---|---|
| 1 | `error.rs:56-253` | solvere | Reason-string already DRIFTING across 3 From impls (NameNotSymbol: 2× "arg-vector name slot must be a plain symbol" + 1× "field/arg name slot must be a plain symbol") | Extract `fn classify(&self) -> (Span, String, String)` on ArgSpecError; From impls become mechanical wrappers |
| 2 | `parse.rs:126-142 + 173-189` | solvere + struere | Keyword-parse pattern duplicated for fixed-param type-slot + ret-type slot | Extract `parse_keyword_type(ast, head, err_ctor) -> Result<TypeExpr, ArgSpecError>` |
| 3 | `parse.rs:80-90` | struere (L1) + purgare (L2-rune-acceptable) | `unreachable!` behind a runtime-valid path; `allow_rest_binder=true` panics | RUNE: `// rune:purgare(future-fixture) — Stone 241.4 implements allow_rest_binder=true; 241.1 unreachable by design` (grimoire-prescribed) |
| 4 | `probe:25-35` | struere (L1) + sequi (L2) + complectens (L2) — 3 spells converge | `impl Deref<Target=Span>` return type leaks heap-pin strategy through opaque trait | AMEND: return `(Vec<WatAST>, Span)` owned (clone span at helper boundary; cheap in test code) |

### L2 findings (amend OR rune; lower-priority)

- **`parse.rs:158`** (temperare): redundant `is_bare_symbol(..., "->")` after loop break — tautology. REMOVE.
- **`parse.rs:99`** (solvere + struere): `idx + 2 >= len` opaque. REWRITE as `args_vec.len().saturating_sub(idx) < 3`.
- **`parse.rs:98`** (intueri): WHAT comment ("Need 3 items for a complete triple"). Either remove comment or rewrite as WHY.
- **`probe:25`** (intueri): `argspec_inputs` reads as factory but parses. RENAME `parse_vector_items`.
- **`probe:38`** (intueri): `invoke` too generic. RENAME `parse_triples`.
- **`error.rs:16`** (struere): ArgSpecError doesn't derive Clone (TypeError only derives Debug — verified by sonnet's honest delta). Acceptable; keep as-is.
- **`probe`** (complectens): no per-helper `#[test]` for `argspec_inputs` or `invoke`. Add `#[test]` for each.
- **`probe`** (vocare): contracts 03/04 don't verify TypeExpr content of ret_type. Extend assertions.
- **`probe`** (vocare): 3 ArgSpecError variants UNPROBED (MalformedTypeKeyword, RetTypeNotKeyword, IncompleteSignature). Add contracts 11/12/13.
- **`probe:25`** (vocare): Span not re-exported from `wat::argspec`. Either re-export Span OR document the reach.

### Verdict: DIVERGED on the home; Stone 241.1.fix queued

Per `feedback_namespaced_home_vigilia_gate`: commit-readiness requires L1+L2=0. Compaction-pressure forced Phase A commit ahead of Phase B convergence; the doctrine's compaction-pressure exception (inscribed in CLIFFNOTES Currently) acknowledged this. Phase B now declares the home DIVERGED with 4 L1 + ~12 L2 findings; **Stone 241.1.fix is owed** before Phase 1 advances to Stone 241.2.

### Spells that CONVERGED individually

- **purgare** — CONVERGED with 3 rune-acceptable future-fixture sites (`unreachable!`, `rest_param`, three From impls). No genuine dead code; runes close the gap.
- **sequi** — CONVERGED with 1 L2 (opaque Deref; same site as struere's L1).
- **temperare** — CONVERGED with 1 L2 (redundant check at parse.rs:158).
- **vocare** — CONVERGED with 4 L2 (coverage gaps; no vantage violation).

### Spells that DIVERGED

- **intueri** — 3 L2 (probe helper names + WHAT comment)
- **solvere** — 2 L1 (reason-string drift + keyword-parse duplication) + 3 L2
- **struere** — 2 L1 (unreachable! + Deref leak) + 3 L2
- **complectens** — 3 L2 (missing per-helper tests + Deref leak)

### Cross-spell convergence on key sites

- `probe:25` opaque `impl Deref<Span>`: flagged by struere (L1), sequi (L2), complectens (L2), vocare (L2). Strong signal — AMEND, not rune.
- `parse.rs` keyword-parse duplication: flagged by solvere (L1) + struere (L2). AMEND via extracted helper.
- `error.rs` From impl structure: flagged by solvere (L1 message drift) + struere (L2 wrong-level). AMEND via `classify()` extraction.

### Next move: Stone 241.1.fix BRIEF queued for next-session sonnet spawn

The amend pass has a clear scope (4 L1 amends + 5-6 high-priority L2 amends + 3 runes accepted), an obvious shape (mirror this stone's pattern), and a clean verification (re-cast vigilia; converge L1+L2=0). Per spawn-block winding: Stone 241.2 (migrate A1/A2/A3) blocks on Stone 241.1.fix closure.
