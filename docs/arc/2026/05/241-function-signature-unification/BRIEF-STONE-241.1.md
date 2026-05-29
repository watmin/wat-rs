# BRIEF — Stone 241.1 — mint canonical `parse_argspec_triples` at `src/argspec/`

**Status:** READY TO SPAWN. `model: "sonnet"`.

## What to do

Mint the canonical argspec parser at the new substrate-internal home **`src/argspec/`**. Three files: `mod.rs` (thin re-exports), `parse.rs` (parser fn + `ArgSpec` + `ParseOptions`), `error.rs` (`ArgSpecError` enum + three `From<>` impls). One line added to `src/lib.rs`. Make the FM 2-bis probe (10 contracts at `tests/probe_arc241_stone1_argspec_canonical.rs`, committed `e0d1d054`) go 10/10 PASS. **NO migration of any existing parser (A1/A2/A3/A4); those stand untouched — 241.2/241.3 migrate them.**

NOT new mechanism territory — this composes pieces that all already exist (verified):
- `WatAST::Vector / Symbol / Keyword` and `Span` are existing AST primitives
- `TypeExpr` + canonical type-keyword parsing (`parse_type_expr_with_span` etc.) are existing types-module surface
- The four old parsers (A1/A2/A3/A4) are inventoried in `AUDIT.md` with their per-site invariants

The work is the parser walker + the locked types from AUDIT verbatim + module organization + `From<>` conversion impls.

## Read in order

1. `docs/arc/2026/05/241-function-signature-unification/DESIGN-STONE-241.1.md` — sub-DESIGN: the algorithm, the trap-door audit T1–T8, the 10 locked decisions D1–D10. Authoritative on every shape.
2. `tests/probe_arc241_stone1_argspec_canonical.rs` — **LOAD-BEARING** 10 contracts; ALL must PASS. This is the contract you satisfy. (Pre-stone: 1 compile error — `unresolved import wat::argspec`. That absence is the only gap. Lib baseline 834/0 PASS at HEAD.)
3. `docs/arc/2026/05/241-function-signature-unification/AUDIT.md` § "Confirmed for the consolidation plan" — the `ArgSpec` / `ParseOptions` / `ArgSpecError` shapes locked verbatim. Ship them exactly.
4. `src/comms/mod.rs` — module organization PRECEDENT. Thin re-exports at the top; concern-named children below. Mirror this discipline at `src/argspec/mod.rs`.
5. `src/runtime.rs:6750` (`parse_fn_signature`) — A1 reference implementation. Existing canonical-triple walker; the algorithm and slot validation are the model. **Do not migrate A1 here** — read only.
6. `src/runtime.rs:6880` (`parse_defclause_args`) — A4 reference implementation. Same algorithm without ret-type; demonstrates how `include_ret_type: false` should behave.
7. `src/types.rs:67` — `enum TypeExpr` definition; what the type slot parses INTO.
8. `src/types.rs:2683` (`parse_type_expr_with_span`) — the canonical span-carrying type-keyword parser to wrap. Use this for type-slot parsing; the error wraps as `ArgSpecError::MalformedTypeKeyword`.
9. `src/span.rs` — `Span` type. Public; used by every error variant.
10. `src/lib.rs:62` (`pub mod comms;`) — the location pattern for the `pub mod argspec;` addition.
11. `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.0.md` — the structural shape for `SCORE-STONE-241.1.md`. **Mirror it verbatim** (per `feedback_stone_briefs_cite_prior_score`): scorecard row format, line-counts-per-file table, honest deltas section.

## Implementation sketch

### `src/argspec/mod.rs` (~25 lines)

```rust
//! # Argspec — canonical parser for the `[name <- :T name <- :T ... [-> :Ret]]` triple form.
//!
//! This module owns the ONE canonical parser for the canonical argspec form used at
//! every binding site (defn, defclause, defstruct fields, defenum tagged-variant
//! fields). It eliminates the failure class of parser divergence across binding sites
//! by structurally consolidating four prior duplicated parsers (per
//! `docs/arc/2026/05/241-function-signature-unification/AUDIT.md`).
//!
//! Per-site invariants (e.g. "include the ret-type slot") live in `ParseOptions`.
//! Callers convert at their boundary via `From<ArgSpecError>` for the per-call-site
//! error class (`RuntimeError`, `CheckError`, `TypeError`).
//!
//! Migration plan: Stones 241.2/241.3/241.4 route the four prior parsers
//! (parse_fn_signature × 3 + parse_defclause_args) through here; this stone (241.1)
//! ships the canonical parser ALONGSIDE the old ones.

mod parse;
mod error;

pub use parse::{parse_argspec_triples, ArgSpec, ParseOptions};
pub use error::ArgSpecError;
```

### `src/argspec/parse.rs` (~140 lines)

```rust
use crate::ast::WatAST;
use crate::span::Span;
use crate::types::{TypeExpr, parse_type_expr_with_span};
use super::error::ArgSpecError;

/// Result of parsing a canonical argspec.
#[derive(Debug, Clone)]
pub struct ArgSpec {
    pub fixed_params: Vec<(String, TypeExpr)>,
    pub rest_param: Option<(String, TypeExpr)>,   // None pre-241.4
    pub ret_type:   Option<TypeExpr>,             // None when ParseOptions.include_ret_type = false
}

/// Per-site invariants. Empty struct could be Default'd; kept explicit for clarity.
#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    pub include_ret_type: bool,    // fn = true; defclause = false
    pub allow_rest_binder: bool,   // 241.4 only; always false in 241.1
}

/// Canonical parser for `[name <- :T name <- :T ... [-> :Ret]]`.
///
/// `args_vec` is the inner items of a WatAST::Vector at the binding site.
/// `head` is the surface form name (e.g. ":wat::core::defn") for error context.
/// `form_span` is the Vector's own span; used as fallback in errors when no
/// offending element provides a more specific span.
pub fn parse_argspec_triples(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
    options: ParseOptions,
) -> Result<ArgSpec, ArgSpecError> {
    let mut idx = 0usize;
    let mut fixed_params: Vec<(String, TypeExpr)> = Vec::new();

    // Walk triples (name <- :T) until we hit -> (if include_ret_type) or end.
    while idx < args_vec.len() {
        // Check for `&` (rest-marker) at this position
        if is_bare_symbol(&args_vec[idx], "&") {
            if !options.allow_rest_binder {
                return Err(ArgSpecError::RestBinderNotSupported {
                    span: args_vec[idx].span().clone(),
                    head: head.to_string(),
                });
            }
            // 241.4 will implement rest-binder parsing here; 241.1 rejects above.
            unreachable!("allow_rest_binder=false in 241.1");
        }
        // Check for `->` (ret-arrow) — if so, stop param parsing
        if is_bare_symbol(&args_vec[idx], "->") {
            break;
        }
        // Need 3 items for a triple
        if idx + 2 >= args_vec.len() {
            return Err(ArgSpecError::IncompleteSignature {
                span: form_span.clone(),
                head: head.to_string(),
            });
        }
        // Slot 0: name (must be Symbol)
        let name = match &args_vec[idx] {
            WatAST::Symbol(ident, _) => ident.name.clone(),
            other => return Err(ArgSpecError::NameNotSymbol {
                span: other.span().clone(),
                head: head.to_string(),
            }),
        };
        // Slot 1: `<-` arrow (must be bare Symbol "<-")
        if !is_bare_symbol(&args_vec[idx + 1], "<-") {
            return Err(ArgSpecError::MissingArrow {
                span: args_vec[idx + 1].span().clone(),
                head: head.to_string(),
            });
        }
        // Slot 2: type (must be Keyword; parse via parse_type_expr_with_span)
        let ty = match &args_vec[idx + 2] {
            WatAST::Keyword(kw, kw_span) => {
                parse_type_expr_with_span(kw, kw_span)
                    .map_err(|inner| ArgSpecError::MalformedTypeKeyword {
                        span: kw_span.clone(),
                        head: head.to_string(),
                        inner: Box::new(inner),
                    })?
            }
            other => return Err(ArgSpecError::TypeNotKeyword {
                span: other.span().clone(),
                head: head.to_string(),
            }),
        };
        fixed_params.push((name, ty));
        idx += 3;
    }

    // Handle ret-type slot if expected
    let ret_type = if options.include_ret_type {
        if idx >= args_vec.len() {
            return Err(ArgSpecError::MissingRetArrow {
                span: form_span.clone(),
                head: head.to_string(),
            });
        }
        if !is_bare_symbol(&args_vec[idx], "->") {
            return Err(ArgSpecError::MissingRetArrow {
                span: args_vec[idx].span().clone(),
                head: head.to_string(),
            });
        }
        idx += 1;
        if idx >= args_vec.len() {
            return Err(ArgSpecError::RetTypeNotKeyword {
                span: form_span.clone(),
                head: head.to_string(),
            });
        }
        let ret = match &args_vec[idx] {
            WatAST::Keyword(kw, kw_span) => {
                parse_type_expr_with_span(kw, kw_span)
                    .map_err(|inner| ArgSpecError::MalformedTypeKeyword {
                        span: kw_span.clone(),
                        head: head.to_string(),
                        inner: Box::new(inner),
                    })?
            }
            other => return Err(ArgSpecError::RetTypeNotKeyword {
                span: other.span().clone(),
                head: head.to_string(),
            }),
        };
        idx += 1;
        Some(ret)
    } else {
        None
    };

    // Trailing items rejection
    if idx < args_vec.len() {
        return Err(ArgSpecError::TrailingItems {
            span: form_span.clone(),
            head: head.to_string(),
            count: args_vec.len() - idx,
        });
    }

    Ok(ArgSpec {
        fixed_params,
        rest_param: None,    // 241.1: rest unsupported; 241.4 extends
        ret_type,
    })
}

/// Helper: is the AST a bare Symbol with the given ident name?
fn is_bare_symbol(ast: &WatAST, name: &str) -> bool {
    matches!(ast, WatAST::Symbol(ident, _) if ident.name == name)
}
```

### `src/argspec/error.rs` (~80 lines)

```rust
use crate::span::Span;
use crate::types::TypeError;

/// Sum of failure modes for canonical argspec parsing.
/// Each variant carries `span: Span` (per AUDIT.md line 161).
#[derive(Debug, Clone)]
pub enum ArgSpecError {
    /// Slot 0 of a triple was not a Symbol.
    NameNotSymbol { span: Span, head: String },
    /// Slot 1 of a triple was not bare Symbol "<-".
    MissingArrow { span: Span, head: String },
    /// Slot 2 of a triple was not a Keyword.
    TypeNotKeyword { span: Span, head: String },
    /// Wrapped error from parse_type_expr_with_span on the type keyword.
    MalformedTypeKeyword { span: Span, head: String, inner: Box<TypeError> },
    /// include_ret_type=true but no "->" found after final triple.
    MissingRetArrow { span: Span, head: String },
    /// "->" found but next slot was not a Keyword.
    RetTypeNotKeyword { span: Span, head: String },
    /// Trailing items beyond expected end.
    TrailingItems { span: Span, head: String, count: usize },
    /// Triple incomplete at end of args_vec.
    IncompleteSignature { span: Span, head: String },
    /// `&` rest-binder marker present but allow_rest_binder=false.
    RestBinderNotSupported { span: Span, head: String },
}

// Per AUDIT.md § "Recommendation for 241.1": From<> impls let callers convert at
// their site boundary. These wire the canonical error into each call-site's
// native error class without duplicating shape inside the parser.

impl From<ArgSpecError> for crate::runtime::RuntimeError {
    fn from(err: ArgSpecError) -> Self {
        // Pattern: wrap as MalformedForm carrying head + reason + span.
        // Sonnet picks the exact field shape from existing RuntimeError::MalformedForm
        // construction (e.g. src/runtime.rs:6750 area shows the pattern).
        // Concrete impl: extract span + head + describe_reason(&err) per variant.
        todo!("wire to RuntimeError::MalformedForm; mirror parse_fn_signature error shape")
    }
}

impl From<ArgSpecError> for crate::check::CheckError {
    fn from(err: ArgSpecError) -> Self {
        // Pattern: CheckError::MalformedForm or equivalent variant.
        todo!("wire to CheckError; mirror parse_fn_signature_for_check_diag error shape")
    }
}

impl From<ArgSpecError> for crate::types::TypeError {
    fn from(err: ArgSpecError) -> Self {
        // Pattern: TypeError::MalformedDecl or equivalent.
        todo!("wire to TypeError::MalformedDecl; mirror parse_struct error shape")
    }
}
```

**Note on `todo!()`**: those three `From<>` impls are FORWARD-COMPATIBLE substrate for 241.2/3/7. They MUST be implemented (not `todo!()`-panicking) since 241.1's probe never triggers them. **For 241.1**: implement them as `unimplemented!()` body OR fully wire to the existing error variants — sonnet's discretion. The probe's 10 contracts don't exercise these From impls. The discipline calls for full implementation; the calibration band allows it; do not ship with `todo!()`.

### `src/lib.rs` (1 line addition)

Add `pub mod argspec;` adjacent to `pub mod comms;` at line 62.

## Discipline

- Modify `src/argspec/*` (NEW) + `src/lib.rs` (ONE line) + the probe file (already committed) ONLY.
- NO new `Value` variant. NO new `TypeExpr` variant. NO parallel registries.
- NO holon-rs (STOP-4).
- DO NOT migrate A1/A2/A3/A4 — that is 241.2 / 241.3.
- DO NOT implement `&` rest-binder logic — 241.1 returns `ArgSpecError::RestBinderNotSupported` when encountered.
- DO NOT add the `name_symbol_only` ParseOption — REJECTED per D4 (canonical contract is unconditional).
- Module-level doc on `mod.rs` IS the doctrine inscription; write it.

## STOP triggers (REJECTION — not permission to defer)

1. **STOP-1** — Unexpected compile errors not traced to a probe-named contract.
2. **STOP-2** — Lib baseline drops below **834**.
3. **STOP-3** — **60 min** elapsed (per arc 236.0 mint-stone upper bound).
4. **STOP-4** — `holon-rs` touched.
5. **STOP-5** — Rust files outside `src/argspec/*` + `src/lib.rs` (ONE line) touched.
6. **STOP-6** — Scope creep:
   - Migrating ANY of A1/A2/A3/A4 — that is 241.2 / 241.3
   - Implementing `&` rest-binder LOGIC — that is 241.4 (rejection-only in 241.1)
   - Adding `name_symbol_only` ParseOption (REJECTED per D4)
   - Adding any other ParseOption beyond the two locked
7. **STOP-7** — Probe doesn't reach **10/10 PASS**.
8. **STOP-8** — ANY prior arc 237 probe regresses (237.1–237.8a tests stay green).
9. **STOP-9** — Clippy regression above baseline (currently ~54).
10. **STOP-10** — You find yourself wanting a new `TypeExpr` variant, new `Value` variant, or parallel registry — STOP; existing types are sufficient.

Each STOP is REJECTION criteria: ship NOTHING when hit; surface as finding.

## FM 2-bis evidence

Probe at `tests/probe_arc241_stone1_argspec_canonical.rs` committed `e0d1d054`. Pre-stone state verified:
- **1 compile error**: `error[E0432]: unresolved import wat::argspec` at line 18
- Isolated to module-resolution; all other imports + helpers + test bodies compile cleanly
- Lib baseline 834/0 PASS at HEAD

Post-stone: 10/10 PASS.

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.md` (NEW). Mirror `SCORE-STONE-236.0.md` structural shape verbatim:

- 10-row scorecard (probe contracts 1–10) — each row: `contract_NN — <description> — PASS/FAIL — <verification command>`
- Final API signatures (the four `pub use` items + `parse_argspec_triples` full signature)
- Line counts per file (`mod.rs`, `parse.rs`, `error.rs`, lib.rs delta, probe delta)
- Clippy delta (must be 0 above baseline)
- Lib baseline confirmation (834 holds)
- Cascade depth: 0 (pure additive; no callers migrated)
- Honest deltas (any in-flight discovery worth surfacing)

## Calibration

**Target band: 30–50 min Mode A.**
**Upper bound: 60 min (STOP-3).**

Surface estimate (~200–300 lines net):
- `src/argspec/mod.rs` — ~25 lines (module doc + re-exports)
- `src/argspec/parse.rs` — ~140 lines (ArgSpec/ParseOptions + parser fn + private helper)
- `src/argspec/error.rs` — ~80 lines (enum + 3 From impls)
- `src/lib.rs` — 1 line
- (probe committed `e0d1d054`; not counted in stone deliverables)

**Per `feedback_stone_briefs_cite_prior_score`:** mirror `SCORE-STONE-236.0.md` shape exactly (Stone 236.0 was the closest precedent — pure additive type-system foundation; 80–150 lines surface; no migration cascade; 25-min ship). 241.1 is structurally identical but at higher line count due to the parser's grammar walking + the three error conversions.

**Confidence: HIGH.** AUDIT.md locks the types verbatim; the algorithm is fully specified in sub-DESIGN; the probe is the contract; the comms/ precedent is the module-organization model.

Agent prompt: vanilla `cargo test --release --test probe_arc241_stone1_argspec_canonical` for verification; vanilla `cargo build --release --tests --workspace` for compile check; vanilla `cargo clippy --release` for lint baseline. One per line. No tool-availability preamble (FM 16).
