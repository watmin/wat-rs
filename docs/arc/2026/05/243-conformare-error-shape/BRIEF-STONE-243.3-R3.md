# BRIEF — Stone 243.3 R3 sweep — close remaining 6 R2 vigilia findings

You are sonnet. Stone 243.3 R3 sweep. 6 mechanical fixes closing the remaining R2 vigilia findings. Phase A (TypeError Pattern A retrofit) already SHIPPED in checkpoint `48d3393e`. R1 (6/6) + R3 partial (6/12 landed-or-reversed) also in that checkpoint. **This sweep closes the remaining 6.** After this lands, R2 vigilia re-casts (orchestrator-cast) and SCORE Phase B authors before the atomic commit.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## Critical doctrine (pre-authorized — read BEFORE strike)

1. **NO runes for these 6 fixes** — all are solvable + perf-OK per `feedback_runes_illegal_when_solvable`. Runes are EXCEPTION mechanisms (unsolvable paths OR perf-impairing fixes only). The 6 here are mechanical structural refactors + redundant-comment removals — every one closes via FIX.
2. **NO deferral language** anywhere new (`feedback_dont_document_non_fixes` + exigere spell). Phrases like "future arc", "outside scope", "would require", "intentionally", "to be added" are REJECTION triggers.
3. **NO "skip pre-existing"** framing (`feedback_pre_existing_is_not_exemption`). If your work surfaces ADDITIONAL findings in touched files, report them honestly — DO NOT silently skip them as "pre-existing." Orchestrator triages via rune-vs-FIX; you do not.
4. **HARD CUT** removals (the bool flag retires; the redundant comments delete) — no backward-compat shims, no deprecated `// removed` markers.
5. **Sonnet writes substrate** (`feedback_sonnet_writes_substrate`) — you do the Rust edits.
6. **holon-rs NEVER touched** (STOP-5).
7. **DO NOT write to INTERSTITIAL** (`feedback_sonnet_never_drafts_interstitial`).
8. **DO NOT commit** — orchestrator commits atomic after R2 vigilia + SCORE Phase B.
9. **DO NOT cast vigilia or conformare** — orchestrator-cast post-sweep.

## Pre-spawn baseline (verified at HEAD `93760ecc`; substrate state at checkpoint `48d3393e`)

- Lib: **890 PASS / 0 FAIL**
- tests/function: **8 / 0**
- probe_arc243_stone3_typeerror_pattern_a: **3 / 0**
- Workspace test-build: clean (exit 0)
- Clippy: **897**

All gates must hold post-strike. Lib + tests/function + probe_arc243_stone3 are LOAD-BEARING — any regression is a STOP trigger.

## The 6 fixes — disk-state cited

### R3.1 — `RegistrationPrivilege` enum (src/types.rs:280-334)

Replace the `bypass_prefix_gate: bool` parameter of `register_validated` with a named enum.

**Site (verified at HEAD):**
- `register_validated` at line 327 takes `bypass_prefix_gate: bool` (line 331); body at line 334 reads `if !bypass_prefix_gate && ...`
- Call site 1: line 293 (`register_with_span`) passes `false`
- Call site 2: line 320 (`register_stdlib_with_span`) passes `true`

**Target shape:**
```rust
/// Distinguishes user-source registration (subject to reserved-prefix gate)
/// from stdlib registration (privileged to register `:wat::*` directly).
enum RegistrationPrivilege {
    User,
    Stdlib,
}

fn register_validated(
    &mut self,
    def: TypeDef,
    span: Span,
    privilege: RegistrationPrivilege,
) -> Result<(), TypeError> {
    let name = def.name().to_string();
    if matches!(privilege, RegistrationPrivilege::User)
        && crate::resolve::is_reserved_prefix(&name)
    {
        return Err(TypeError { span, kind: TypeErrorKind::ReservedPrefix { name } });
    }
    // ... rest unchanged ...
}
```

Call site updates:
- Line 293: `self.register_validated(def, span, false)` → `self.register_validated(def, span, RegistrationPrivilege::User)`
- Line 320: `self.register_validated(def, span, true)` → `self.register_validated(def, span, RegistrationPrivilege::Stdlib)`

Enum is module-private (no `pub`); does not escape `src/types.rs`.

**WHY (do not inscribe in code; this is for your understanding):** bool flags at call sites read as anonymous magic; the named enum makes the privilege axis self-documenting. Struere-class fix.

### R3.2 — `splice_type_decls` extraction (src/types.rs:1742-1858)

`splice_type_decls_user` (line 1742) and `splice_type_decls_stdlib` (line 1805) are near-identical functions differing ONLY in which `env` method gets called (`register_with_span` vs `register_stdlib_with_span`). Extract a generic helper threaded by closure; both public wrappers become thin shells.

**Target shape:**
```rust
fn splice_type_decls<F>(
    form: WatAST,
    env: &mut TypeEnv,
    register: &F,
) -> Result<WatAST, TypeError>
where
    F: Fn(&mut TypeEnv, TypeDef, Span) -> Result<(), TypeError>,
{
    let (items, span) = match form {
        WatAST::List(items, span) => (items, span),
        other => return Ok(other),
    };
    let head_kw = match items.first() {
        Some(WatAST::Keyword(k, _)) => k.as_str(),
        _ => return Ok(WatAST::List(items, span)),
    };
    match head_kw {
        ":wat::core::do" => {
            let mut new_children = Vec::with_capacity(items.len());
            let mut iter = items.into_iter();
            new_children.push(iter.next().expect("do has keyword"));
            for child in iter {
                match classify_type_decl(&child) {
                    Some(head) => {
                        let decl_span = child.span().clone();
                        let def = parse_type_decl(head, child, decl_span.clone())?;
                        register(env, def, decl_span)?;
                    }
                    None => {
                        new_children.push(splice_type_decls(child, env, register)?);
                    }
                }
            }
            Ok(WatAST::List(new_children, span))
        }
        ":wat::core::let" => {
            let mut new_children = Vec::with_capacity(items.len());
            let mut iter = items.into_iter();
            new_children.push(iter.next().expect("let has keyword"));
            if let Some(bindings) = iter.next() {
                new_children.push(bindings);
            }
            for child in iter {
                match classify_type_decl(&child) {
                    Some(head) => {
                        let decl_span = child.span().clone();
                        let def = parse_type_decl(head, child, decl_span.clone())?;
                        register(env, def, decl_span)?;
                    }
                    None => {
                        new_children.push(splice_type_decls(child, env, register)?);
                    }
                }
            }
            Ok(WatAST::List(new_children, span))
        }
        _ => Ok(WatAST::List(items, span)),
    }
}

fn splice_type_decls_user(form: WatAST, env: &mut TypeEnv) -> Result<WatAST, TypeError> {
    splice_type_decls(form, env, &|env, def, span| env.register_with_span(def, span))
}

fn splice_type_decls_stdlib(form: WatAST, env: &mut TypeEnv) -> Result<WatAST, TypeError> {
    splice_type_decls(form, env, &|env, def, span| env.register_stdlib_with_span(def, span))
}
```

The `// Mirrors the splice-recursion pattern in preregister_fn_defs_in_do` comment at types.rs:1740 STAYS (it's present-state observation about a sibling pattern). The user/stdlib wrappers are thin enough to omit doc comments.

**WHY:** solvere — single source of truth eliminates the divergence risk between two near-identical functions.

### R3.3 — defclause helper extraction (src/check.rs:10043-10121)

The `:wat::core::defclause` arm of `collect_splice_defs_ctx` (lines 10043-10069) and the body of `preregister_defclause_in_env` (lines 10087-10121) share the same shape: parse the form via `parse_defclause_form`, build a `Vec<(Vec<TypeExpr>, TypeExpr, bool)>`, call `env.register_defclause(name, clauses, span)`.

**Target shape:**
```rust
/// Stone 237.2/237.3 — shared defclause registration: parse a top-level
/// `(:wat::core::defclause ...)` form and register its clause table into
/// `env.defclause_registrations`. Returns true when registered, false when
/// parse fails or the name is already registered (when `idempotent=true`).
fn register_defclause_from_form(
    form: &WatAST,
    env: &mut CheckEnv,
    idempotent: bool,
) -> bool {
    let span = form.span().clone();
    let (name, cs) = match crate::runtime::parse_defclause_form(form) {
        Ok(pair) => pair,
        Err(_) => return false,
    };
    if idempotent && env.get_defclause_clauses(&name).is_some() {
        return false;
    }
    let clauses: Vec<(Vec<TypeExpr>, TypeExpr, bool)> = cs
        .clauses
        .iter()
        .map(|cl| {
            let arg_types: Vec<TypeExpr> =
                cl.args.iter().map(|(_, ty)| ty.clone()).collect();
            (arg_types, cl.return_type.clone(), cl.rest_param.is_some())
        })
        .collect();
    env.register_defclause(name, clauses, span);
    true
}
```

**Caller updates:**
- `collect_splice_defs_ctx` arm at line 10048: replace the body with `register_defclause_from_form(form, env, false);` (non-idempotent — overwrites are silently allowed there per pre-existing semantic; verify by reading the surrounding comment about "emitted by infer_def; don't overwrite"). Match exact pre-existing semantics — if the original DOES skip duplicates, pass `idempotent: true`; verify by reading.
- `preregister_defclause_in_env` (lines 10096-10121): replace body with `register_defclause_from_form(form, env, true);` (idempotent — line 10108's `if env.get_defclause_clauses(&name).is_none()` confirms).

**Trap-door T1:** verify that `collect_splice_defs_ctx` callers' EXISTING semantics around register-vs-skip match the helper's `idempotent` parameter. If divergent, pick the safer (idempotent=true) and surface to orchestrator as honest delta.

**WHY:** solvere — two sites carrying near-identical clause-building logic; helper extraction eliminates drift risk.

### R3.7 — drop redundant body comment in `parse_type_expr` (src/types.rs)

Inspect `parse_type_expr` (defined around line 2863). The rune block at lines 2859-2862:

```rust
// rune:struere(host-constraint) — public surface preserved for callers
// without a keyword span in scope (arc 138 lineage); Span::unknown() is
// the honest placeholder when no source position is available. Span-aware
// callers use parse_type_expr_with_span directly.
pub fn parse_type_expr(kw: &str) -> Result<TypeExpr, TypeError> {
    parse_type_expr_with_span(kw, &Span::unknown())
}
```

The rune already captures WHY structurally. If a redundant body comment exists WITHIN `parse_type_expr` (or in its DOC-comment span at lines 2854-2858 that REPEATS the rune's content), drop the redundant comment. Keep the rune (it's the structural why); drop the repeat.

**Inspection step:** read lines 2840-2880 in full. Identify the redundant comment text that mirrors the rune. Delete that redundant text only. If you find NO redundancy, report "R3.7: no redundant comment found; rune at line 2859 is the only WHY explanation" in your return paragraph.

**WHY:** `feedback_dont_document_non_fixes` — comments defending non-fixes are deferral-layer pollution; the rune is the legitimate structural acknowledgment.

### R3.8 — verify (and tighten if needed) types.rs:30-36 "# Scope notes" section

Current text at lines 30-36:
```rust
//! # Scope notes
//!
//! The name-resolution pass resolves call heads; field-position type
//! references are validated at use site, not at registration time.
//! Code generation for Rust-backed compiled binaries is out of wat-rs
//! scope (058 backlog Track 2 tracks this concern).
```

This text reads PRESENT-TENSE on the surface ("validates at use site"; "is out of wat-rs scope"). But examine for deferral framing:
- "058 backlog Track 2 tracks this concern" — this names an external tracker (058 backlog) for an explicit out-of-scope decision. **This is acceptable affirmative-out-of-scope language** per FM 11 ("Out of arc N's scope. Tracked in arc M") — provided 058 Track 2 actually exists.

**Verification step:** confirm `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/` exists and has any Track 2 reference. If yes → leave the text unchanged (it's honest affirmative-out-of-scope). If no (the tracker reference is fictional) → rewrite the closing sentence to: `Code generation for Rust-backed compiled binaries is outside wat-rs scope by design — the substrate compiles to its own runtime.`

Report which path you took in your return paragraph.

**WHY:** exigere — verify the tracker-citation is honest before treating it as affirmative-out-of-scope. A citation to a non-existent tracker IS deferral dressed as affirmation.

### R3.9 — verify (and tighten if needed) check.rs:1680-1710 hint-extensibility framing

Current text at lines 1680-1690:
```rust
/// Collect all migration hints that fire for this (callee, expected,
/// got) triple into a single string. Each hint already self-identifies
/// via its leading `"arc N — "` prefix; we just concatenate.
///
/// Returns `None` when no hint applies — currently the steady state
/// (arcs 111 / 112 / 113 retired their helpers 2026-04-30 once the
/// respective consumer waves swept clean). The function stays as
/// Migration-hint extensibility point: to add a hint for a new migration
/// scenario, add a `<scenario>_migration_hint(callee, expected, got)` entry
/// to the array below. The check pass invokes each entry; the first that
/// returns Some wins.
```

The text reads as PRESENT-TENSE (extensibility point; how to add a hint; how the dispatch works). The phrase "The function stays as Migration-hint extensibility point" reads slightly awkwardly but is present-state.

**Verification step:** read lines 1660-1720 in full context. Identify any forward-promise phrasing ("future arcs will add", "scaffold for future arcs", "when needed we'll", etc.). If found → rewrite to present-state. If only present-tense extensibility-point framing → leave unchanged and report "R3.9: docstring already present-tense; no rewrite needed" in your return paragraph.

**WHY:** exigere — the commit message named "scaffold for future arcs" framing; that may have already been cleaned in an earlier R3 round, or it may still be lurking. Verify and tighten only if needed.

## Cadence (sequential — each fix verifies before the next starts)

1. Baseline gate: `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test function 2>&1 | tail -3` (expect 8/0); `cargo test --release --test probe_arc243_stone3_typeerror_pattern_a 2>&1 | tail -3` (expect 3/0)
2. **R3.1**: edit src/types.rs:280-334 (RegistrationPrivilege enum + 2 call-site renames + body update). Run `cargo build --release --tests` (expect clean) + `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0).
3. **R3.2**: edit src/types.rs:1742-1858 (extract closure-threaded helper; thin wrappers). Run `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); the splice path is exercised by lib tests.
4. **R3.3**: edit src/check.rs:10043-10121 (extract `register_defclause_from_form` helper). Run `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0) + `cargo test --release --test function 2>&1 | tail -3` (expect 8/0).
5. **R3.7**: inspect types.rs:2840-2880; drop redundant body comment if found.
6. **R3.8**: verify types.rs:30-36; rewrite ONLY if 058 Track 2 doesn't exist (cite path or absence in return paragraph).
7. **R3.9**: verify check.rs:1660-1720; rewrite ONLY if forward-promise framing found.
8. **Final gates** (MUST hold): `cargo test --release --lib -p wat` ≥ 890 PASS / 0 FAIL; `cargo test --release --test function` = 8/0; `cargo test --release --test probe_arc243_stone3_typeerror_pattern_a` = 3/0; `cargo build --release --tests --workspace` = exit 0; `cargo clippy --release 2>&1 | grep -cE "^warning:"` ≤ 897.
9. **DO NOT COMMIT** — orchestrator commits atomic post-vigilia + SCORE Phase B.
10. **Return paragraph** (≤ 200 words): which fixes landed (R3.1/.2/.3 confirmed; R3.7/.8/.9 status per inspection); final gates; any trap-doors encountered + how absorbed; any ADDITIONAL findings surfaced in touched files (per `feedback_pre_existing_is_not_exemption` — report honestly; do NOT silently skip).

## STOP triggers (REJECTION — ship nothing; surface verbatim)

1. Compile errors not traced to the 6 named fixes
2. Lib < 890
3. tests/function < 8
4. probe_arc243_stone3 < 3
5. Workspace test-build fails post-strike
6. Clippy > 897
7. 60 min elapsed (mechanical scope; the bound is generous)
8. holon-rs touched (STOP-5)
9. Scope creep into other R3 items (R3.4..R3.6 + R3.10..R3.12 LANDED-OR-REVERSED in checkpoint; don't re-touch)
10. Scope creep into other error types (Stone 243.4+ are next-stones' scope; this sweep is TypeError-touched files only)
11. Backward-compat aliases for the bool→enum rename
12. New deferral language anywhere
13. INTERSTITIAL touched
14. Vigilia or conformare cast attempted by sonnet
15. New runes for the 6 fixes — all are solvable + perf-OK; all FIX
16. AMBIGUOUS deferral text encountered in R3.7/R3.8/R3.9 — STOP, surface verbatim with file:line, await orchestrator triage rather than autonomous rewrite into wrong shape

## Read in order (pre-strike)

1. `docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `docs/CONFORMARE.md` (Pattern A doctrine + rune mechanism)
3. `docs/arc/2026/05/243-conformare-error-shape/DESIGN.md`
4. `docs/arc/2026/05/243-conformare-error-shape/BRIEF-STONE-243.3.md` (the Phase A brief that produced checkpoint `48d3393e`)
5. `docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.3.md` (Phase A audit; Phase B pending)
6. `git log -1 48d3393e --format=%B` (the checkpoint commit; names the 12-item R3 ledger + which landed/reversed/remain)
7. `src/types.rs` (you'll edit 4 spans within it — read full file before R3.1 starts)
8. `src/check.rs` lines 10000-10130 + 1660-1720 (R3.3 + R3.9 sites)

## Calibration

**Predicted band: 30-60 min Mode A.** Six mechanical fixes: 3 structural refactors (enum rename + 2 helper extractions) + 3 inspection-and-conditional-rewrite tasks. Comparable to Stone 241.18a R3.6/R3.7 micro-fix sweeps (~10-15 min each). Cascade depth: shallow per Stone 243.3 Phase A audit pattern (lib tests use structural assertions; private helper signatures don't break consumers).

## Post-strike — orchestrator path

After your return:
1. Orchestrator casts R2 vigilia (8 spells; intueri + solvere + purgare + struere + sequi + temperare + exigere + conformare) on touched files. **No "skip pre-existing" instruction to spells** per `feedback_pre_existing_is_not_exemption`. Drives L1+L2=0.
2. Orchestrator authors SCORE-STONE-243.3.md Phase B section (vigilia ledger + conformare attestation + final gates + doctrine reconciliation).
3. Orchestrator commits Stone 243.3 atomic (R3 sweep + vigilia artifacts + SCORE Phase B).
4. Stone 243.4 opens — first domain rehome (src/types/ + TypeError to types/error.rs) under inscribed corrected doctrine.
