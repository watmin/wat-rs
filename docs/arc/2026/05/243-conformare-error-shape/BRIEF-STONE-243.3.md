# BRIEF — Stone 243.3 — TypeError Pattern A retrofit

You are sonnet. Stone 243.3 — first conformare retrofit. Apply Pattern A (`struct *Error { span: Span, kind: *ErrorKind }`) to `TypeError` substrate-wide.

**Anchor cwd:** `/home/watmin/work/holon/wat-rs/`. Verify with `pwd`. Reject `.claude/worktrees/`.

## Critical doctrine (pre-authorized — read BEFORE strike)

1. **Pattern A is the type shape** per `docs/CONFORMARE.md`. Each error type is its own concrete struct + per-type kind enum. No generic wrapper. No trait.
2. **HARD CUT** (`feedback_hard_cut_admits_no_bypasses`) — no backward-compat aliases for the old TypeError enum shape; the variants move into TypeErrorKind cleanly.
3. **NO deferral language** (`feedback_dont_document_non_fixes` + exigere spell) — no "future arc could", "intentionally discarded", "outside scope", "would require", "could", "should" comments.
4. **NO runes for solvable findings** (`feedback_runes_illegal_when_solvable`) — runes are EXCEPTION mechanisms for unsolvable paths or perf-impairing fixes only.
5. **Sonnet writes substrate** (`feedback_sonnet_writes_substrate`) — orchestrator briefs + scores + commits; sonnet does the Rust edits.
6. **holon-rs NEVER touched** (STOP-5).
7. **DO NOT write to INTERSTITIAL** (`feedback_sonnet_never_drafts_interstitial`).
8. **SCORE doc authored at end** (`feedback_score_present_check_before_closure`) — `docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.3.md` covering Phase A scorecard + post-strike attestation.

## Pre-spawn baseline (verified at HEAD)

- Lib: 890 PASS / 0 FAIL
- tests/function: 8/0
- Workspace test-build: clean
- Clippy: 897
- Stone 243.3 STRIKE-READY: DESIGN + FM 2-bis probe + BRIEF + EXPECTATIONS all committed; probe FAILS to compile pre-stone (intended FM 2-bis disconfirmation shape)

## What to do (Phase A — sonnet)

### S1 — Refactor TypeError shape in `src/types.rs`

Current shape (16 variants, 15 with span field, 1 spanless):
```rust
pub enum TypeError {
    DuplicateType { name: String, span: Span },
    // ... 14 more spanned variants ...
    CyclicSubtype { child: String, parent: String },  // lacks span
}
```

Post-stone shape:
```rust
pub struct TypeError {
    pub span: Span,
    pub kind: TypeErrorKind,
}

pub enum TypeErrorKind {
    DuplicateType { name: String },
    // ... 14 more variants WITHOUT span fields ...

    // rune:conformare(spanless-by-domain) — register_subtype operates on
    // FQDN string arguments; no AST node in scope at registration time;
    // the outer struct's span field is `Span::unknown()` at the emitter
    // site because no source location exists for registry-cycle detection.
    CyclicSubtype { child: String, parent: String },
}
```

Operational steps:
1. Rename `pub enum TypeError` → `pub enum TypeErrorKind`
2. Strip `span: Span` field from every variant (15 sites in the enum definition)
3. Mint new `pub struct TypeError { pub span: Span, pub kind: TypeErrorKind }` immediately ABOVE the (renamed) TypeErrorKind enum
4. Add the rune annotation immediately preceding the CyclicSubtype variant
5. Update `impl Display for TypeError` (around `src/types.rs:1657`+) — match on `&self.kind` not `self`; use `self.span` directly for any span-prefix rendering

### S2 — Cascade through emitter sites (estimated 114)

Find all sites via:
```
grep -rn "TypeError::[A-Z]" src/
```

For each `TypeError::Variant { span, fields }` construction:
- Spanned variants (15 kinds): rewrite as `TypeError { span, kind: TypeErrorKind::Variant { fields } }` — move `span` to outer position; remove from inner variant
- CyclicSubtype (1 site at `src/types.rs:435`): `TypeError { span: Span::unknown(), kind: TypeErrorKind::CyclicSubtype { child, parent } }`

The substrate-as-teacher discipline handles the cascade: Rust's compile errors at every old construction site direct the work. Iterate `cargo build` → fix one category of errors → re-run → continue until clean.

### S3 — Cascade through consumer match arms

Find all matches via:
```
grep -rn "match.*TypeError\|TypeError::[A-Z]" src/ tests/
```

For each `match err { TypeError::Variant { span, fields } => ... }`:
- Rewrite as `match err.kind { TypeErrorKind::Variant { fields } => ... }`
- Where `span` was destructured in the pattern + used in the body, replace with `err.span` (or equivalent named binding from the outer struct)

**Critical site — `src/function/parse.rs:154-172`** (the 16-arm span-extraction match in BadRetType arm):
```rust
// Pre-stone (current 17 lines):
ParseStep::BadRetType(e) => {
    let span = match &e {
        TypeError::MalformedTypeExpr { span, .. } => span.clone(),
        TypeError::AnyBanned { span, .. } => span.clone(),
        // ... 14 more arms ...
        TypeError::CyclicSubtype { .. } => Span::unknown(),
    };
    RuntimeError::MalformedForm { head: ":wat::core::fn".into(), reason: e.to_string(), span }
},

// Post-stone (collapses to single-path access):
ParseStep::BadRetType(e) => RuntimeError::MalformedForm {
    head: ":wat::core::fn".into(),
    reason: e.to_string(),  // or e.kind.to_string() / appropriate render
    span: e.span,
},
```

The WHY comment for the 16-arm match (`// WHY: each TypeError variant carries its own span field; extract via match`) goes away — no longer applicable. Delete the comment too.

### S4 — From impl span preservation

Find via:
```
grep -rn "impl From<TypeError>" src/
```

For each `impl From<TypeError> for X`:
- Source's `e.span` → destination's `span` field (preserve, don't drop)
- Source's `e.kind` → mapped into destination's kind/structure

Known site: `src/freeze.rs:583` `impl From<TypeError> for StartupError`.

If `impl From<ArgSpecError> for TypeError` exists, ensure ArgSpecError's `classify()` span feeds the new TypeError struct's span field.

### S5 — Test cascade

`tests/probe_arc237_stone1_typeunion_substrate.rs` references TypeError; update any pattern matches to new shape.

The newly committed FM 2-bis probe (`tests/probe_arc243_stone3_typeerror_pattern_a.rs`) currently FAILS to compile (pre-stone) — after S1 lands, it must COMPILE + PASS (3 contracts).

### S6 — CONFORMARE.md update

`docs/CONFORMARE.md` currently describes Pattern A abstractly. Update to cite Stone 243.3 + TypeError as the first applied example, with a concrete code-block showing the post-retrofit TypeError shape. Replace the generic "MyError" prose example with the concrete TypeError reference.

### S7 — Author SCORE-STONE-243.3.md (Phase A portion)

Per `feedback_score_present_check_before_closure`. Phase A SCORE covers:
- Mode (A / B)
- Per-step audit (S1-S6 confirmation; emitter count actually touched; consumer count actually touched)
- Cascade audit table (sites updated per file)
- Honest deltas
- Trap-doors encountered + absorbed in-flight
- Final metrics (lib N/0, clippy count, tests/function 8/0, probe_arc243_stone3 3/0)

NOTE: Phase B (conformare spell re-cast on src/types.rs) is orchestrator-cast post-strike per Song #44 wisdom. Sonnet's SCORE has a placeholder footer: "Phase B conformare re-cast attestation pending orchestrator cast."

## Discipline

- HARD CUT TOTAL — no backward-compat aliases
- NO deferral language anywhere (exigere will catch)
- NO defensive comments explaining non-fixes
- holon-rs NEVER touched
- DO NOT commit (orchestrator commits after conformare re-cast attests)
- DO NOT update INTERSTITIAL
- DO NOT cast vigilia or conformare (orchestrator-cast)
- Other error types (RuntimeError, CheckError, ParseStep, ArgSpecError, etc.) OFF-LIMITS — this stone is TypeError ONLY; next stones handle each type

## Read in order (pre-strike)

1. `docs/COMPACTION-AMNESIA-RECOVERY.md`
2. `docs/CONFORMARE.md` — the doctrine you are applying
3. `docs/arc/2026/05/243-conformare-error-shape/DESIGN.md` — arc plan
4. `docs/arc/2026/05/243-conformare-error-shape/DESIGN-STONE-243.3.md` — this stone
5. `docs/arc/2026/05/243-conformare-error-shape/EXPECTATIONS-STONE-243.3.md` — scorecard
6. `docs/arc/2026/05/243-conformare-error-shape/CONFORMARE-FIRST-CAST.md` — spell's verdict that selected TypeError as starter
7. `tests/probe_arc243_stone3_typeerror_pattern_a.rs` — FM 2-bis probe (must compile + pass post-strike)
8. `src/types.rs` — read full file before starting S1; understand the Display impl + internal type-machinery patterns

## Cadence

1. Baseline: `cargo test --release --lib -p wat 2>&1 | tail -3` (expect 890/0); `cargo test --release --test function 2>&1 | tail -3` (expect 8/0)
2. Verify probe FAILS pre-stone: `cargo build --release --tests --test probe_arc243_stone3_typeerror_pattern_a 2>&1 | tail -5` (expect compile errors)
3. S1: refactor `src/types.rs` (rename enum, mint struct, strip variant spans, add rune)
4. Cargo iteration: substrate-as-teacher cascade through emitters + consumers via compile errors
5. S2-S5: cascade until `cargo build --release --tests --workspace` clean
6. Verify probe now passes: `cargo test --release --test probe_arc243_stone3_typeerror_pattern_a 2>&1 | tail -3` (expect 3/0)
7. S6: update CONFORMARE.md with concrete TypeError example
8. Final gates: lib ≥ 890, tests/function 8/0, workspace test-build clean, clippy ≤ 945
9. S7: author SCORE-STONE-243.3.md (Phase A)
10. DO NOT COMMIT — orchestrator commits after Phase B conformare re-cast attests

## STOP triggers (REJECTION)

1. Compile errors not traced to TypeError refactor
2. Lib < 890
3. tests/function < 8
4. Workspace test-build fails after refactor
5. 240 min elapsed (cascade scope is real; the bound is generous)
6. holon-rs touched (STOP-5)
7. Sub-stone scope creep (other error types refactored — only TypeError this stone)
8. Backward-compat aliases for old TypeError enum shape
9. New deferral language anywhere (exigere will catch)
10. INTERSTITIAL touched
11. Conformare or vigilia cast attempted by sonnet

## Post-strike return

One paragraph: Pattern A applied to TypeError (struct + kind enum minted); emitter cascade count (sites updated); consumer cascade count (sites updated); 16-arm match at parse.rs collapsed to `err.span`; CyclicSubtype rune in place; Display + From impls updated; FM 2-bis probe passing 3/0; SCORE Phase A authored; lib N/0; clippy count; tests/function N/0.
