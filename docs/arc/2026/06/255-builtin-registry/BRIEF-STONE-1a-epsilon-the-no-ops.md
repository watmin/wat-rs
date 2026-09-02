# BRIEF — STONE 1a-ε: the three no-ops join the registry

Register `:wat::core::use!`, `:wat::config::set-redef!`, `:wat::config::set-eval-redef!` — **shape ③**
of the DESIGN's three: forms that reach the evaluator and return `Unit`, because their real work
already happened at freeze. They are **not** `Unevaluated`; they evaluate, they just do nothing.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-delta-and-epsilon-three-shapes-of-not-really-evaluating.md`
— read the ★★★ three-shape finding and the ★★ contract.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5123.

## ★★ The gate changed one stone ago, and it changes what these rows owe

`every_special_form_carries_check_and_eval_impls` no longer asks `@Category`. It asks
**`@Purity Unevaluated`**. These three are `Pure`, so the gate demands **`role = check` AND
`role = eval`** — and does NOT demand `role = declare`. A declare annotation on them is an optional
reflection improvement, not a requirement; add one only where a real freeze-time processor exists and
you can name it honestly.

## Read in order

1. **The DESIGN's three shapes**, and `src/intrinsic/special/load_file.rs` for the row shape.
2. **`src/runtime.rs:2120`** — the setters' shared eval arm, and its own comment explaining WHY it is
   a no-op (*"the flag has already been processed at freeze time"*). **`:2947`** — `use!`'s.
3. **`src/check.rs:2706`** (`infer_config_set_bool`, the setters' check) and **`:4766`** (`use!`'s
   own check arm).
4. **`src/config.rs:503`/`:521`** (`collect_entry_file_inner`) and
   **`src/declare/register.rs:1770`/`:1778`** (`register_runtime_defs_form`) — **two** candidate
   freeze-time processors for the setters. See STOP-3.
5. **`src/intrinsic/mod.rs`'s `purity_mandated_examples`** — the mandate that makes `Pure` the harder
   choice here, and the reason each of these needs an example that actually runs.
6. **`src/rete/purity.rs`'s `KNOWN_UNREVIEWED`** — `:wat::core::use!` is on it. Registering a purity
   for it makes that ledger go STALE until the name is deleted.

## The work

### 1 — three doc-only structs

`@Purity Pure` · `@Determinism` · `@Totality` · `@ExpandTime` · `@Category` · `@added` · prose ·
FQDN-headed `@syntax` · `@ret`.

⬜ **`@Category`** — argue it. `:Ambient`? `:Declaration`? Something else? A config setter mutates a
per-runtime flag at freeze; `use!` names a dependency. **They may not share a category** — say so if
they do not, with reasons. ⚠ The gate no longer forces any category, so this is a free and honest
choice for the first time in this campaign. Do not inherit `Declaration` out of habit.

★ **The `@Purity Pure` ground must argue the SHAPE**, not just assert the pole: every consumer of
`@Purity` asks a runtime question; these forms DO reach the evaluator and return `Unit`; nothing
observable happens at that moment; therefore `Pure`. Contrast `Unevaluated`, which would be a lie —
they are evaluated.

### 2 — the annotations the gate demands

`role = check` and `role = eval` on each. ⚠ The setters share both arms with each other — stacking
two `#[wat_special_form_impl]` on one fn is proven precedent (`src/check.rs:15553`).

### 3 — runnable `@example` per row

`purity_mandated_examples` requires it for a `Pure` + `Deterministic` row. **Write one and RUN it**
with the pre-built binary; report the exact form and its output. An `@example-norun` here is a
retreat, and if a row genuinely cannot be run, that is a finding that its shape was misread — STOP
and report rather than downgrading to `@example-norun`.

### 4 — the ledgers

`KNOWN_UNREVIEWED`: `:wat::core::use!` must leave. `FROZEN_CHECKER_DEBT_LEDGER`: check
`register_builtins` per row before adding — a row with a `CheckEnv` scheme does not join. Report both
counts before and after.

## Blast radius

`src/intrinsic/special/` (+3 files, +3 mod lines) · `src/runtime.rs` · `src/check.rs` ·
possibly `src/config.rs`/`src/declare/register.rs` (see STOP-3) · `src/intrinsic/mod.rs` ·
`src/rete/purity.rs`. No `.wat` corpus change.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — these are NOT `@Purity Unevaluated`, and do not make them so to dodge the example
mandate.** Measured, by running them: both forms in expression position return and the program
completes. `Unevaluated` means the axis has no runtime verdict to give; *"nothing happens"* is a
verdict.

**⛔ STOP-2 — verify shape ③ per form.** Confirm each actually reaches its eval arm rather than
trusting the DESIGN's table. If one of the three turns out to be refused or unreachable, it is shape
① or ② — a finding, not a row to force.

**⛔ STOP-3 — the setters have TWO candidate freeze-time processors** (`collect_entry_file_inner`,
`register_runtime_defs_form`). `role = declare` is OPTIONAL for these rows. **Annotate one only if
you can say which is the honest primary, and why.** If both genuinely process the form, annotate
both and say so — stacking is legal. **If you cannot tell, annotate neither and report it**; a role
naming the wrong fn is a false answer from the sole authority, and two stones ago exactly that
shipped.

**⛔ STOP-4 — `use!` appears to have no freeze-time processor at all.** Confirm that yourself. If
true, it carries check + eval and no declare, and that is correct — do not invent one.

**⛔ STOP-5 — do not touch the eval or check arms themselves.** You are annotating existing code, not
reshaping it. `runtime.rs:2120`'s no-op stays a no-op.

**STOP-6 — verbatim otherwise.**

## Sabotage — report each as "predicted red, unverified"

1. change one row to `@Purity Unevaluated` → what do the gate and
   `unevaluated_purity_carries_no_route_to_evaluation` say? (⚠ two tests may fire; name both)
2. drop one `role = eval` → what does the gate say?
3. replace a runnable `@example` with `@example-norun` → what does `purity_mandated_examples` say?

## Report

The three doc structs verbatim · **your `@Category` argument per row, and whether they differ** ·
the annotations and their targets · **each `@example` and the output you got when you RAN it** ·
STOP-3 answered explicitly · `KNOWN_UNREVIEWED` and `DEBT` before/after · the three sabotage
predictions · and what surprised you.

## Prior comparable

`BRIEF-STONE-1a-delta-the-loaders.md` — the sibling stone, shape ②. Same report shape.
