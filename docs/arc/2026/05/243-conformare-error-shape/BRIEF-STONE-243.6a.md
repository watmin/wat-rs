# BRIEF — Stone 243.6a — `CheckError` → Pattern A (carve to `src/check/error.rs`)

## What to do

Retrofit `CheckError` to **Pattern A** and carve it into the `src/check/` home — the exact shape `TypeError` already shipped at 243.3 (`src/types/error.rs`). This **composes pieces that all already exist and are verified**: the outer-struct+kind-enum form, the split `Display` impls, the carve-and-reexport. You are mirroring a shipped template, not inventing a mechanism. The cascade is large but overwhelmingly in-file mechanical (452 of 459 sites in `check.rs`); `cargo` names every site — reshape the type, follow the errors to green.

## Read in order (the lair, pre-walked)

1. `src/types/error.rs` — **the template.** `pub struct TypeError { pub span, pub kind }` + `pub enum TypeErrorKind` + split `Display` (TypeErrorKind span-free `:152`, TypeError delegates `:254`). Copy this shape exactly.
2. `docs/arc/2026/05/243-conformare-error-shape/SCORE-STONE-243.3.md` — the prior stone's SCORE; mirror its structure for your SCORE.
3. `src/check.rs:88–725` — the current flat `enum CheckError` (33 variants). Every variant carries `span`. Note the 5 multi-span variants (below).
4. `src/check.rs:727` `impl fmt::Display for CheckError` + `:1075` `impl fmt::Display for CheckErrors` + `:1349 impl CheckError` + `:1361 fn diagnostic()` + `:1873 impl CheckErrors` + `:1875 fn diagnostics()` — the impls to reshape.
5. `src/check.rs:49–50` — `pub mod env; pub use env::CheckEnv;` — the carve precedent; add `pub mod error;` + `pub use error::{CheckError, CheckErrorKind, CheckErrors};` alongside.
6. `src/check/env.rs` (top) — the warded-home module-doc style + the `vigilatum` stamp placement (you do NOT add the stamp — that's earned by the orchestrator's live vigilia cast post-strike).
7. `tests/probe_arc243_stone6_checkerror_pattern_a.rs` — the contract you must satisfy (it fails to compile now; it must compile + pass 3/0 after).

## Implementation sketch (fill the path; do not invent the shape)

In a new `src/check/error.rs`:

```rust
pub struct CheckError {
    pub span: Span,
    pub kind: CheckErrorKind,
}

pub enum CheckErrorKind {
    ArityMismatch { callee: String, expected: usize, got: usize },   // span dropped → outer
    UnknownCallee { callee: String },                                // span dropped → outer
    ReturnTypeMismatch { function: String, expected: String, got: String, remedies: Vec<Remedy> },
    // … all 33 variants, span field removed; remedies/other fields preserved …
    // 5 MULTI-SPAN variants keep their SECONDARY span(s) as domain-named fields:
    SandboxScopeLeak { /* … */ second_span: Span, third_span: Span },        // outer = most-actionable
    ProcessJoinBeforeOutputDrain { /* … */ accessor_span: Span },
    ProcessJoinHoldsStdinSender { /* … */ stdin_sender_span: Span },
    DefRedefForbidden { /* … */ original_def_span: Span },
    DefRedefTypeChange { /* … */ original_def_span: Span },
}

impl fmt::Display for CheckErrorKind { /* span-free per-variant message — mirror types/error.rs:152 */ }
impl fmt::Display for CheckError { /* delegate to kind; prefix span when known — mirror :254 */ }
```

Then: every `CheckError::Variant { …, span }` construction → `CheckError { span, kind: CheckErrorKind::Variant { … } }`; every match on `CheckError::Variant` → match on `err.kind` / `CheckErrorKind::Variant`. `diagnostic()` reads `self.span` directly (the N-arm extraction collapses).

**Multi-span rule:** for each of the 5, the outer `span` is the **most-actionable** location (the site the user edits to fix — read the existing `Display` message to choose); the secondary span(s) become domain-named kind fields. SandboxScopeLeak(3), ProcessJoinBeforeOutputDrain(2), ProcessJoinHoldsStdinSender(2), DefRedefForbidden(2), DefRedefTypeChange(2).

## Discipline (blast radius)

- `src/check.rs` + new `src/check/error.rs` + the 7 cross-file cascade sites (`runtime.rs`, `function/parse.rs`, `function/infer.rs`, `argspec/mod.rs`) ONLY.
- NO new `Value` variant. NO `holon-rs` edits. NO error-semantics change (no merging/recovery). NO touching `check_program`'s walker structure or `collect_hints` (those are Stone 243.6b — leave their runes in place).
- Do NOT add the `vigilatum` stamp (earned post-strike by live cast).
- Do NOT commit. Leave the tree dirty for the orchestrator to verify + commit.

## STOP triggers (REJECTION criteria — each names the correct path)

1. If a variant's "most-actionable" span is genuinely ambiguous after reading its `Display` message → STOP, name the variant + both candidate spans, surface as a finding. Do NOT guess silently.
2. If carving CheckError to `error.rs` forces a circular import (error.rs needs a type that needs CheckError) → STOP; the correct path is a `use crate::…` one-way edge (mirror how `env.rs` imports from the parent). If genuinely circular, surface it.
3. If the cascade reveals a CheckError construction that has NO span available at the call site → STOP and name it (this is a real span-discipline gap, not a reshape detail — the orchestrator decides). Do NOT pass `Span::unknown()` to paper over it.
4. If `remedies: Vec<Remedy>` or any non-span field would be dropped to make a variant compile → STOP; preserve every non-span field on the kind variant.
5. Any temptation to "fall back to" leaving span on a variant → REJECTED. Pattern A is span-at-the-outer-struct, zero exceptions.

## FM 2-bis evidence

`tests/probe_arc243_stone6_checkerror_pattern_a.rs` is committed and **disconfirms at HEAD** — 6 compile errors, exactly the gap: `E0432` (CheckErrorKind unresolved), `E0574` ×4 (enum not struct), `E0609` (no field `span`). Your reshape makes it compile + pass 3/0. That flip is the proof.

## SCORE doc spec + calibration

Write `SCORE-STONE-243.6a.md` mirroring `SCORE-STONE-243.3.md`: scorecard (each contract row · command · result), the cascade fail-count waterfall, honest deltas, line counts, the 5 multi-span dispositions you chose (variant → most-actionable span + secondary field names).

- **Predicted band:** 60–120 min Mode A (reshape + carve + 459-site cascade). STOP at 240 min.
- **Verification:** `cargo test --release --test probe_arc243_stone6_checkerror_pattern_a` → 3/0; `cargo build --release --tests` → clean; `cargo test --release --lib -p wat` → green (report pass/fail counts).

Plain tools throughout — vanilla `cargo`, `grep`, `git` (read-only). Do not commit.
