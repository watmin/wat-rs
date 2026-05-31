# BRIEF — function/ WARD R2 — annihilate the live-cast L2 cluster (earn the stamp)

You are sonnet. The `src/function/` home's R1 strike landed clean structurally (Pattern A
holds, ArityMismatch gone, no regression — solvere/purgare/sequi/temperare/conformare all
L1=0 L2=0), but a live 8-spell re-cast surfaced an L2 cluster from intueri/struere/cernere.
This R2 annihilates them so the home earns its `vigilatum` stamp on a clean re-cast.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits after re-casting
the vigilia. If you think a commit is needed, STOP and say so. (#1 rejection trigger.)

Work ONLY in `/home/watmin/work/holon/wat-rs/`. Files in scope:
`src/function/{parse,infer,eval,metadata,mod}.rs`. Touch tests only if an assertion needs
the new wording.

---

## FIX 1 (L2, cernere — the double-span, VERIFIED) — kill the ReturnTypeMismatch double-stamp

`src/function/infer.rs` (~166) constructs `CheckError::ReturnTypeMismatch` with:
```rust
function: format!("<fn@{}>", body_span),
span: body_span.clone(),
```
`ReturnTypeMismatch`'s Display (`src/check.rs:759-762`) is `"{}{}: body produces …"` where
the first `{}` = `span_prefix(span)` and the second = `function`. So the location renders
TWICE: once from the span prefix, once embedded in the `<fn@span>` label. Same double-span
class as argspec MalformedTypeKeyword + this home's own BadRetType.

**IMPORTANT — the `<fn@span>` convention (corrected from R1 draft):** `format!("<fn@{}>", …span)`
IS an established 6-site convention (`freeze.rs:361,390`, `runtime.rs:21753,21772,21785,21844`)
— BUT every one of those sites is the `func.name.unwrap_or_else(|| <fn@span>)` fallback used
in a render context that does NOT prepend a span prefix (plain log-line messages like
`"function {label} has no explicit return"`), so there `<fn@span>` is the SOLE locator and
correct. The ReturnTypeMismatch path is the ONE context whose Display already prefixes the
span — so here, and only here, re-embedding the span in the label double-stamps. The named
sibling RTM sites (check.rs:9306/9341) pass a real NAME (`{cs.name}/clause#{n}`), never a
span, so they don't double-stamp. Do NOT touch the 6 convention sites; fix only this one.

Fix: keep the machine-readable `span: body_span.clone()` (the structured `span` field feeds
tooling — the `Diagnostic` builder at check.rs:1388 — and must stay real). Change ONLY the
human label so it does not re-embed the location:
```rust
function: "<anonymous fn>".to_string(),
span: body_span.clone(),
```
Rendered: `<file>:<line>: <anonymous fn>: body produces …` — location once (prefix), label
is a clean human name. This legitimately diverges from `<fn@span>` because this is the only
render path that already carries the location via prefix; the convention stays intact where
it belongs (the no-prefix contexts).

---

## FIX 2 (L2, struere) — dedupe the ParseStepKind→reason mapping; collapse the unreachable

The reason strings for `ArrowMissing`/`RetTypeNotKeyword`/`BadRetType`/`ArgSpecFailed` are
WORD-FOR-WORD duplicated between `parse.rs`'s `parse_fn_signature` mapper and `infer.rs`'s
`parse_fn_signature_for_check_diag`. Future wording drift would be silent. Extract one
source of truth:

**2a.** Add to `parse.rs` (near ParseStepKind):
```rust
impl ParseStepKind {
    /// Span-free human reason for this parse-step failure. Both tier mappers
    /// (eval RuntimeError, check CheckError) render through this — one source of truth.
    pub(in crate::function) fn reason(&self) -> String {
        match self {
            ParseStepKind::ArgsVecNotVector { found_variant } =>
                format!("fn signature: expected a vector `[name <- :T ...]` as the args-vector; got {found_variant}"),
            ParseStepKind::ArrowMissing { found_variant } =>
                format!("fn signature: expected `->` between args-vector and return type; got {found_variant}"),
            ParseStepKind::RetTypeNotKeyword { found_variant } =>
                format!("fn signature: expected a return-type keyword after `->` (e.g. `:wat::core::i64`); got {found_variant}"),
            ParseStepKind::BadRetType(k) => format!("invalid return type: {k}"),
            ParseStepKind::ArgSpecFailed(k) => k.reason(),
        }
    }
}
```
(Note `ArgsVecNotVector`'s wording here is the FIX-3 aligned form — see FIX 3.)

**2b.** `parse_fn_signature`'s mapper (parse.rs) collapses to a single arm:
```rust
parse_fn_signature_prefix(args).map_err(|step| RuntimeError::MalformedForm {
    head: FN_HEAD.into(),
    reason: step.kind.reason(),
    span: step.span,
})
```

**2c.** `parse_fn_signature_for_check_diag` (infer.rs) — collapse the outer-guard +
inner-`unreachable!` shape into ONE flat match on `step.kind` (the `unreachable!` goes
away — it was only there because the two-level split couldn't see the guard):
```rust
match parse_fn_signature_prefix(args) {
    Ok(parsed) => Some(parsed),
    Err(step) => {
        // ArgsVecNotVector is the silent tier (outer form not fn-shaped at all):
        // no diagnostic, caller falls through to a fresh placeholder.
        if matches!(step.kind, ParseStepKind::ArgsVecNotVector { .. }) {
            return None;
        }
        errors.push(CheckError::MalformedForm {
            head: FN_HEAD.into(),
            reason: step.kind.reason(),
            span: step.span,
            remedies: vec![],
        });
        None
    }
}
```
Keep the doc block above the fn (the silent-vs-diagnostic tier split explanation) — it's
still accurate; just ensure it matches this shape.

---

## FIX 3 (L2, cernere) — align ArgsVecNotVector voice

Already folded into FIX 2a's `reason()`: `ArgsVecNotVector` now reads
`"fn signature: expected a vector `[name <- :T ...]` as the args-vector; got {found_variant}"`
— matching the `"fn signature: expected …; got …"` shape of its two siblings (was the
odd-one-out `"fn signature must be a vector …"`). No separate edit needed beyond FIX 2a;
just confirm no other site still emits the old "must be a vector" wording.

---

## FIX 4 (L2, cernere) — eval `<3 args` message stops double-naming the head

`src/function/eval.rs` (~44-50): the `sig_args.len() < 3` RuntimeError reason is
`"expected (:wat::core::fn [name <- :T ...] -> :Ret body ...); got {} args"`. Rendered
through `MalformedForm`'s Display it becomes `malformed :wat::core::fn form: expected
(:wat::core::fn …)` — the head appears twice. Drop the head from the example (the Display
wrapper already names it):
```rust
reason: format!("expected [name <- :T ...] -> :Ret body ...; got {} arg(s)", sig_args.len()),
```

---

## FIX 5 (L2, intueri) — metadata.rs title names the domain

`src/function/metadata.rs:1` title is `//! # fn-form shared helper` — names the ROLE, not
the DOMAIN; a reader opening `metadata.rs` learns nothing about metadata. Change the title
to name the domain:
```
//! # fn-form binding-metadata peel
```
Keep the body paragraph as-is (it already explains the peel correctly).

---

## FIX 6 (L2, intueri) — unify the error-accumulator name in infer.rs

Within `infer.rs` the SAME concept has two names: `parse_fn_signature_for_check_diag`'s
parameter is `errors`, but `infer_fn`'s local is `diagnostics` (and the doc says "drain
`errors`"). Pick ONE — rename `infer_fn`'s local `diagnostics` → `errors` throughout
`infer_fn` (the declaration + every use: the `drain_errors_into(&mut …)`, the
`.is_empty()` checks, the `CheckResult::errs(…)` / `partial_with(…, …)` calls). After this,
one name (`errors`) names the accumulator everywhere in the file, matching the substrate's
`check.rs` convention.

---

## DO NOT TOUCH (L3 — leave per let-need-reveal / not-this-stone)

- temperare: ALL L3 (eval_fn re-parse / Arc::new / clones — substrate-schema, cost-may-move,
  unproven). LEAVE.
- intueri L3: `body` vs `body_ast` sister-name difference; the A2/A3 labels in parse.rs doc.
  LEAVE (cosmetic).
- cernere L3: `ArrowMissing` "got symbol" not showing the symbol's text (`found_variant` is
  `&'static str` by design — showing the value needs a deeper change). LEAVE.
- solvere L3: `pub` vs `pub(in crate::function)` on ParseStep fields; `pub(super)` vs
  `pub(in crate::function)` on peel. Semantically identical. LEAVE.

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 890 / 0)
- `cargo test --release --lib -p wat function`
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat`
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (must not regress vs ~877)

Report: every file touched + one-line description; the new `ParseStepKind::reason()` impl;
the collapsed `parse_fn_signature` mapper + the flat `parse_fn_signature_for_check_diag`
match (showing the `unreachable!` is gone); the new ReturnTypeMismatch label; the five gate
numbers; explicit confirmation of ZERO git mutations. Raw report.
