# BRIEF — STONE A-2-ii-b-1: the Option/Result constructors get homes

Home and rule `:wat::core::Some`, `:wat::core::Ok`, `:wat::core::Err` — the three Option/Result
constructors `meter-2` made visible and parked. `Some` is the fourth and last verb blocking a record
accessor from classifying pure. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-A-2-ii-b-1-the-option-result-constructors-get-homes.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the three rulings with their grounds, and the **predicted red**.
2. `src/intrinsic/option.rs` and `src/intrinsic/record.rs` — **the template**, written last stone:
   a thin `#[wat_intrinsic]` delegate over an existing named fn, with a full directive block and
   grounding prose per axis. `option.rs` is also where `Some` belongs.
3. `src/runtime.rs:15023,15047,15071` — `eval_some_ctor` / `eval_ok_ctor` / `eval_err_ctor`, the three
   bodies you are delegating to. Read each; they are ~20 lines.
4. `src/runtime.rs`, `fn eval_list` around `:5174` — the keyword-guard arms that come out.
5. `src/intrinsic/mod.rs`, `FROZEN_CHECKER_DEBT_LEDGER` — read the `Option/expect` row added last
   stone, and the `nth`/`reverse` rows below it. Yours should read like them.

## The work

### 1 — home the three

One thin `#[wat_intrinsic]` delegate each, calling straight into the existing named fn. **Bodies do
not move.** Declare the real arity (1 each) so the hand-rolled `args.len() != 1` guards retire.
Remove their three keyword-guard arms from `eval_list`.

`Some` belongs beside `Option/expect` in `src/intrinsic/option.rs`. Put `Ok`/`Err` where the
codebase's own organisation says they go — if that means a new `src/intrinsic/result.rs`, follow how
`option.rs`/`record.rs` were created last stone.

### 2 — the rulings

All three: **Pure · Deterministic · Total**. Each body is an arity check, then `eval_inner` on one
argument, then a wrap that cannot fail. The arity check retires on homing, leaving no failure path.

Write the grounding prose per axis, as the template does, citing what you read.

⚠ **`Err` is a constructor, not a failure.** `(Err v)` *builds* a `Result`; it does not raise. Under
`RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`, a matchable error-bearing arm is
exactly the shape the ruling calls **total**. Do not let the name pull the ruling toward `Partial`.

### 3 — the two ledgers

- Delete the three `KNOWN_UNREVIEWED` rows in `src/rete/purity.rs` (52 → 49). The ratchet requires it.
- **Add three `FROZEN_CHECKER_DEBT_LEDGER` rows** in `src/intrinsic/mod.rs`, each with a reason.

This second one is a **predicted consequence, measured before this brief was written**: all three are
checked by hand-written `check_call` arms (`src/check.rs:4938,4948,4958`) and **none carries an
`env.register()` TypeScheme**, so `check_env.get` returns `None` and
`doc_arg_ret_types_match_checker_scheme` skips them. Same shape as `Option/expect` last stone.

### 4 — verify the payoff

`wat-scripts/scratch-pad/255-probe-the-accessor-classifies-pure.wat` exists and prints `false`/`false`
today. Its **first row must flip to `true`** — that is this stone's whole point, and the reason the
previous stone did not meet its acceptance criterion. You cannot rebuild, so report the pre-existing
binary's output and say plainly that the flip is the orchestrator's to verify.

## Blast radius

`src/intrinsic/option.rs` · a home for `Ok`/`Err` · `src/intrinsic/mod.rs` (module wiring + 3 ledger
rows) · `src/runtime.rs` (three `eval_list` arms out; the three fns become `pub(crate)`) ·
`src/rete/purity.rs` (three rows out). No body moves. No changes to `sort`, `None`, or
`src/collection/transform.rs`.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — if the predicted red does NOT appear.** If you can determine that any of the three DOES
carry an `env.register()` TypeScheme, STOP and report which — the design's measurement would be
wrong, and adding an unnecessary ledger row is exactly the laundering the ledger's own doc forbids.

**STOP-2 — `Err` is not `Partial`.** If you find yourself reasoning toward `Partial` for `Err`
because of its name, STOP and re-read the ruling. If you find an actual raise in `eval_err_ctor` past
the arity check, that is a real finding — report it with the line.

**STOP-3 — no body moves.** If any of the three cannot be a thin delegate over its existing named fn,
STOP and report what forced it.

**STOP-4 — `None` is not in scope.** `meter-2` excluded it by name with a cited reason (its
`eval_list` occurrence is a pattern-clause head inside `match`'s own implementation). If your change
appears to require touching `None`, STOP and report.

## Report

Per-file diff summary; the three rulings with the line you read for each; whether the predicted
checker-debt red is real (and the rows you added); the probe's output from the pre-existing binary.
Then the part the orchestrator cannot reconstruct: what surprised you — a constructor that did not
match its siblings, a dispatch path the design did not predict, or a place where the homing read
wrong.
