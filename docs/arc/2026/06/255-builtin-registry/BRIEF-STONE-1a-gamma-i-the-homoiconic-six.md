# BRIEF — STONE 1a-γ-i: the six homoiconic verbs that really evaluate

Register `quote`, `quasiquote`, `macroexpand`, `macroexpand-1`, `forms`, `struct->form` — a **fourth
shape**, distinct from the three the loaders and no-ops taught us: these reach the evaluator, do real
work, and **return a real value**. They are not `Unevaluated` and they are not no-ops.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-delta-and-epsilon-three-shapes-of-not-really-evaluating.md`
for the three shapes these are NOT. There is no separate design for this stone: the shape is ordinary
(check + eval, returning a value) and the work is mechanical. **The judgement is entirely in the five
axis rulings per row.**

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5123.

## ★★ What the last stone learned, so this one does not rediscover it

Registering a `role = eval` gives the row a **handler**. The gate
`registry_first_door_owns_every_handler_row_no_literal_arm_survives` then **demands the literal arm
in `dispatch_keyword_head_value` be deleted**, because the registry-first door answers the name
first and the arm can never fire. **That is expected here, for all six. Delete each arm as part of
the stone**, leaving a retirement comment in the shape `runtime.rs` already uses. Do not wait for the
gate to tell you.

★ And a landmine already paid for: **`role = eval` CANNOT be stacked on one fn** — the generated shim
is named from the fn identifier, so two FQDNs on one eval fn is a duplicate-symbol compile error.
`macroexpand`/`macroexpand-1` have separate eval fns, so this should not bite; if you find any two
rows sharing one eval fn, give each its own delegate.
(`[[NOTE-role-eval-cannot-stack-and-the-error-does-not-say-so]]`)

## Read in order

1. **`src/intrinsic/special/use_form.rs`** — the row template from the last stone, and the pattern for
   wiring a check arm to a named delegate.
2. **The six eval fns — already named, already one-line delegates from the arm:**
   ```
   quote          runtime.rs:2183  → eval_quote
   quasiquote     runtime.rs:2195  → eval_quasiquote
   struct->form   runtime.rs:2196  → reflect::render::eval_struct_to_form
   forms          runtime.rs:2339  → reflect::r#match::eval_forms
   macroexpand-1  runtime.rs:2340  → reflect::expand::eval_macroexpand_1
   macroexpand    runtime.rs:2341  → reflect::expand::eval_macroexpand
   ```
3. **The five check arms — INLINE bodies, not delegates** (`macroexpand`/`-1` share one):
   ```
   check.rs:3289  quote          ≈14 lines
   check.rs:3353  forms          ≈11 lines
   check.rs:3364  struct->form   ≈18 lines
   check.rs:3850  macroexpand*   ≈26 lines
   check.rs:4889  quasiquote     ≈ 8 lines
   ```
4. **`src/rete/purity.rs`'s `KNOWN_UNREVIEWED`** — **all six are on it** and all six must leave.

## The work

### 1 — six doc-only structs

Five axes each, **argued per row**. These six do genuinely different things and should NOT share a
table: `quote` returns its argument unevaluated; `quasiquote` evaluates the unquoted sub-forms;
`macroexpand` runs macro expansion; `forms` and `struct->form` read a value's structure.

⛔ **`@Purity` is the one to get right, and it may STOP you** — see STOP-1.

### 2 — `role = eval` on the six existing fns, and delete the six arms

### 3 — `role = check`: extract each arm body to a named fn, arm delegates

The same wiring `:wat::core::use!` got last stone: the body moves to a named fn in its impl home, the
`infer_list` arm becomes a call, and the fn carries the annotation. **A `role = check` naming a fn
nothing calls is a registry pointing at dead code** — clippy caught exactly that last stone.

⚠ `macroexpand`/`macroexpand-1` share one arm: extract one fn, stack two `role = check` annotations
on it (legal — `check` emits source only; `src/check.rs:15553` is the precedent).

### 4 — the ledgers

`KNOWN_UNREVIEWED` −6. `FROZEN_CHECKER_DEBT_LEDGER`: check `register_builtins` per row before adding.
Report both before and after.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — if a row is honestly `@Purity Effectful`, STOP AND REPORT. Do not force it, and do not
downgrade it to keep the floor green.** All six are `:wat::core::`, which is NOT one of
`effectful_by_prefix`'s eight prefixes — so a row declaring `Effectful` re-opens
`declared_purity_vs_effectful_by_prefix_census`, the red that `Purity::Unevaluated` was minted to
close and that the campaign has not yet solved for evaluating verbs
(`[[NOTE-the-prefix-guess-has-run-out-of-road]]`). ★ `macroexpand` is the likely one: expansion can
run arbitrary macro bodies. **A blocked row reported is worth more than five shipped and one lied
about.**

**⛔ STOP-2 — verify each row reaches its eval arm and RETURNS.** These are shape ④, not ①/②/③.
Confirm per form with the binary. A row that turns out to be refused, unreachable, or a no-op belongs
to a different shape — report it rather than forcing the table.

**⛔ STOP-3 — extract the check bodies VERBATIM.** Move them; do not improve them. A behaviour change
smuggled into an extraction is invisible in a diff that is expected to move code.

**⛔ STOP-4 — do not touch the shared silent-accept arm at `check.rs:4880-4884`** (`unquote`,
`unquote-splicing`, the loaders, …). Those rows are 1a-γ-ii and are a different shape — they have no
eval arm at all.

**⛔ STOP-5 — every `@syntax` FQDN-headed**, verified by `--check`ing a concrete instantiation.

**STOP-6 — verbatim otherwise.**

## Sabotage — report each as "predicted red, unverified"

1. leave one literal eval arm in place after annotating its fn → what does the registry-first-door
   gate say?
2. drop one `role = check` → what does the gate say? (⚠ these rows are not `Unevaluated`, so the
   gate's `else` branch is the one that fires — name it correctly)
3. leave one of the six on `KNOWN_UNREVIEWED` → what does that ledger's STALE check say?

## Report

The six doc structs verbatim · **the `@Purity` ruling per row, with its ground** · the six deleted
arms and their retirement notes · the five extracted check fns and the arms that now delegate ·
each row's `@syntax` and the instantiation you `--check`ed · `KNOWN_UNREVIEWED` and `DEBT`
before/after · the three sabotage predictions · and what surprised you.

## Prior comparable

`BRIEF-STONE-1a-epsilon-the-no-ops.md`. `src/intrinsic/special/use_form.rs` is the row and
check-wiring standard.
