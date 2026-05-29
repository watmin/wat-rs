# DESIGN — Stone 241.1 — mint canonical `parse_argspec_triples` at `src/argspec/`

**Status:** READY (sub-DESIGN). The FOUNDATION; all Phase 1 migrations (241.2/241.3/241.4) compose over this. Mirrors arc 236.0's mint-stone shape — pure additive type-system foundation, NO migration of existing parsers.

## Why this stone

The substrate carries FOUR copies of canonical argspec-parsing logic (per `AUDIT.md`: A1 `parse_fn_signature` runtime.rs:6750 / A2 `parse_fn_signature_for_check` check.rs:15205 / A3 `parse_fn_signature_for_check_diag` check.rs:15258 / A4 `parse_defclause_args` runtime.rs:6880). The duplication runs all the way down to the **error-enum class** (RuntimeError / silenced `()` / CheckError / RuntimeError) — not just message wording. Two parsers in different sites can accept different forms; the substrate accepts what the next binding site silently rejects; LLM co-authors generate code that works in one site and breaks in another.

Per failure-engineering doctrine: **eliminate the class**. State to make unrepresentable: *two binding sites accepting different arg-vector forms*. This stone mints the canonical parser; subsequent stones migrate callers; the class is closed when the four old parsers retire.

## What this stone delivers

A new substrate-internal module at `src/argspec/` containing:

```rust
pub fn parse_argspec_triples(
    args_vec: &[WatAST],
    head: &str,
    form_span: &Span,
    options: ParseOptions,
) -> Result<ArgSpec, ArgSpecError>;

pub struct ArgSpec {
    pub fixed_params: Vec<(String, TypeExpr)>,
    pub rest_param: Option<(String, TypeExpr)>,   // None pre-241.4
    pub ret_type:   Option<TypeExpr>,             // None when ParseOptions.include_ret_type = false
}

pub struct ParseOptions {
    pub include_ret_type: bool,    // fn = true; defclause = false
    pub allow_rest_binder: bool,   // 241.4 only; always false in 241.1
}

pub enum ArgSpecError { /* variants per D-Error below; each carries span: Span */ }
```

A1–A4 remain UNCHANGED. The new parser stands alongside as a callable replacement; 241.2/3/4 wire callers through it.

## The algorithm (parse the canonical triple form)

`parse_argspec_triples(args_vec, head, form_span, options)`:

1. **Iterate `args_vec` items in triple chunks of 3.**
2. For each triple:
   - **slot[0]** — must be `WatAST::Symbol` (the name). Non-Symbol → `ArgSpecError::NameNotSymbol { span, head }`.
   - **slot[1]** — must be a bare `WatAST::Symbol` whose ident equals `"<-"`. Anything else → `ArgSpecError::MissingArrow { span, head }`.
   - **slot[2]** — must be `WatAST::Keyword` (the type). Non-Keyword → `ArgSpecError::TypeNotKeyword { span, head }`. Parse via `parse_type_expr_with_span(kw, &span)` (or equivalent canonical type-keyword parser). Parse failure → wrap as `ArgSpecError::MalformedTypeKeyword { span, inner }`.
   - Push `(name, ty)` to fixed_params.
3. **After fixed param triples**, if `options.include_ret_type`:
   - Expect a bare `WatAST::Symbol` `"->"`. Missing → `ArgSpecError::MissingRetArrow { span, head }`.
   - Expect a `WatAST::Keyword` (the ret type). Non-Keyword → `ArgSpecError::RetTypeNotKeyword { span, head }`. Parse → ret_type.
4. **If trailing items remain** beyond expectation → `ArgSpecError::TrailingItems { span, head, count }`.
5. **If `args_vec` empty when ret type expected** → `ArgSpecError::IncompleteSignature { span, head }`.

**Rest-binder `&`** is NOT supported in 241.1 (always rejected). Stone 241.4 adds the support; rest_param field exists in the struct so the API surface is stable from 241.1 onward.

## Locked decisions

### D1 — Home location: `src/argspec/` (intueri-verified)

Per intueri cast 2026-05-28 (recorded in `DESIGN.md` § Scope expansion + `FORM-COLLAPSE-NOTES.md`): the directory home is `src/argspec/`. Files inside:

- `src/argspec/mod.rs` — thin: re-exports `ArgSpec`, `ParseOptions`, `ArgSpecError`, `parse_argspec_triples`
- `src/argspec/parse.rs` — `parse_argspec_triples` fn + `ArgSpec` struct + `ParseOptions` struct
- `src/argspec/error.rs` — `ArgSpecError` enum + `From<ArgSpecError>` impls for `RuntimeError`, `CheckError`, `TypeError`

Mirrors `comms/` precedent: directory named for the substrate-internal concept; thin mod.rs; concern-named files inside. Per `feedback_namespaced_home_vigilia_gate` (2026-05-28): the namespaced home commits ONLY after a vigilia cast drives L1+L2 findings to zero on `src/argspec/*` + `tests/probe_arc241_stone1_argspec_canonical.rs`. Vigilia (`~/work/holon/datamancy/vigilia/SKILL.md`) is the aggregator; it spawns the applicable defensive subset in parallel — for this home: intueri + solvere + purgare + struere + sequi + temperare (6 always-apply) + complectens + vocare (test-substrate). SCORE-green is the floor; vigilia-convergence is the bar.

### D2 — Shape: per AUDIT.md (locked there 2026-05-27)

The `ArgSpec` / `ParseOptions` / `ArgSpecError` shapes from AUDIT.md § "Confirmed for the consolidation plan" stand exactly. Stone 241.1 ships them verbatim — they are LOCKED design substrate sonnet mirrors, not invention.

### D3 — Public API surface

Only `parse_argspec_triples` and the three types (`ArgSpec`, `ParseOptions`, `ArgSpecError`) are `pub`. Private internals (helper fns like a chunked-iterator-from-slice) module-internal. Per the dungeon-crawl shape: external callers see ONE entry point; internal decomposition is sonnet's discretion within the home.

### D4 — `name_symbol_only` ParseOption is REJECTED

Per AUDIT.md "Open question": every authoritative site already requires Symbol at the name slot. There is no consumer for a configurable name-slot kind. The canonical contract is **unconditional**: name MUST be Symbol; non-Symbol is ALWAYS `ArgSpecError::NameNotSymbol`. No ParseOption controls this. Per `feedback_refuse_easy_solutions`: don't ship surface for nobody.

### D5 — HARD CUT on migration: ZERO A1–A4 changes in 241.1

Stone 241.1 ships the new parser ONLY. The four existing parsers (A1/A2/A3/A4) remain bit-for-bit unchanged at their current file:line locations. Their callers continue routing through them. The substrate-as-teacher cascade BEGINS at 241.2.

This separation is load-bearing: it lets the new parser's API stabilize (and the integration probe verify behavior) before any migration cascade ripples through ~50 fn-binding-form call sites.

### D6 — Integration probe at `tests/probe_arc241_stone1_argspec_canonical.rs`

The FM 2-bis probe lives at `tests/probe_arc241_stone1_argspec_canonical.rs` (NEW). It exercises `parse_argspec_triples` directly via `crate::argspec::*` imports. Contracts (10):

1. **Empty argspec, no ret type expected** (`include_ret_type: false`) → `ArgSpec { fixed_params: [], rest_param: None, ret_type: None }`
2. **Single fixed param, no ret** (`[x <- :wat::core::i64]`, `include_ret_type: false`) → fixed_params: `[("x", i64)]`; ret_type: None
3. **Multiple fixed params + ret** (`[x <- :i64 y <- :i64 -> :i64]`, `include_ret_type: true`) → all three populated correctly
4. **Ret-only signature** (`[-> :i64]`, `include_ret_type: true`) → empty fixed_params; ret_type populated
5. **Non-Symbol at name slot** (`[42 <- :i64]`) → `Err(ArgSpecError::NameNotSymbol)` with the integer's span
6. **Missing `<-` arrow** (`[x : :i64]`) → `Err(ArgSpecError::MissingArrow)` with the offending element's span
7. **Non-Keyword at type slot** (`[x <- "i64"]`) → `Err(ArgSpecError::TypeNotKeyword)` with the offending element's span
8. **Missing `->` when ret expected** (`[x <- :i64]`, `include_ret_type: true`) → `Err(ArgSpecError::MissingRetArrow)`
9. **Trailing items after ret** (`[x <- :i64 -> :i64 garbage]`, `include_ret_type: true`) → `Err(ArgSpecError::TrailingItems { count: 1 })`
10. **`&` rest-marker present** (any form with `&` inside) → `Err(ArgSpecError::RestBinderNotSupported)` — 241.1 explicitly rejects; 241.4 extends to accept

Pre-stone: ALL 10 fail to compile because `crate::argspec` doesn't exist. The probe IS the contract sonnet satisfies; post-stone: 10/10 PASS.

Per `feedback_assertion_demands_evidence` + dungeon-crawl Phase 2: the probe commits BEFORE the BRIEF and is part of the room-map sonnet mirrors, not assertion.

### D7 — `lib.rs` touch: ONE line addition

`pub mod argspec;` added to `src/lib.rs` (mirror existing pattern: `pub mod comms;` at line 62). This is the ONLY change outside `src/argspec/*` and `tests/`. STOP triggers (below) hard-bound the blast radius to this line + new home + new probe.

### D8 — Module-level doctrine inscription in `mod.rs`

Module-level doc comment on `src/argspec/mod.rs` explaining:
- **WHY**: the four-parser duplication failure class being eliminated
- **What this owns**: canonical parsing of the `[name <- :T ... [-> :Ret]]` argspec form
- **What it does NOT own**: form-shape parsing (def/defn/defstruct/defenum each parse their own form-level shape; only the argspec-triples region routes through here)
- **The migration plan**: 241.2/3/4 wire A1–A4 callers through here; 241.1 ships the parser alongside

This doc IS the doctrine inscription for the consolidation; future maintainers grep `pub mod argspec` in lib.rs, land on the home, and understand the substrate-architectural decision.

### D9 — No clippy warnings introduced

Keep at current baseline (per CLIFFNOTES Currently: ~54 clippy warnings). Type design must not introduce new lints. New code lints to 0.

### D10 — No regression of any existing test

Pure additive substrate work. Lib baseline (per CLIFFNOTES Currently: 827 / 0 PASS / FAIL) preserved. All arc 237 probes stay green. Workspace test-build green. NO behavior changes outside the new files + the one-line lib.rs addition.

---

## Trap-door audit

### T1 — `ArgSpecError`'s diverse-error-class problem (the AUDIT finding)

The same structural failure ("name slot is not a Symbol") produces THREE different error enum variants across the existing parsers: A1+A4 → `RuntimeError::MalformedForm`; A2 → silenced `()`; A3 → `CheckError::MalformedForm`; B-family → `TypeError::MalformedDecl`. The canonical home solves this with ONE error enum at its boundary + `From<>` impls at the call boundary.

`src/argspec/error.rs` provides:
- `impl From<ArgSpecError> for RuntimeError` — A1/A4 callers convert at site
- `impl From<ArgSpecError> for CheckError` — A3 callers convert at site (A2's silent-`()` mode handled by `or_propagate_errors`-style adapter when 241.2 migrates A2; not 241.1's concern)
- `impl From<ArgSpecError> for TypeError` — for the eventual struct/enum migration (241.7/8 territory; 241.1 ships the impl as forward-compatible substrate)

### T2 — `lib.rs` addition is unavoidable

Per D7. The one-line `pub mod argspec;` addition is the ONLY out-of-home Rust touch. Sonnet must NOT touch anything else in lib.rs.

### T3 — `ArgSpec: Clone` bound

`ArgSpec` derives `Clone` and `Debug`. Downstream consumers (241.2/3/4 + future call sites) may need both for adapter shims during migration. Keep it standard-derivable: `Vec<(String, TypeExpr)>` + `Option<(String, TypeExpr)>` + `Option<TypeExpr>` are all Clone when TypeExpr is. Verify TypeExpr's Clone derivation (it already derives — `src/types.rs:67`).

### T4 — `&[WatAST]` input is idiomatic

Slice (`&[WatAST]`) is cheaper than `Vec<WatAST>` and matches Rust idiom for parser inputs. Callers pass the args slice directly; no allocation.

### T5 — `Span` carrier

`Span` is the substrate's universal location type (`src/span.rs`). Every `ArgSpecError` variant carries `span: Span` (per AUDIT.md line 161). Sonnet picks span propagation per offending element (e.g., the non-Symbol's `WatAST::span()` for `NameNotSymbol`), not the form-level span. This matches arc 138's per-element error-location discipline.

### T6 — Module visibility (mod.rs / parse.rs / error.rs)

`mod.rs` re-exports the public surface with `pub use parse::{ArgSpec, ParseOptions, parse_argspec_triples};` and `pub use error::ArgSpecError;`. Internal helpers in `parse.rs` (chunk iterator, validation predicates) stay `pub(super)` or private. No public surface leaks beyond what `mod.rs` declares.

### T7 — `TypeExpr` import

`use crate::types::TypeExpr;` in `parse.rs`. `parse_type_expr_with_span` (or equivalent canonical type-keyword parser) imported similarly — find the canonical one via grep (sonnet's discretion; multiple variants exist per AUDIT.md "Type slot kind" row; pick the one with span-carrying error reporting).

### T8 — `ArgSpecError` variants carry Span uniformly

Per AUDIT.md line 161: every variant carries `span: Span`. Even variants that conceptually have no specific offending element (e.g., `TrailingItems`) carry the form_span as fallback so error reporting always points at the user's source.

---

## STOP triggers (REJECTION — not permission to defer)

1. **STOP-1** — Unexpected compile errors not traced to a probe-named contract
2. **STOP-2** — Lib baseline regression (current: 827 PASS / 0 FAIL; baseline must hold)
3. **STOP-3** — 60 min elapsed (small focused mint-stone; per arc 236.0 calibration this is the upper bound)
4. **STOP-4** — `holon-rs` touched (substrate is frozen)
5. **STOP-5** — Rust files outside `src/argspec/*` + `src/lib.rs` (ONE line addition) + `tests/probe_arc241_stone1_argspec_canonical.rs` touched
6. **STOP-6** — Scope creep:
   - Migrating ANY of A1/A2/A3/A4 — that is 241.2/3
   - Implementing `&` rest-binder — that is 241.4
   - Adding the `name_symbol_only` ParseOption — REJECTED per D4
   - Adding any other ParseOption beyond the two locked
7. **STOP-7** — Probe doesn't reach 10/10 PASS
8. **STOP-8** — Any prior arc 237 probe regresses (237.1–237.8a tests stay green)
9. **STOP-9** — Clippy warnings increase above baseline
10. **STOP-10** — Sonnet wants to mint a new `TypeExpr` variant, a new `Value` variant, or any parallel registry — STOP; `TypeExpr` and existing types are sufficient (canonical parser is pure parsing, no new type-system concepts)

Each STOP is REJECTION criteria: ship NOTHING when hit; surface as finding.

---

## FM 2-bis evidence

Probe at `tests/probe_arc241_stone1_argspec_canonical.rs` (committed BEFORE the BRIEF, as design substrate). Pre-stone: all 10 contracts fail to compile because `crate::argspec` doesn't exist — the failure is at module-resolution time, isolated to the missing home. No adjacent surfaces fail; the gap is unambiguous.

Post-stone: 10/10 PASS.

This is the disconfirming-probe shape per `COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis: the probe attempts the composition with minimal scaffolding, fails on EXACTLY the gap, and proves the canonical parser is reachable and correct once minted.

---

## SCORE doc spec

`docs/arc/2026/05/241-function-signature-unification/SCORE-STONE-241.1.md` (NEW). 10-row scorecard verbatim (probe contracts 1–10) + final API signatures + line counts per file + clippy delta + lib baseline confirmation + honest deltas + cascade depth (0 expected — pure additive). Mirror `SCORE-STONE-236.0.md` structural shape per `feedback_stone_briefs_cite_prior_score`.

---

## Calibration

**Target band:** 30–50 min Mode A.
**Upper bound:** 60 min (STOP-3).

**Surface estimate (~200–300 lines net):**
- `src/argspec/mod.rs` — ~25 lines (module doc + re-exports)
- `src/argspec/parse.rs` — ~140 lines (ArgSpec/ParseOptions + parser fn + private helpers)
- `src/argspec/error.rs` — ~80 lines (enum + Display + From impls × 3)
- `src/lib.rs` — 1 line (`pub mod argspec;`)
- `tests/probe_arc241_stone1_argspec_canonical.rs` — ~120 lines (10 contracts)

**Confidence: HIGH.** Pure additive type-system foundation; well-bounded API surface; no migration cascade; AUDIT.md provides the locked types verbatim; mirror arc 236.0's mint-stone discipline.

**Per `feedback_stone_briefs_cite_prior_score`:** mirror `SCORE-STONE-236.0.md` (the closest mint-stone precedent — pure additive type-system foundation, 80–150 lines surface, no migration). BRIEF cites it verbatim as the structural shape sonnet copies.

---

## What this unblocks

Stone 241.2 — migrate the three fn-parser variants (A1 + A2 + A3) to route through `parse_argspec_triples`. The canonical parser EXISTS; the substrate-as-teacher cascade can begin.

Beyond Phase 1: 241.4's `&` rest-binder extension lands cleanly on the settled API (rest_param field exists from 241.1; the field just becomes populated when 241.4 ships the parser logic).

---

## Cross-references

- `AUDIT.md` — verified parser-site inventory; the locked ArgSpec/ParseOptions/ArgSpecError shapes
- `DESIGN.md` § Scope expansion 2026-05-28 — the arc-level framing
- `FORM-COLLAPSE-NOTES.md` — intueri verdicts for `src/argspec/` home + the form-collapse dialogue
- `docs/arc/2026/05/236-check-result-class-elimination/DESIGN-STONE-236.0.md` — the mint-stone precedent this stone mirrors (pure additive type-system foundation)
- `docs/arc/2026/05/236-check-result-class-elimination/SCORE-STONE-236.0.md` — the SCORE shape sonnet mirrors
- `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-237.5.md` — the recursive-walker pattern (different shape; useful for parser fn discipline reference)
- `feedback_sonnet_writes_substrate` — orchestrator briefs/scores; sonnet writes the Rust
- `feedback_refuse_easy_solutions` — drove D4's REJECTION of `name_symbol_only` ParseOption
- `feedback_namespaced_home_vigilia_gate` — vigilia-convergence (L1+L2=0) gates commit on `src/argspec/*`; the applicable defensive set for this home is intueri + solvere + purgare + struere + sequi + temperare + complectens + vocare (8 spells; vigilia aggregates)
- `feedback_ward_zone_comms_only` — scoped to comms/; extended by the gate doctrine to argspec/ + future namespaced homes
- `feedback_stone_briefs_cite_prior_score` — BRIEF cites prior comparable SCORE for shape
- `COMPACTION-AMNESIA-RECOVERY.md` § FM 2-bis — the disconfirming-probe discipline this stone's probe follows
