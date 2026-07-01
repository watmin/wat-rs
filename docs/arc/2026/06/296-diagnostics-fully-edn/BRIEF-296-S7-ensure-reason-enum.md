# BRIEF — 296 S7: `EnsureFnInvalid.reason` String → `EnsureFnInvalidReason` enum

> **Executor: one sonnet, MAIN tree** (the `../holon-rs` path dep breaks worktree builds — do NOT use a worktree).
> Orchestrator drew this + `DESIGN-296-S7-ensure-reason-enum.md`; weighs the kill forced-clean by its OWN gate AND the
> emitted wire EDN. **Commit nothing.** Anchor `/home/watmin/work/holon/wat-rs`; `pwd` first; reject any
> `.claude/worktrees/` path. Do NOT spawn subagents.

## The work (one paragraph)
`CheckErrorKind::EnsureFnInvalid { defclause_name, clause_index, reason: String }` carries a **discriminant-as-prose**:
its 7 construction sites emit 5 fixed failure modes, three of which `format!()` structured data (a count, a type pair,
a type) into the string. Replace `reason: String` with a new `#[derive(…ToEdn)]` enum `EnsureFnInvalidReason` (5
variants), restructure the 7 sites to build the variant (no `format!` of the data), add a `Display` for the enum that
reproduces each current sentence **byte-for-byte**, and let the existing `CheckErrorKind` derive emit `:reason`
structurally. Then **un-ignore** the RED probe and make it GREEN. This is a sibling of the S1/S6 typed-cause strikes.

## Read first (in order)
- **`docs/arc/2026/06/296-diagnostics-fully-edn/DESIGN-296-S7-ensure-reason-enum.md`** — the full contract (the enum,
  the site→variant map, the Display, the proof, the blast radius). THIS BRIEF IS THE BUILD ORDER.
- **`src/check/error.rs:54`** — `#[derive(Debug, Clone, wat_macros::ToEdn)] pub enum CheckErrorKind` (the derive already
  emits `:reason (reason.to_edn())`; you only need the field's type to BE `ToEdn`).
- **`src/check/error.rs:306`** — the `EnsureFnInvalid { defclause_name, clause_index, reason: String }` variant (the
  field you retype) and **`:671`** — its `Display` arm (`… :fn is invalid — {reason}`).
- **`src/check.rs:8522–8667`** — the 7 construction sites (the ensure-fn validation block).
- **`tests/diagnostics/probe_arc296_s7_ensure_reason_enum.{rs,wat}`** — the committed RED probe (`#[ignore]`'d) + fixture.
  UN-IGNORE it and make it GREEN. It drives the arg-type-mismatch fixture and asserts `:reason` is a `#wat.kernel/
  ArgTypeMismatch {:arg-type … :clause-return-type …}` tagged value.

## The enum + the site→variant map (grounded — reproduce EXACTLY)
```rust
// src/check/error.rs — beside CheckErrorKind
#[derive(Debug, Clone, wat_macros::ToEdn)]
pub enum EnsureFnInvalidReason {
    NotFnForm,                                                        // sites 8531, 8565
    ArityNotOne { got: usize },                                       // site 8607
    ArgTypeMismatch { arg_type: String, clause_return_type: String }, // site 8622
    ReturnTypeNotBool { got: String },                               // site 8638
    MalformedSignature,                                              // sites 8650, 8660
}
```
| current site (`src/check.rs`) | current `reason:` value | → variant |
|---|---|---|
| 8534 (`_` non-fn head) + 8568 (`_` non-list) | `"must be :wat::core::fn form"` | `NotFnForm` |
| 8610 | `format!("arity must be 1 (one parameter for the result); got {}", param_names.len())` | `ArityNotOne { got: param_names.len() }` |
| 8625 | `format!("arg type must match clause return type: :ensure :fn takes `{}` but clause returns `{}`", format_type(arg_ty), format_type(&clause_ret))` | `ArgTypeMismatch { arg_type: format_type(arg_ty), clause_return_type: format_type(&clause_ret) }` |
| 8641 | `format!("return type must be :bool; got `{}`", format_type(&ret_type))` | `ReturnTypeNotBool { got: format_type(&ret_type) }` |
| 8653 (Err parse) + 8663 (None, <3) | `"malformed :fn signature — expected (:wat::core::fn [param <- :T] -> :bool body)"` | `MalformedSignature` |

The two type strings in `ArgTypeMismatch` are the SAME `format_type(...)` calls as today, just stored in two fields
instead of `format!`'d into one — no data is lost or reshaped, only un-flattened.

## Display (byte-for-byte — the human face MUST NOT change)
Add `impl std::fmt::Display for EnsureFnInvalidReason` reproducing each string EXACTLY:
- `NotFnForm` → `must be :wat::core::fn form`
- `ArityNotOne { got }` → `arity must be 1 (one parameter for the result); got {got}`
- `ArgTypeMismatch { arg_type, clause_return_type }` → ``arg type must match clause return type: :ensure :fn takes `{arg_type}` but clause returns `{clause_return_type}` ``
- `ReturnTypeNotBool { got }` → ``return type must be :bool; got `{got}` ``
- `MalformedSignature` → `malformed :fn signature — expected (:wat::core::fn [param <- :T] -> :bool body)`

The outer `EnsureFnInvalid` Display arm (`error.rs:671`) is UNCHANGED — it already interpolates `{}` on `reason`, which
now Displays via the enum. So the full human sentence is byte-identical to HEAD.

## Implementation sketch (fill it — don't reinvent the shape)
1. `src/check/error.rs` — add the `EnsureFnInvalidReason` enum + its `Display`; change the field
   `reason: String` → `reason: EnsureFnInvalidReason`.
2. `src/check.rs` — the 7 sites: `reason: "…".into()` / `reason: format!(…)` → `reason:
   EnsureFnInvalidReason::<Variant> { … }` per the table.
3. `tests/diagnostics/probe_arc296_s7_ensure_reason_enum.rs` — delete the `#[ignore]`. It must go GREEN.
4. Full gate.

## Proof
- The RED probe (un-ignored) → GREEN: `:reason` is `#wat.kernel/ArgTypeMismatch {:arg-type … :clause-return-type …}`.
- **Display byte-identical:** any existing check-family Display / snapshot test stays GREEN untouched. If a test
  asserted the old `:reason "…"` STRING on the EDN wire, update it to the structural form (intended change).
- FULL gate `cargo nextest run --release` = 0 failed; `cargo build --release` clean (warning delta ~0).

## Blast radius (STOP + report if you exceed this)
`src/check/error.rs` (enum + Display + field) · `src/check.rs` (the 7 sites) · the probe (un-ignore) · any
`--check-output`/golden test asserting the old `:reason` string. NOTHING else.

## STOP triggers (REJECTION criteria — ship nothing, report the gap; NOT permission to defer)
- **STOP-1:** if an 8th `EnsureFnInvalid` construction site exists (grep `EnsureFnInvalid` in `src/`) OR a site's reason
  does NOT map cleanly to one of the 5 variants, STOP and report it — do NOT invent a 6th "catch-all prose" variant.
- **STOP-2:** if `#[derive(…ToEdn)]` on the enum does not compile (a field type lacks `ToEdn`), STOP and report — do
  NOT hand-write an `impl ToEdn`.
- **STOP-3:** if making the probe GREEN needs any change OUTSIDE the blast radius, STOP and report what breaks.

## ⛔ THE ANTI-WEAKENING RULE (non-negotiable — PROBATIO FLEXA MENTITVR)
A probe is NEVER yours to weaken to reach green. Do not invert an assertion, relax the `matches!(Tagged)` check,
re-`#[ignore]` the test, or soften the `:arg-type`/`:clause-return-type` field checks. If a probe goes red, the CODE is
wrong — fix the code, or STOP and report. The orchestrator weighs the **emitted diff + the wire EDN**, not your report.

## Report back
The enum + Display diff; the 7-site diff; the probe un-ignore + its GREEN output (paste the emitted `:reason` EDN); the
FULL gate count (`cargo nextest run --release`); `cargo build --release` warning delta vs HEAD; any STOP; any deviation.
