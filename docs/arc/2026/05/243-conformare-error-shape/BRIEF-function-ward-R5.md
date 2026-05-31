# BRIEF — function/ WARD R5 — close the last 2 doc/convention L2s (earn the stamp)

You are sonnet. The `src/function/` home's R4 re-cast came back 5/8 spells L1=0 L2=0;
structural core clean since R1. TWO small real L2s remain (verified vs live code). This R5
closes them. One rune finding is REJECTED (see bottom) — do not touch it.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits after re-casting
the vigilia. If you think a commit is needed, STOP and say so. (#1 rejection trigger.)

The working tree already holds R1+R2+R3+R4 (uncommitted, by design). ADD R5 on top; do NOT
revert/stash. Files in scope: `src/function/infer.rs` ONLY.

---

## FIX 1 (L2, intueri) — the comment lies on the no-metadata path

`src/function/infer.rs` (~84-90), the metadata-peel comment:
```
// at args[0] is binding-level; peel it off so the type-checker
// sees the real signature at args[1..]. The metadata was already
// stored in binding_metadata by try_parse_fn_shape_def at
// register-defines time.
```
`peel_metadata_preamble` returns `&args[1..]` ONLY when `args[0]` is a metadata map; with no
metadata it returns `args` UNCHANGED (the type-checker sees `args[0..]`). The "at args[1..]"
claim is true only for the meta-present case — it lies on the no-meta path. The sister
comment in eval.rs (~38) words it correctly without the false index claim.

Fix — drop the false unconditional index; state it conditionally (or mirror eval.rs):
```
// at args[0] is binding-level; peel it off so the type-checker
// sees the real signature (args[1..] when metadata is present; args
// unchanged otherwise). The metadata was already stored in
// binding_metadata by try_parse_fn_shape_def at register-defines time.
```

---

## FIX 2 (L2, cernere) — collapse the third "anonymous fn" spelling to the canonical one

`src/function/infer.rs` (~128), the `ReturnTypeMismatch` label:
```rust
function: "<anonymous fn>".to_string(),
```
This is a THIRD spelling of "unnamed fn" in the substrate. The CANONICAL identity-keyword
for a nameless fn is `:anonymous` — `src/runtime.rs:12313` uses exactly `None => ":anonymous".into()`
as "the head keyword when a fn has no name" (the fn-value → holon materialize path). A fn's
`function:` label IS its identity; when present it's the fn's name keyword, so the absent-name
placeholder should read as that same keyword form. `<anonymous fn>` reads as a meta-placeholder
and diverges from the canonical `:anonymous`.

Fix:
```rust
function: ":anonymous".to_string(),
```
Rendered: `<file>:<line>: :anonymous: body produces …; signature declares …` — location once
(span prefix), identity reads as the canonical unnamed-fn keyword. (The `<anonymous>` form at
`runtime_error_edn.rs:200` is the EDN-serialization path — a different render context, outside
this home; not changed here.)

---

## DO NOT TOUCH (rejected / L3)

- **struere/cernere parse.rs:210 rune (`/// rune:sequi(reclassified-by-caller)`)** — REJECTED.
  Two spells flagged (a) its `///` doc-comment placement (convention leans `//`, but the `///`
  form is a valid in-tree minority — e.g. `types.rs:1557` `rune:conformare(spanless-by-domain)`
  is also `///`), and (b) the category should be `struere` not `sequi`. NEITHER is applied: this
  is the EARNED, user-accepted rune from a prior session (committed `bbf670d8`); its category
  `sequi` is correct (intentional information-discard = control-flow honesty = sequi's domain,
  not struere's structure domain); and it sits inside the function's doc block where the `///`
  form is the natural attachment. Relitigating a settled earned artifact on cosmetic grounds is
  churn, not a fix. Do NOT change the rune.
- temperare: ALL L3 (Vec::new before early returns; eval_fn substrate-schema allocs). LEAVE.
- intueri L3: the "intentionally discarded" stop-phrase wording in the rune block — part of the
  rejected earned rune; LEAVE.
- cernere L3: `"form element(s)"` plural-(s) awkwardness at count 0 — consistent with the
  substrate's `"argument(s)"` pattern. LEAVE.

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 890 / 0)
- `cargo test --release --lib -p wat function`
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat`
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (must not regress vs ~877)

Report: the two changed regions (before/after); the five gate numbers; explicit confirmation
of ZERO git mutations + that ONLY src/function/infer.rs was edited. Raw report.
