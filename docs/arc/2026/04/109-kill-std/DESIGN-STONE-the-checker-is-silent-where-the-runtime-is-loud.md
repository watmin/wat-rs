# DESIGN — `infer_fn`'s two silent paths: delete the exemption, do not add a check

> ⚠ **THIS IS NOT ARC 109's CONCERN.** It was found during γ-i and its NOTE lives here, but it is a
> checker-diagnostics defect with no `:-` content. It wants a home; minting an arc number is the
> builder's ruling, not mine. Filed at the discovery site rather than silently adopted.

## The defect

`--check` gives a **false green** on a malformed `fn`. Measured on `c889639aa`:

```
first slot of (:wat::core::fn ??? [x <- :i64] -> :i64 x), then applied to a String
  (nothing)   ✅ rejects        [1 2]  ✅ rejects  (a Vector parses as an argspec and fails properly)
  :- [T]      ✅ rejects  ← closed by γ-i's binder peel
  :foo        ⛔ ACCEPTS        42     ⛔ ACCEPTS        "s"  ⛔ ACCEPTS

fewer than three signature slots
  (fn)  ⛔ ACCEPTS      (fn [x <- :i64])  ⛔ ACCEPTS      (fn [x <- :i64] ->)  ⛔ ACCEPTS
```

**Severity, honestly bounded** — this is NOT a soundness hole:

- the **runtime rejects** every one of them with a located `MalformedForm`;
- there is **no leakage** — a genuine type error beside a malformed fn is still caught;
- so no wrong behaviour can ship. What ships is a **false green from the gate**, plus that fn's body
  and every call to it going unchecked until execution.

## ★ The two sister sequences disagree

`src/function/eval.rs:41` says it outright: *"Note: sister sequence in `src/function/infer.rs`
(infer_fn)."* Same peel, same guard, opposite behaviour:

```rust
// eval.rs:46  — the RUNTIME twin: LOUD
if sig_args.len() < 3 {
    return Err(RuntimeError::new(list_span.clone(), RuntimeErrorKind::MalformedForm {
        head: FN_HEAD.into(),
        reason: format!("expected [name <- :T ...] -> :Ret body ...; got {} element(s)", sig_args.len())
    }));
}

// infer.rs:110 — the CHECK twin: SILENT
if sig_args.len() < 3 {
    // "parse won't even call check for badly-formed fn"
    return CheckResult::ok(fresh.fresh());
}
```

and the second path:

```rust
// infer.rs:57
Err(step) if matches!(step.kind, ParseStepKind::ArgsVecNotVector { .. }) => SigParse::SilentReject,
    // "the non-fn-shaped form is handled by other checker arms"
```

## ⛔ Both rationales are FALSE at the only two call sites

```
src/check.rs:2377   ":wat::core::fn" => crate::function::infer_fn(args, env, locals, fresh, subst)
src/check.rs:4704   ":wat::core::fn" => { let (val, mut errs) = crate::function::infer_fn(…) }
```

`infer_fn` is reachable **only** under a `:wat::core::fn` head, from a dispatch that has already
matched it. So the form is always a fn form — *"not fn-shaped at all"* cannot occur — and there is no
"other checker arm": the head is consumed. `check` IS called; nothing else reports it.

★ A comment asserting a precondition the disk refutes, protecting the very thing it describes from
being questioned. Same class as this arc's *"checker-locked"* false law.
`[[feedback_a_comment_can_ship_a_gap_as_a_law]]`

## The fix: remove the exemption, do not author a diagnostic

The loud path already exists and already carries the right words. `SigParse::Diagnosed` →
`CheckResult::err` → a located `MalformedForm` whose text is `ParseStepKind::reason()` — **the same
string the runtime prints.** So:

1. Delete the `ArgsVecNotVector → SilentReject` guard; everything falls through to `Diagnosed`.
2. Replace the `len() < 3` early return with the same located error, mirroring eval.rs's own
   `reason` text verbatim so the twins finally agree.
3. **`SigParse::SilentReject` is then dead — delete the variant.** Its doc says the enum exists to
   *"make the silent-vs-diagnostic distinction structural"*; with the variant gone the distinction
   disappears and **there is no silent arm left to fall into.** Convention → unrepresentable.
4. `infer_fn_non_vector_args_returns_silent_placeholder` pins the removed behaviour, so it goes — and
   it earns its removal twice: it calls `infer_fn` directly with a synthetic array, so it proves
   nothing about what a caller sees. Replaced by a `.wat` probe that must now fail `--check`.

## ⚠ THE RISK THAT DECIDES THE SIZE — and it is probably why the arm was written

**Six macro templates in `wat/` quasiquote a `(:wat::core::fn …)` form** (`core.wat`, `service.wat`,
`bracket.wat`). If any routes through `infer_fn` with an unquote node where the args-vector belongs,
deleting the exemption lights them all up.

Probed: a `defmacro` emitting `` `(:wat::core::fn ~argv -> :wat::core::i64 ~body) `` accepts today —
but that does NOT isolate whether it travels through this arm or is simply never inferred as an
expression. **The floor answers this and no reading of mine can.**

- Floor green → the exemption was pure debt, and the stone is the four deletions above.
- Those six light up → we have found the real constraint, and the design becomes *"diagnose unless
  the slot is an unquote node"* — a slot rule, which this arc has now built three times.

Either outcome is the stone succeeding. The failure mode to avoid is **assuming** the first.

## The four questions

- **Obvious?** YES — two sister sequences over the same shape, one erroring and one shrugging, with
  the divergence justified by comments the call sites refute.
- **Simple?** YES — it deletes a guard, a branch and an enum variant. No diagnostic is authored; the
  message already exists in `ParseStepKind::reason()` and in eval.rs.
- **Honest?** YES — and it is the axis that fails today: `--check` reporting success on a program the
  runtime will refuse is the gate lying about its own subject.
- **Good UX?** YES — the error moves from execution time to check time, in the same words, and the
  caller learns it from the gate that exists to tell them.
