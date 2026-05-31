# BRIEF — function/ WARD R4 — close the last 2 L2s (earn the stamp)

You are sonnet. The `src/function/` home's R3 re-cast came back 5/8 spells L1=0 L2=0; the
structural core is clean. TWO small real L2s remain (verified vs live code). This R4 closes
them so the home earns its `vigilatum` on a clean re-cast. One additional cernere L2 is
REJECTED (see bottom) — do not touch it.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits after re-casting
the vigilia. If you think a commit is needed, STOP and say so. (#1 rejection trigger.)

The working tree already holds the R1+R2+R3 function/ changes (uncommitted, by design). ADD
R4 on top; do NOT revert/stash. Files in scope: `src/function/infer.rs` + `src/function/eval.rs`
ONLY.

---

## FIX 1 (L2, struere) — use the right-sized constructor in the Diagnosed arm

`src/function/infer.rs` (~108-110), the `SigParse::Diagnosed` arm:
```rust
SigParse::Diagnosed(err) => {
    errors.push(err);
    return CheckResult::errs(errors);
}
```
This routes a provably-SINGLE error through the multi-error vec API. At this point `errors`
(declared at the top of `infer_fn`) is still empty — nothing has pushed to it before the
signature-parse step — so the vec carries exactly one element. `CheckResult::err(error)`
exists (check.rs:1180; it wraps a single CheckError) and is the right-sized constructor.

Fix — return the single error directly, no vec intermediary:
```rust
SigParse::Diagnosed(err) => return CheckResult::err(err),
```
Do NOT touch the `let mut errors` declaration or its other uses — `errors` is still live for
the body-inference phase below (the `drain_errors_into(&mut errors)` collect, the
`ReturnTypeMismatch` push, and the final `errors.is_empty()` body-gate). Only this one arm
changes: it no longer needs to push-then-wrap.

---

## FIX 2 (L2, cernere) — eval `<3 args` message names the right concept

`src/function/eval.rs` (~46), the `sig_args.len() < 3` guard reason:
```rust
reason: format!("expected [name <- :T ...] -> :Ret body ...; got {} arg(s)", sig_args.len()),
```
`sig_args` counts the fn-form's CHILD ELEMENTS (args-vector, `->`, ret-keyword, body nodes)
after metadata peel — NOT function parameters. "arg(s)" collides with the domain term
"argument" (a user reads "got 1 arg(s)" as "1 parameter"). Name the real unit:
```rust
reason: format!("expected [name <- :T ...] -> :Ret body ...; got {} form element(s)", sig_args.len()),
```
(Keep everything else — head: FN_HEAD, span: list_span.clone() — unchanged.)

---

## DO NOT TOUCH (rejected / attested / L3)

- **cernere L2-2 (the `reason()` "fn signature:" prefix inconsistency)** — REJECTED. The
  proposed fix (prepend `"fn signature: "` to `BadRetType` and `ArgSpecFailed` in
  `ParseStepKind::reason()`) is NOT applied because: (a) `BadRetType`'s body is
  `"invalid return type: {k}"` where `{k}` carries the ALREADY-ATTESTED stutter (the
  not-yet-Pattern-A MalformedForm wrapper, tracked 243.6/243.7) — adding a third label
  deepens exactly that stutter; (b) `ArgSpecFailed` delegates to the WARDED argspec home's
  `ArgSpecErrorKind::reason()` — wrapping warded output with a local prefix fights the warded
  precedent. The prefix asymmetry is folded into the SAME cross-home attestation as the
  BadRetType stutter (resolved uniformly when the MalformedForm wrapper goes Pattern A at
  243.6/243.7). Do NOT add the prefix; do NOT add a rune.
- temperare: ALL L3 (eval_fn re-parse/Arc/clones; reason() String on error path; the empty
  `type_params` Vec). LEAVE.
- intueri/cernere/solvere L3 cosmetics: `body` vs `body_ast`; A2/A3 labels; provenance-doc
  nuance; backtick-vs-bare token style. LEAVE.

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 890 / 0)
- `cargo test --release --lib -p wat function`
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat`
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (must not regress vs ~877)

Report: the two changed lines (before/after); confirmation `errors` is still used by the
body-inference phase (FIX 1 didn't orphan it); the five gate numbers; explicit confirmation
of ZERO git mutations. Raw report.
