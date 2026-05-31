# BRIEF — function/ WARD R3 — close the last 2 real L2s (earn the stamp)

You are sonnet. The `src/function/` home's R2 re-cast came back 5/8 spells L1=0 L2=0
(structural core clean). Two REAL must-fix L2s remain (verified against live code); one
cernere L2 is being REJECTED this round as an attested cross-home item (see bottom). This
R3 closes the two so the home earns its `vigilatum` on a clean re-cast.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits after re-casting
the vigilia. If you think a commit is needed, STOP and say so. (#1 rejection trigger.)

The working tree already holds the R1+R2 function/ changes (uncommitted, by design). ADD
R3 on top; do NOT revert/stash. Files in scope: `src/function/{parse,infer}.rs` ONLY.

---

## FIX 1 (L2, solvere — doc lies about a mechanism) — parse.rs:203-207

The doc on `parse_fn_signature_for_check` describes the A3 path's mechanism FALSELY:
```
/// A3 diagnostic parser (`parse_fn_signature_for_check_diag` in `infer.rs`,
/// which threads full `ArgSpecError` detail into the inference error stream
/// via `From<ArgSpecError> for CheckError`).
```
The real A3 path (verified at infer.rs:58-72) routes `ParseStepKind::ArgSpecFailed(k)` →
`k.reason()` → `CheckError::MalformedForm`. `From<ArgSpecError> for CheckError` is NEVER
invoked on this path (the prefix returns a `ParseStep`, not an `ArgSpecError`). The doc
names a mechanism that does not exist.

Fix — replace the parenthetical (lines 205-207) with the true mechanism:
```
/// A3 diagnostic parser (`parse_fn_signature_for_check_diag` in `infer.rs`,
/// which surfaces each `ParseStep` as a `CheckError::MalformedForm` whose reason
/// comes from `ParseStepKind::reason()` — for the argspec case, that delegates to
/// `ArgSpecErrorKind::reason()`).
```
(Keep the rest of the doc + the rune block unchanged.)

---

## FIX 2 (L2, struere — type-enforce the silent-vs-diagnostic split) — infer.rs

`parse_fn_signature_for_check_diag` currently returns `Option<(...)>` and pushes into a
`&mut Vec<CheckError>` side-channel; `None` means TWO different things (silent-reject vs
diagnosed) disambiguated only by a prose `errors.is_empty()` invariant at the caller. Make
the distinction a TYPE so the invariant is structural, not documented. This also removes the
`if matches!(...)` guard (struere's other L2).

**2a.** Replace the function with a private 3-way outcome enum + a clean match. In
`src/function/infer.rs`, define above the function:
```rust
/// Outcome of fn-signature diagnostic parsing — makes the silent-vs-diagnostic
/// distinction structural (no `errors.is_empty()` side-channel inference).
enum SigParse {
    /// Parsed cleanly.
    Parsed(Vec<String>, Vec<TypeExpr>, TypeExpr),
    /// Outer form is not fn-shaped at all (ArgsVecNotVector) — silent: caller
    /// returns a fresh placeholder, no diagnostic.
    SilentReject,
    /// Fn-shaped but malformed — carries the diagnostic to surface.
    Diagnosed(CheckError),
}
```
Rewrite the function to return `SigParse` (drop the `errors: &mut` parameter entirely):
```rust
fn parse_fn_signature_for_check_diag(args: &[WatAST; 3]) -> SigParse {
    match parse_fn_signature_prefix(args) {
        Ok((p, t, r)) => SigParse::Parsed(p, t, r),
        Err(step) if matches!(step.kind, ParseStepKind::ArgsVecNotVector { .. }) =>
            SigParse::SilentReject,
        Err(step) => SigParse::Diagnosed(CheckError::MalformedForm {
            head: FN_HEAD.into(),
            reason: step.kind.reason(),
            span: step.span,
            remedies: vec![],
        }),
    }
}
```
(The `Err(step) if …` guard here is a clean pattern-guard at the match head — NOT the
nested `if matches!` inside an arm that struere flagged. This is the idiomatic form.)

**2b.** Update the sole caller `infer_fn` (infer.rs ~123-135). Replace the
`match parse_fn_signature_for_check_diag(sig3, &mut errors) { Some(parsed) => …, None => {
if errors.is_empty() … } }` block with a match on the 3-way outcome:
```rust
let (param_names, param_types, ret_type) = match parse_fn_signature_for_check_diag(sig3) {
    SigParse::Parsed(p, t, r) => (p, t, r),
    SigParse::SilentReject => {
        // Not fn-shaped at all — silent-by-intent; return a fresh placeholder.
        return CheckResult::ok(fresh.fresh());
    }
    SigParse::Diagnosed(err) => {
        errors.push(err);
        return CheckResult::errs(errors);
    }
};
```
Note: `errors` is still declared at the top of `infer_fn` (it accumulates the
ReturnTypeMismatch later in the body, and `drain_errors_into(&mut errors)`). The change is
that the SIGNATURE-parse step no longer needs the side-channel — it returns its diagnostic
by value, and the caller decides. The `errors.is_empty()` discriminant + its invariant
comment are DELETED (the type now carries the distinction). Verify `errors` is still used
correctly for the body-inference diagnostics below the signature block — leave that part
intact.

After this, grep `parse_fn_signature_for_check_diag` — its only caller is `infer_fn`; the
`&mut errors` argument is gone from both decl and call.

---

## DO NOT TOUCH (L3 / attested cross-home / rejected)

- **cernere L2-1 (BadRetType `"invalid return type: {k}"` stutter)** — REJECTED this round.
  When `k = TypeErrorKind::MalformedTypeExpr`, the full render stutters
  (`malformed :wat::core::fn form: invalid return type: malformed type expression …`). BUT
  this is the SAME shape already ACCEPTED in the warded argspec home
  (`"invalid type keyword: {inner}"`, error.rs:62) — the stutter root is the not-yet-Pattern-A
  `CheckError::MalformedForm` / `RuntimeError::MalformedForm` WRAPPER (which prepends
  "malformed … form:"), which is OUTSIDE this home. Fixing function/ alone would diverge it
  from the warded precedent. This is attested to the CheckError Pattern A stone (243.6) and
  RuntimeError Pattern A stone (243.7), where the wrapper Display is fixed uniformly. Do NOT
  change the BadRetType reason; do NOT add a rune. LEAVE.
- temperare: ALL L3 (eval_fn re-parse / Arc / clones; reason() String alloc on error path).
  LEAVE.
- intueri/cernere L3 cosmetics: `body` vs `body_ast`; A2/A3 labels; "arg(s)" plural;
  article asymmetry across the three reason strings. LEAVE.

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 890 / 0)
- `cargo test --release --lib -p wat function`
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat`
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (must not regress vs ~877)

Report: every file touched + one-line description; the new `SigParse` enum + the rewritten
`parse_fn_signature_for_check_diag`; the updated `infer_fn` call-site block; confirmation the
`errors.is_empty()` discriminant is gone; the new parse.rs:205-207 doc text; the five gate
numbers; explicit confirmation of ZERO git mutations. Raw report.
