# BRIEF — STONE 1a-β-0b: a form that never evaluates gets a purity pole of its own

Add `:Unevaluated` — a fourth `Purity` pole for a form that is **never evaluated**, so the axis has
no runtime verdict to give — in wat, where `Purity` is generated from. Widen the five hand-written
messages that enumerate the poles, and **gate that list the way `Category`'s already is**. Then give
`:wat::core::defsurface` the pole it needs, and gate the pole itself so it cannot be claimed by a
form that does evaluate.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-beta-0b-a-form-that-never-evaluates.md`.
Measurement: `NOTE-the-prefix-guess-has-run-out-of-road.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

⚠ **THE TREE IS DIRTY AND RED, DELIBERATELY.** Stone 1a-β-0 (`SpecialFormRole::Declare` +
`:wat::core::defsurface`) is uncommitted in the working tree and the floor is red at **exactly one**
test — `intrinsic::tests::declared_purity_vs_effectful_by_prefix_census` — which is the blocker this
stone exists to dissolve. That work is correct and stays; you are building on top of it. Do not
revert it, and do not treat that red as yours to route around.

## Read in order

1. **The DESIGN above** — the contract decision and the two gates are pinned there.
2. **`wat/runtime-meta.wat:37-45`** — the `Purity` `defenum` and its per-variant `;;` prose. **This
   is the source of truth**; the Rust enum is generated from it. Read `Preserving`'s prose closely —
   yours is its sibling and should read like it.
3. **`crates/wat-doc/src/lib.rs:49-93`** — how `Purity`/`Category` are generated, and the ⛔ comment
   explaining why there is no Rust list to forget.
4. **`crates/wat-doc/src/lib.rs:73-77`** (`CATEGORY_LEGAL_VALUES`) and **`:2059`**
   (`category_message_lists_every_variant`) — **the exact pattern for your message gate**: a
   hand-written enumeration, and a test that holds it against `variants()`.
5. **`src/rete/purity.rs:474` · `:2109` · `src/intrinsic/mod.rs:2188` · `src/intrinsic/reflect.rs:84`**
   — the four consumers. **Read them to VERIFY they need no edit.** You are not changing them.
6. **`src/intrinsic/special/defsurface.rs`** — the row whose Purity ground you are rewriting.
7. **`src/intrinsic/mod.rs`**, the ratchets around `every_special_form_carries_check_and_eval_impls` —
   the shape for your structural gate.

## The work

### 1 — the pole, in the source of truth

`:Unevaluated` in `wat/runtime-meta.wat`'s `:wat::runtime::Purity`, with `;;` prose in the voice of
its three siblings. Say what it means — *the form is never evaluated, so the axis has no runtime
verdict* — and, briefly, how it differs from `Preserving` (which has no purity of its own because it
inherits one).

### 2 — the five hand-written messages

```
crates/wat-macros/src/wat_intrinsic.rs:645   "(known: Pure, Effectful, Preserving)"
crates/wat-macros/src/wat_intrinsic.rs:657   "known: Pure, Effectful, Preserving"
crates/wat-doc/src/lib.rs:692   :987   :1424 "value must be one of: Pure, Effectful, Preserving"
```

**Find them yourself and report the count you found** — the three line numbers above are where I saw
them, and a line number drifts. All must list the new pole.

### 3 — the message gate (this is the extirpare rung, not a nicety)

Those five are hand-written and **nothing holds them to the enum**. `Category` has exactly this
problem and solved it: one `const` + `category_message_lists_every_variant`. Do the same for
`Purity` — hoist the repeated literal to a single `const` if that is the smaller change, and add a
test asserting every `Purity::variants()` entry appears in it. **A sixth message added later must go
red, not rot.**

### 4 — the structural gate on the pole

A row declaring `@Purity Unevaluated` must have **no `handler`, no `tail_handler`, and no `Eval` or
`Tail` impl role** — any of the four is a route to evaluation, so the claim would be false. Walk
`registry().all_entries()`, collect offenders by name, one assert naming them.

⚠ **State in the test's own doc what it cannot see**: a hand-written `runtime.rs` match arm is not a
registry fact and this gate does not reach it. Claiming otherwise would be the containment argument
that does not name its consumers.

### 5 — `defsurface` takes the pole

`@Purity Effectful` → `@Purity Unevaluated`, and **rewrite the Purity ground to argue the new
claim**. The existing ground argues a registry mutation is an effect; the new one must argue that all
four consumers of `@Purity` ask a runtime question and this form has no runtime. Do not leave the old
paragraph with a new label on it.

## Blast radius

`wat/runtime-meta.wat` (one form) · `crates/wat-macros/src/wat_intrinsic.rs` ·
`crates/wat-doc/src/lib.rs` · `src/intrinsic/mod.rs` (two gates) ·
`src/intrinsic/special/defsurface.rs`. **No `.wat` corpus change beyond that one defenum** — this is
a variant addition to one form in one file, not a structural migration, so the wat-fix codemod path
does not apply.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — do NOT edit the four consumers.** `rete/purity.rs:474`, `:2109`,
`intrinsic/mod.rs:2188`, `reflect.rs:84` are `matches!` on accepted poles and already compute the
right answer for an unmatched variant. If any one of them appears to need an edit, that is a finding
that contradicts the DESIGN's central measurement — STOP and report it.

**⛔ STOP-2 — the pole is named for the FORM's condition, never a moment.** Not `FreezeTime`, not
`Declarative`. `runtime-meta.wat`'s own axis discipline is *"the DOING, not the moment it happens"*,
and `Declaration` is `Category`'s word.

**⛔ STOP-3 — do not touch `effectful_by_prefix`, `is_effectful_op`, or the census test.** The census
must go green because `defsurface` stopped declaring `Effectful`, NEVER because the gate was
loosened. If you find yourself editing `declared_purity_vs_effectful_by_prefix_census`, stop.

**⛔ STOP-4 — do not change `defsurface`'s `Determinism` or `Totality`.** They describe the
freeze-time pass, they are true, and symmetry with the Purity pole is a ruling nobody made.

**⛔ STOP-5 — do not revert or modify Stone 1a-β-0's uncommitted work** (the `Declare` role, the
macro arm, the derived gate, `show-source`'s order, the `defsurface` row's other axes, the
`FROZEN_CHECKER_DEBT_LEDGER` line). It is correct and it is this stone's foundation.

**STOP-6 — verbatim otherwise.** No signature tidying, no opportunistic cleanup.

## Sabotage — report each as "predicted red, unverified"

1. give `defsurface` a `role = eval` annotation → what does the structural gate say?
2. remove one pole from the widened message list → what does the message gate say?
3. strip `:wat::kernel::` from a kernel row's name so the prefix misses it while it still declares
   `Effectful` → does the census still fire? (proves STOP-3 held — the census kept its teeth)

## Report

The wat `defenum` diff verbatim · every message you found and widened, **with the count** · the
message gate verbatim · the structural gate verbatim, including its cannot-see note · **the rewritten
Purity ground verbatim** · confirmation each of the four consumers needs no edit, **with the reason
per site** · the three sabotage predictions · and what surprised you.

## Prior comparable

`BRIEF-STONE-1a-beta-0-the-third-regime-gets-its-name.md` — the stone directly beneath this one, same
report shape. `crates/wat-doc/src/lib.rs:73-77` + `:2059` is the message-gate pattern to copy.
