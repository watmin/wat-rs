# BRIEF — STONE: the hand-rolled arms retire, and every unevaluated form gets the named error

Two per-keyword arms in `runtime.rs` give `def` and `defclause` a named refusal. Seven sibling forms
were never added and get `UnknownFunction` — told they do not exist. Replace both arms with one
guard keyed on `@Purity Unevaluated`, and rewrite the message so it is true for all eleven.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-hand-rolled-arms-retire-and-every-unevaluated-form-gets-the-named-error.md`
Prior art it executes: `NOTE-declaration-position-class-guard.md` (2026-06-24) — **step 3**, deferred
until the registry could answer. It can now.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5123.

## Read in order

1. **The DESIGN**, especially the contract (the predicate) and the message section.
2. **`docs/.../NOTE-declaration-position-class-guard.md`** — the 2026-06-24 plan you are executing.
   Its *"Why a hand-rolled list is the WRONG cure"* section names the anti-pattern to refuse.
3. **`src/runtime.rs:1990`** — the registry-first door in `dispatch_keyword_head_value`, and
   **`:1898`** — the same door in `dispatch_keyword_head`. **Both need the guard.**
4. **`src/runtime.rs:2140` and `:2150`** — the two hand-rolled arms (`def`, `defclause`) you are
   deleting.
5. **`src/value/signal.rs:296`** (the variant) and **`:707`** (the Display text you are rewriting).
6. **`src/intrinsic/mod.rs`** — `unevaluated_purity_carries_no_route_to_evaluation`, which already
   proves every `Unevaluated` row has no handler. That is WHY these rows fall past the registry-first
   door and land in the unknown-function fallback.

## The work

### 1 — one guard, both doors

After the registry-first handler lookup fails and before the literal `match head`:

```
registry().lookup_entry(head) is Some AND its purity == Unevaluated
    ⇒ RuntimeErrorKind::DeclarationInExpressionPosition(head)
```

⛔ **No name list.** The predicate is the row's own declared purity. If you find yourself writing a
`matches!` or a `const` of FQDNs, stop — that is the exact cure the 2026-06-24 note refused.

### 2 — delete the two hand-rolled arms

`def` at `:2140`, `defclause` at `:2150`. Leave retirement comments in the shape `runtime.rs` uses.

### 3 — rewrite the Display message

The current text says *"declaration forms are top-level registration forms"* — **false for the three
loaders**, which are `@Category Splice` and register nothing. Write a message true of every
`Unevaluated` row: consumed before evaluation, registered **or spliced** at freeze time, never
evaluated, so it cannot appear in expression position.

⚠ **Keep the variant NAME.** 20 sites, two of them `.wat` corpus fixtures asserting the EDN tag. A
rename is a user-visible surface change and a corpus migration; the message is what carries the
truth.

## Blast radius

`src/runtime.rs` (two guards added, two arms deleted) · `src/value/signal.rs` (the Display text).
**Nothing else.** No `.wat` change, no registration, no ledger movement.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — the predicate is `@Purity Unevaluated`, never `@Category`.** Measured counterexample:
`:wat::core::use!` is `@Category Declaration` **and** legally evaluates to `Unit`
(`use_form.rs:76-77`, `runtime.rs` arm deleted at Stone 1a-ε). A category-keyed guard would refuse a
form that works today. If keying on purity appears not to cover something you expected, STOP and
report — do not add a second key.

**⛔ STOP-2 — a made-up head must STILL say `UnknownFunction`.** `(:wat::core::zorble 1)` does not
exist and `UnknownFunction` is the honest answer for it. **Verify this with the pre-built binary
before and after your change is conceptually applied**, and report both. A guard that refuses every
unrecognised head would satisfy every other acceptance row while being broken.

**⛔ STOP-3 — do not touch the checker.** These forms still type-check clean in expression position.
Making the refusal static is a larger question and is not this stone.

**⛔ STOP-4 — both doors, or report why one.** `dispatch_keyword_head` and
`dispatch_keyword_head_value` each carry the registry-first lookup. If you conclude only one needs
the guard, say so with the reason rather than silently doing one.

**STOP-5 — verbatim otherwise.**

## Sabotage — report each as "predicted red, unverified"

1. key the guard on `@Category Declaration` instead → what happens to `:wat::core::use!`?
2. remove the `Unevaluated` condition (fire for every registry row) → what breaks first?
3. leave one hand-rolled arm in place → does anything notice? ⚠ If nothing does, **say so** — that
   is a coverage finding, not a pass.

## Report

The guard verbatim and where you put it in **each** door · the two deleted arms and their retirement
notes · **the new Display text verbatim** · your before/after `zorble` check with the binary · the
three sabotage predictions · and what surprised you.

★ **The report must state, per form, what each of the eleven now raises** — `def`, `defmacro`,
`defenum`, `newtype`, `typealias`, `defalias`, `defsurface`, `structtype`, and the three loaders.
Eight of those currently say `UnknownFunction`; that is the defect being closed and the list is how
we see it closed.

## Prior comparable

`BRIEF-STONE-1a-epsilon-the-no-ops.md`. The 2026-06-24 note is the design.
