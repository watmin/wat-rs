# BRIEF — STONE: `:wat::holon::literal` is a special form

You are a **rider**, not the orchestrator. **Ending your turn ENDS you** — nothing will wake you.
Run every command in the FOREGROUND and block on it. You may not spawn sub-agents.

Anchor: **`/home/john/work/holon/wat-rs`**. `pwd` first. Any path containing `.claude/worktrees/`
is harness state — never operate on it. Do not commit, push, stash, or revert. Do not run the full
floor; the orchestrator runs it centrally.

Read `DESIGN-STONE-holon-literal-is-a-special-form.md` (sibling) first.

## The work in one paragraph

`:wat::holon::literal` captures its argument unevaluated — its own check arm says *"exactly as
`:wat::core::quote`"* — but it is declared `#[wat_intrinsic]`, so the registry reports
`Kind::Intrinsic`. That wrong answer already forced a hand-written exception into `eval_apply`.
Reclassify it to `#[wat_special_form]` with the two `role =` pointers the completeness gate
requires, delete the exception, and correct one stale comment on the `Kind` enum itself.

## Rooms, in order

1. **`src/check.rs:3265-3282`** — read BOTH arms. `:wat::holon::literal`'s inline arm, and
   directly below it `:wat::core::forms`'s arm, which is **already** a `role = check` delegation
   (Stone 1a-γ-i). The second is your worked template for the first; copy its shape.
2. **`src/intrinsic/holon/atom.rs:~640-670`** — the doc block and `#[wat_intrinsic]` attribute on
   `eval_holon_literal`.
3. **`src/intrinsic/special/forms.rs`** — a complete, shipped example of the target shape: a
   `#[wat_special_form]` struct with its doc block, plus separate `#[wat_special_form_impl(…,
   role = eval)]` and `role = check` fns. Read it before writing anything.
4. **`src/runtime.rs`, `eval_apply`'s STOP-8 check** — the two-name exception. Only
   `:wat::holon::literal` leaves; `:wat::core::defn` stays with its reasoning intact.
5. **`wat/runtime-meta.wat`, the `Kind` enum** — `:SpecialForm`'s comment claims "no
   NativeHandler". Measured false (19 `role = eval` impls; `intrinsic/mod.rs:418` states the
   opposite). Correct it.

## The completeness gate is your acceptance test

`src/intrinsic/mod.rs:2802` requires every `Kind::SpecialForm` row to carry **both** a `check` and
an `eval` impl. A half-done reclassification goes red there by design — that gate is the stone's
own proof, so let it drive rather than pre-satisfying it by guesswork.

## ⛔ The spelling trap — the corpus does NOT write this verb's FQDN

Census on `:wat::holon::literal` alone returns **zero test files** and is wrong. The corpus writes
it as the **`#holon` reader tag**. Search for BOTH:

```bash
grep -rln "holon::literal\|#holon" tests/ wat/ wat-scripts/ wat-tests/ src/
```

Expect ~10 files / ~42 sites. `tests/types/probe_arc294b_holon_literal.rs` is the dedicated test.
The orchestrator published "zero" before catching this.

## What is expected to change, and what is not

- **`eval_apply` still rejects it.** The whole point is that the registry now answers correctly, so
  the exception is unnecessary — not that the rejection goes away. Probe it before and after.
- **Reflection's sentinel head moves** `:wat::core::__internal/registered` →
  `:wat::core::__internal/special-form` (`reflect/lookup.rs:418`, `reflect/verbs.rs:270`). Expected
  and correct. If a test pins the old value, update it and say so in your report.
- **`check.rs:5650`'s arity fallback no longer applies** — and this must cost nothing, because
  literal's own check arm enforces `args.len() != 1` and returns first. **Verify that by probe**:
  `(:wat::holon::literal a b)` must still be an `ArityMismatch` at check time, naming
  `:wat::holon::literal`, expected 1.

## STOP triggers — each rejects; none permits a smaller delivery

- **STOP-1** — if the completeness gate cannot be satisfied because no honest `role = check` or
  `role = eval` pointer exists, STOP and report which and why. Do not add a stub fn that nothing
  calls to satisfy a wall.
- **STOP-2** — if `(:wat::holon::literal a b)` stops being an arity error, STOP. The DESIGN's claim
  that `check.rs:5650` is unreachable for this verb is then wrong.
- **STOP-3** — do not touch `@Purity`. `Pure` on a verb that never evaluates its argument looks
  wrong, and `quote`/`stream::lazy` say `Pure` too — a three-row question, out of scope, and
  changing it here would hide a behaviour change inside a reclassification.
- **STOP-4** — do not remove `:wat::core::defn`'s exception, and do not audit other rows for the
  same mis-kinding.

## Verification

```
cargo nextest run --release -E 'binary_id(wat)'
cargo nextest run --release -E 'binary_id(wat::types)'
cargo nextest run --release -E 'binary_id(wat::reflection)'
cargo nextest run --release -E 'binary_id(wat::value)'
cargo clippy --release --all-targets -- -D warnings
```

Run the binaries whole. The orchestrator has repeatedly handed riders a list of test names that
omitted where the failures were.

## What to report

The final doc-block header (all five `@` axis lines plus `@syntax` if you added one); the
before/after for `eval_apply`'s rejection and for the arity probe; the reflection sentinel's
before/after; your two-spelling census with counts; the Summary line per scoped run; and anything
that surprised you.
