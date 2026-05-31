# BRIEF — function/ WARD — ParseStep Pattern A (type-impossible arity) + annihilate vigilia findings

You are sonnet. Ward the `src/function/` home. A live 8-spell vigilia found one root
failure domain (`ParseStep::ArityMismatch` — spanless, the exact "now retired" example
`docs/CONFORMARE.md:11` names but never actually retired) plus a cluster. This ONE
stone annihilates them so the home earns its `vigilatum` stamp on a clean re-cast.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits atomically
after re-casting the vigilia. If you think a commit is needed, STOP and say so. (#1
rejection trigger.)

Work ONLY in `/home/watmin/work/holon/wat-rs/`. Files in scope:
`src/function/{parse,infer,eval,metadata,mod}.rs`, `src/runtime.rs`, `src/check.rs`,
and `docs/CONFORMARE.md`. Touch tests only if an assertion needs updating.

---

## FIX 1 (L1, the root) — make arity type-impossible; delete ArityMismatch

`parse_fn_signature_prefix` (parse.rs) currently takes `sig: &[WatAST]` and opens with
`if sig.len() != 3 { return Err(ParseStep::ArityMismatch { actual: sig.len() }) }`.
ArityMismatch is spanless → maps to `RuntimeError { span: Span::unknown() }`. Eliminate
the CLASS, not the symptom: make the arity precondition a TYPE.

**1a.** Change the prefix signature to `sig: &[WatAST; 3]` and DELETE the
`if sig.len() != 3` guard entirely (the type now guarantees exactly 3). Destructure
`sig[0]/sig[1]/sig[2]` as before.

**1b.** Change BOTH public entry points to take `&[WatAST; 3]`:
- `parse_fn_signature(sig: &[WatAST; 3]) -> Result<_, RuntimeError>`
- `parse_fn_signature_for_check(sig: &[WatAST; 3]) -> Result<_, ()>`
Both just forward to the prefix (the `for_check` keeps its existing
`rune:sequi(reclassified-by-caller)` + `.map_err(|_| ())` — do NOT remove that rune).

**1c.** DELETE `ParseStep::ArityMismatch { actual }` and the arm that maps it
(in `parse_fn_signature`'s match) and the arm in infer.rs's diag matcher. The
`actual` field had no reader anyway (verified: dead payload).

**1d.** Update the four call sites to pass `&[WatAST; 3]`:
- `src/function/eval.rs` (~53): after the existing `if sig_args.len() < 3` guard, do
  `let sig3: &[WatAST; 3] = sig_args[..3].try_into().expect("len >= 3 gated above");`
  then `parse_fn_signature(sig3)?`.
- `src/runtime.rs` (~4129): `sig_args` is ALREADY a `[WatAST; 3]` array literal — just
  pass `&sig_args` (it already coerces; confirm it compiles as `&[WatAST; 3]`).
- `src/function/infer.rs` (~121): after the `if sig_args.len() < 3` guard, same
  `try_into` as eval, then `parse_fn_signature_for_check_diag(sig3, &mut diagnostics)`.
  Change `parse_fn_signature_for_check_diag`'s param to `&[WatAST; 3]` too.
- `src/check.rs` (~9357, the A2 `:ensure :fn` classifier): currently
  `parse_fn_signature_for_check(fn_items.get(..3).unwrap_or(fn_items))`. Replace with a
  try_into that folds the `<3` case into the EXISTING `Err(())` → `EnsureFnInvalid` arm:
  ```rust
  match fn_items.get(..3).and_then(|s| <&[WatAST; 3]>::try_from(s).ok()) {
      Some(sig3) => match crate::function::parse_fn_signature_for_check(sig3) {
          Ok((param_names, param_types, ret_type)) => { /* existing Ok body */ }
          Err(()) => { /* existing Err(()) body — EnsureFnInvalid "malformed :fn signature…" */ }
      },
      None => { /* SAME EnsureFnInvalid "malformed :fn signature…" push as the Err(()) arm */ }
  }
  ```
  (Honest: a <3 slice IS a malformed :fn shape; it now routes to the same diagnostic
  instead of relying on the prefix to reject a wrong-length slice.)

---

## FIX 2 (L1, conformare) — ParseStep to Pattern A (closes the BadRetType double-span)

After FIX 1, ParseStep has 5 variants; three carry their own `span`, two wrap full
Pattern-A sub-errors. Retrofit to the canonical Pattern A struct+kind shape, reusing the
machinery the argspec ward just built (`TypeErrorKind` has a span-free `Display`;
`ArgSpecErrorKind::reason()` is `pub(crate)`):

```rust
pub(in crate::function) struct ParseStep {
    pub span: Span,
    pub kind: ParseStepKind,
}
pub(in crate::function) enum ParseStepKind {
    ArgsVecNotVector { found_variant: &'static str },
    ArrowMissing,
    RetTypeNotKeyword,
    BadRetType(Box<crate::types::TypeErrorKind>),
    ArgSpecFailed(Box<crate::argspec::ArgSpecErrorKind>),
}
```

At each construction site in the prefix, hoist the span to the outer struct:
- ArgsVecNotVector: `ParseStep { span: other.span().clone(), kind: ArgsVecNotVector { found_variant } }`
- ArrowMissing: `ParseStep { span: sig[1].span().clone(), kind: ArrowMissing }`
- RetTypeNotKeyword: `ParseStep { span: other.span().clone(), kind: RetTypeNotKeyword }`
- BadRetType: when `parse_type_expr_with_span(..)` returns `Err(te)`:
  `ParseStep { span: te.span, kind: BadRetType(Box::new(te.kind)) }`
- ArgSpecFailed: when `parse_argspec_triples(..)` returns `Err(ae)`:
  `ParseStep { span: ae.span, kind: ArgSpecFailed(Box::new(ae.kind)) }`

**WHY this fixes a real latent bug (not just doctrine):** `parse_fn_signature` currently
maps `BadRetType(e) => RuntimeError { reason: e.to_string(), span: e.span }`. Post the
argspec ward, `TypeError`'s Display = `span_prefix + kind`, so `e.to_string()` ALREADY
embeds the span → the rendered diagnostic DOUBLE-STAMPS the location (identical to the
MalformedTypeKeyword bug just fixed in argspec). Storing `Box<TypeErrorKind>` (span-free
Display) + `span: step.span` eliminates the double-stamp.

**The mappers** (`parse_fn_signature` → RuntimeError; the diag matcher → CheckError) now
read `step.span` uniformly and render reasons span-free:
- `ArgsVecNotVector { found_variant }` → reason `format!("fn signature must be a vector `[name <- :T ...]`; got {found_variant}")`
- `ArrowMissing` → see FIX 4 wording
- `RetTypeNotKeyword` → see FIX 4 wording
- `BadRetType(k)` → reason `format!("invalid return type: {k}")` (k is TypeErrorKind, span-free Display)
- `ArgSpecFailed(k)` → reason `k.reason()` (ArgSpecErrorKind::reason(), span-free)
All use `head: FN_HEAD` (FIX 5) and `span: step.span` / `step.span.clone()`.

---

## FIX 3 (L1, sequi) — promote BadRetType to a check-tier diagnostic

In `parse_fn_signature_for_check_diag` (infer.rs), `BadRetType` currently returns `None`
SILENTLY (the silent tier). That means `(fn [x <- :i64] -> :Any body)` type-checks
clean then crashes at runtime — a real asymmetry. Move `BadRetType` from the silent arms
to the diagnostic arms: push a `CheckError::MalformedForm { head: FN_HEAD, reason:
format!("invalid return type: {k}"), span: step.span, remedies: vec![] }` then `None`
(mirror how `ArgSpecFailed` already pushes). The remaining silent arms (ArgsVecNotVector,
and — post FIX1 — nothing else shape-prior) stay silent; document the split honestly in
the function doc (which arms push vs return silent, and WHY: ArgsVecNotVector is a
not-an-fn-shape rejection, BadRetType is a real content error worth surfacing).

---

## FIX 4 (L2, cernere) — absence-only messages name what was found

`ArrowMissing` and `RetTypeNotKeyword` say only "missing X" while the home's own
`ArgsVecNotVector` says "got {found_variant}". Match that bar (both the parse.rs
RuntimeError mapper AND the infer.rs CheckError mapper — keep the two tiers' wording
identical to each other):
- ArrowMissing → `"fn signature: expected `->` between args-vector and return type; got {found}"`
  where `{found}` is `sig[1].variant_name()` (thread the found-variant into the
  ParseStepKind::ArrowMissing if needed, OR compute at the single prefix construction
  site and store it: `ArrowMissing { found_variant: &'static str }` — your call, keep it
  Pattern A). 
- RetTypeNotKeyword → `"fn signature: expected a return-type keyword after `->` (e.g. `:wat::core::i64`); got {found}"`
  similarly carrying the found variant.

(If adding `found_variant` to these two kinds is cleaner than recomputing, do that — it
keeps the message honest and the kind still span-free.)

---

## FIX 5 (L2, struere) — extract FN_HEAD const

The literal `":wat::core::fn"` appears ~9× across parse.rs/infer.rs/eval.rs as both the
`parse_argspec_triples` head arg and the `head:` field. Declare once:
```rust
pub(in crate::function) const FN_HEAD: &str = ":wat::core::fn";
```
in `src/function/mod.rs`. Replace all 9 literal sites (grep `:wat::core::fn` in
src/function/) with `FN_HEAD`. (Where a `String` is needed, `FN_HEAD.into()`.)

---

## FIX 6 (L1, intueri) — kill the rotting line number in mod.rs

`src/function/mod.rs` (~42) cites `src/check.rs (~9810, …)` — the real call site is
~9357 and WILL drift again. Drop the brittle line number; reference the call site by its
stable identity instead, e.g. "the `:ensure :fn` defclause validation in `src/check.rs`
(the sole caller of `parse_fn_signature_for_check`)". No `~NNNN`.

---

## FIX 7 (L2, intueri) — singularize metadata.rs doc

`src/function/metadata.rs` (~1-5) says "shared helpers" / "utilities used by multiple
sub-modules" for a module with ONE function. Singularize: "shared helper" / "utility
used by sibling sub-modules within `src/function/`."

---

## FIX 8 (doc, conformare) — make CONFORMARE.md true

`docs/CONFORMARE.md:11` calls `ParseStep::ArityMismatch { actual: usize }` "now retired"
— it was NOT retired until this stone. The claim is now becoming true. Update the line so
it reads as a genuinely-retired worked example (e.g. note it was retired by making the fn
prefix take `&[WatAST; 3]`, arity now type-impossible) rather than implying it was already
done. Keep it as the canonical teaching example; just stop lying about its status.

---

## DO NOT TOUCH (L3 — leave per let-need-reveal)

- temperare: eval_fn re-parse / `synthesize_fn_body` clone / `Arc::new(body)` /
  `env.clone()` — all substrate-schema or cost-may-move, unproven. LEAVE.
- `<fn@span>` anonymous-fn label in ReturnTypeMismatch — LEAVE for now (it's a
  cross-substrate convention question; not this stone's scope). Do NOT change it.
- The `diagnostics.is_empty()` discriminant in infer_fn — leave the logic; if you want,
  add ONE clarifying comment naming the invariant, but no structural change.

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 890+ / 0)
- `cargo test --release --lib -p wat function` (the home's unit tests)
- `cargo test --release --test 'wat_arc241*'` AND any fn-form probe (defn/fn parsing)
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat`
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (must not regress vs ~877; the
  Box wrapping must not trip large_enum_variant)

Report: every file touched + one-line description; the new ParseStep + ParseStepKind
decl; the new prefix signature; the BadRetType mapper (both tiers) showing span-free
reason + step.span; confirmation ArityMismatch is fully gone (grep returns nothing); the
six gate numbers; explicit confirmation of ZERO git mutations. Raw report.
