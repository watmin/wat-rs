# BRIEF — function/ WARD R6 — close the last 2 (fixpoint) prose L2s

You are sonnet. The `src/function/` home's R5 re-cast came back 6/8 spells L1=0 L2=0. The
structural core has been clean since R1; what remains are two diagnostic-prose L2s, and each
has a STABLE FIXPOINT (the fix cannot oscillate to a third position). This R6 closes them so
the home earns its `vigilatum`.

ZERO git mutations — NO commit/add/stash/reset, NO scratch files outside named edits.
`git status`/`git diff`/`git grep` READ-ONLY only. The orchestrator commits after re-casting
the vigilia. If you think a commit is needed, STOP and say so. (#1 rejection trigger.)

The working tree already holds R1-R5 (uncommitted, by design). ADD R6 on top; do NOT
revert/stash. Files in scope: `src/function/eval.rs` + `src/function/parse.rs` ONLY.

---

## FIX 1 (L2, cernere — the oscillation fixpoint) — `eval.rs` arity-message unit word

`src/function/eval.rs` (~46), the `sig_args.len() < 3` guard reason currently reads:
```rust
reason: format!("expected [name <- :T ...] -> :Ret body ...; got {} form element(s)", sig_args.len()),
```
History: this was `"arg(s)"` → cernere flagged "arg" as misleading (collides with "parameter")
→ changed to `"form element(s)"` → cernere now flags "form" as redundant (the `MalformedForm`
wrapper already names the form) and inconsistent with the substrate's `"elements"` convention
(runtime.rs:4922, 6782). The FIXPOINT that satisfies BOTH concerns — not a parameter word, not
redundant, matches convention — is bare `"element(s)"`:
```rust
reason: format!("expected [name <- :T ...] -> :Ret body ...; got {} element(s)", sig_args.len()),
```
Change `form element(s)` → `element(s)`. Nothing else on that line changes.

---

## FIX 2 (L2, intueri) — drop the opaque arc-jargon label in `parse.rs`

`src/function/parse.rs` (~203), in the doc for `parse_fn_signature_for_check`:
```
/// Returns (names, types, ret). This is the A2 CLASSIFIER-PROBE: it answers
/// "is this a well-formed fn-shape?" for the `:ensure :fn` validator, NOT the
/// A3 diagnostic parser (`parse_fn_signature_for_check_diag` in `infer.rs`,
```
"A2 CLASSIFIER-PROBE" / "A3" are arc-internal taxonomy handles (defined in
`src/argspec/mod.rs`, not in this file) — a cold reader of `parse.rs` cannot decode them
without cross-file hunting. The surrounding English already carries the meaning. Replace the
opaque handles with plain English, keeping the explanation intact:
```
/// Returns (names, types, ret). This is the SILENT CLASSIFIER: it answers
/// "is this a well-formed fn-shape?" for the `:ensure :fn` validator, NOT the
/// DIAGNOSTIC parser (`parse_fn_signature_for_check_diag` in `infer.rs`,
```
(Change "A2 CLASSIFIER-PROBE" → "SILENT CLASSIFIER" and "A3 diagnostic parser" → "DIAGNOSTIC
parser". Leave the rest of the doc + the rune block UNCHANGED — the rune at ~210 is a settled
earned artifact; do not touch it.)

---

## DO NOT TOUCH (settled / attested / L3)

- The `parse.rs:210` `rune:sequi(reclassified-by-caller)` — settled earned artifact (placement
  `///` + category `sequi` both correct/accepted). Do NOT touch.
- The two attested cross-home items (BadRetType `"invalid return type: {k}"` stutter +
  `reason()` "fn signature:" prefix asymmetry) — tracked to 243.6/243.7. LEAVE.
- All temperare L3 (substrate-schema allocs, error-path allocs). LEAVE.
- `body` vs `body_ast` sister-name nuance; the WHAT-comment at infer.rs:111; `ParseStep`
  field `pub` vs `pub(in crate::function)` — all L3. LEAVE.

---

## VERIFY before returning — report EXACT numbers

- `cargo test --release --lib -p wat` (expect 890 / 0)
- `cargo test --release --lib -p wat function`
- `cargo build --release --tests --workspace` (expect Finished)
- `cargo build --release -p wat`
- `cargo clippy --release -p wat 2>&1 | grep -c warning` (must not regress vs ~877)

Report: the two changed lines/regions (before/after); the five gate numbers; explicit
confirmation that ONLY eval.rs + parse.rs were edited and ZERO git mutations. Raw report.
