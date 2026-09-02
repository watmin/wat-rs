# BRIEF — STONE 1a-β-ii: the last three register, and the first hand-list dies

Register `def`, `defmacro`, `defalias` — the last three names in
`freeze::is_liftable_declaration_head` without a `Declare` impl — then flip its one caller to ask the
registry, **delete the predicate**, and **delete the meter that measured it**. This is the campaign's
first hand-list kill.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1a-beta-ii-the-last-three-and-the-first-hand-list-dies.md`
— its ★★★ contract decision governs the whole stone.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5124.

## Read in order

1. **The DESIGN**, especially ★★★ *"a REFUSAL is not an IMPLEMENTATION"* — it decides `def`'s roles
   and its purity, and the two are linked.
2. **`src/intrinsic/special/defsurface.rs`** and **`structtype.rs`** — the row template, and the
   `Declaration`/`Unevaluated` grounds your three share.
3. **The three declare-time processors:**
   ```
   def       src/declare/register.rs:152   register_defines
   defmacro  src/macros/parse.rs:83        parse_defmacro_form
   defalias  src/declare/parse.rs:219      parse_defalias_form
   ```
4. **`src/check.rs:7978`** — `infer_def`, `def`'s OWN check impl (reached from `infer_list:2610`).
   ⚠ Not the shared silent-accept arm at `check.rs:4865` — that one covers nine names and is not
   `def`'s; leave it alone.
5. **`src/runtime.rs:2132`** — `def`'s refusal arm. **Read it, then read the DESIGN's contract
   again.** It does NOT get `role = eval`.
6. **`src/freeze.rs:1967`** — `is_liftable_declaration_head`, the predicate you are deleting, and its
   ⛔ RENAMED block explaining why it has that name.
7. **`src/closure_extract.rs:2578`** — `split_body_prelude`, its one production caller.
8. **`src/intrinsic/mod.rs:2829`** — `liftable_declaration_head_missing_and_foreign`, the meter you
   are deleting once it reads empty.

## The work

### 1 — three doc-only structs

`@Category Declaration`, `@Purity Unevaluated`, `@Determinism Deterministic`, `@Totality Partial`,
`@ExpandTime RuntimeOnly` — verify each per form rather than inheriting, and argue each ground,
citing `defsurface`/`structtype` where the argument is identical. FQDN-headed `@syntax`, verified by
`--check`ing a concrete instantiation.

### 2 — the annotations

```
role = declare   on all three processors
role = check     on infer_def          ← def only; the other two have no check impl of their own
```

⛔ **No `role = eval` on anything.** See STOP-1.

### 3 — three debt-ledger names

`def`/`defmacro`/`defalias` join `FROZEN_CHECKER_DEBT_LEDGER` for the same reason the previous five
did, unless `check_env.get` returns `Some` for one of them — **check, do not assume**. The existing
entries carry the reasoning; extend it, do not retype it.

⚠ `:wat::core::def` is also in `REGISTRY_MEMBERSHIP_GAP_B` and `KNOWN_UNREVIEWED`. Those are
bidirectional ratchets: registering it makes both go **STALE** until its name is deleted from each.
Report both counts before and after.

### 4 — the kill

- `split_body_prelude` asks the registry. **The accessor lives in `src/intrinsic/`**, with the
  registry — not a new predicate in `freeze.rs` under a different name.
- **`is_liftable_declaration_head` is DELETED**, not rewritten as a wrapper.
- **The meter is DELETED** once you have confirmed it reads MISSING empty. ⚠ Confirm first, delete
  second, and say in your report which order you did it in.
- `tests/macros/probe_declaration_form_lift.rs` — its membership half tested the predicate. Retire
  what no longer has a subject; **keep anything that tests the LIFT itself**, which still exists.

## Blast radius

`src/intrinsic/special/` (+3 files, +3 mod lines) · `src/declare/register.rs` ·
`src/declare/parse.rs` · `src/macros/parse.rs` · `src/check.rs` (one annotation) ·
`src/freeze.rs` (one fn deleted) · `src/closure_extract.rs` (one call flipped) ·
`src/intrinsic/mod.rs` (accessor, ledger, meter deleted) · `src/rete/purity.rs` (one ledger name) ·
`tests/macros/probe_declaration_form_lift.rs`. No `.wat` corpus change.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `runtime.rs:2132` does NOT get `role = eval`.** It is a refusal, not an
implementation: `role = eval` claims *"here is the code that evaluates this form"*, and that arm says
the form cannot be evaluated. Annotating it would make `show-source :wat::core::def` present an
error-raiser as the evaluator, AND it would break `@Purity Unevaluated` — the gate
`unevaluated_purity_carries_no_route_to_evaluation` would fire, correctly.

**⛔ STOP-2 — do not give `def` `@Purity Effectful`.** Measured: `:wat::core::` is not one of
`effectful_by_prefix`'s eight prefixes, so `Effectful` re-opens
`declared_purity_vs_effectful_by_prefix_census` — the exact red `Purity::Unevaluated` was minted to
close. If you believe `Unevaluated` is wrong for `def`, STOP and report; do not route around it.

**⛔ STOP-3 — `is_liftable_declaration_head` is DELETED, and so is the meter.** A predicate about
what a NAME is, kept in `freeze.rs` as a registry-query wrapper, leaves the misplaced authority the
hand-list was only half of. A meter whose domain no longer exists asserts over an empty set and can
never fail. **The floor's total is expected to go DOWN by one. That is the stone working.**

**⛔ STOP-4 — the LIFT must still work, and must still discriminate.** `split_body_prelude` exists to
lift a leading declaration prelude out of a fn body. Verify both directions with the pre-built
binary: a body whose prefix IS declarations still lifts; a body whose first form is NOT a declaration
still does not. **A registry query that returns `true` for everything would pass every test that only
checks the positive case.**

**⛔ STOP-5 — do not touch `is_mutation_form`/`is_mutation_head`, `DECLARATION_HEADS`,
`declare::is_declaration_form`, or `check.rs:4865`'s shared arm.** Four separate questions, three
already measured and ruled on, one unmeasured.

**STOP-6 — verbatim otherwise.**

## Sabotage — report each as "predicted red, unverified"

1. give `def` `role = eval` on the refusal arm → what does the `Unevaluated` gate say?
2. make the new accessor return `true` unconditionally → does anything fail? (this is STOP-4's
   discriminator; if nothing fails, **say so** — that is a finding about test coverage, not a pass)
3. leave `:wat::core::def` in `REGISTRY_MEMBERSHIP_GAP_B` after registering → what does the Gap B
   ratchet say?

## Report

The three doc structs verbatim · the four annotations and their targets · **the accessor verbatim and
where you put it** · `split_body_prelude`'s new call · confirmation the predicate and the meter are
gone, **and in which order you confirmed-then-deleted** · MISSING/GAP_B/KNOWN_UNREVIEWED/DEBT before
and after · **both directions of the lift, verified with the binary** · what you retired from the
probe file and what you kept · the three sabotage predictions · and what surprised you.

## Prior comparable

`BRIEF-STONE-1a-beta-i-the-type-declaration-family.md`. `src/intrinsic/special/structtype.rs` is the
row standard.
