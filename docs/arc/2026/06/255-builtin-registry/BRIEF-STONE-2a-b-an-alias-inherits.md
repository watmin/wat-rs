# BRIEF — STONE 2a-b: an alias inherits its axes; declaring one becomes an error

An alias row must declare **no** axes. The registry answers with its target's, resolved after every
submission is folded. Declaring one is a **hard error**, not a silently-ignored field.

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-2a-b-an-alias-inherits-it-does-not-declare.md`
— read the measurement at the top. **The defect it fixes is on disk right now, and it is ours.**

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. Run every command you do run in
the FOREGROUND and block on it. The orchestrator builds, floors and clippies centrally — you do not
run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`. You may run the pre-existing
`./target/release/wat` and `--check` for a fast read. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything. Tree clean, floor green at 5126.

## The defect, so you can see it before you fix it

```
                :wat::i64::>  (target)     :wat::rete::i64::>  (its alias)
@Totality       Total                      Partial        ⛔
@Category       Probe                      Reflection     ⛔
```

Run it: `(:wat::core::render-doc :wat::rete::i64::>)` and the same for the target. One behaviour —
the registry re-dispatches one to the other — reported two ways.

## Read in order

1. **The DESIGN**, contract and acceptance.
2. **`crates/wat-doc/src/lib.rs:811-826`** — where `parse()` enforces the five required directives
   (`MissingPurity`, `MissingDeterminism`, `MissingTotality`, `MissingExpandTime`,
   `MissingCategory`), and **`:1540-1562`** — `parse_special_form()`'s identical block. ⚠ **Both**,
   and 2a found a **third** path (`from_metadata`) — check whether it enforces them too.
3. **`src/intrinsic/mod.rs`'s `registry()`** — the `OnceLock` fold. Its loops are at `:598`
   (`IntrinsicSubmission`), `:648` (impls) and `:662` (`SpecialFormSubmission`). ⚠ **The target may
   fold after the alias**, so resolution cannot happen inside a loop.
4. **`src/intrinsic/special/rete_i64_gt_alias.rs`** — the witness, and the two lines that lie.

## The work

### 1 — an alias row declares no axes, and declaring one is an ERROR

In every parse path: when `@alias` is present, the five axis directives are **not required** — and
if one is *given*, that is a `DocError`, not a shrug. Name the error for what it is: an alias's axes
come from its target, so stating one is a claim the registry will not honour.

### 2 — the registry resolves them at fold time

After **all** submissions are folded, walk the entries: for every row with `alias_of: Some(target)`,
replace its five axes with the target's.

⛔ **Not inside a loop** — the target may not be registered yet when the alias is. If you cannot do
it as a second pass over the finished map, STOP and report rather than reordering the loops on a
guess.

### 3 — the witness loses its five axis lines

And with them the `Totality`/`Category` contradiction. Its prose should say plainly that the axes are
its target's, and why an alias states none.

### 4 — a gate proving the resolution RAN

For every `alias_of` row, its five axes must equal its target's. **Non-vacuous**: assert it inspected
at least one row and name the rows inspected.

⚠ This is not the equality-gate the DESIGN disqualified. That one would police two independent
declarations; this one proves a **derivation happened** — the difference is that after your change
there is only one declaration, and the gate checks the copy reached the entry rather than checking
two authors agreed.

## Blast radius

`crates/wat-doc/src/lib.rs` (parse paths) · `crates/wat-macros/src/{wat_intrinsic,wat_special_form}.rs`
(if they demand the axes independently) · `src/intrinsic/mod.rs` (the resolution pass + the gate) ·
`src/intrinsic/special/rete_i64_gt_alias.rs`. **No other row changes** — every non-alias row keeps
declaring its own five.

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — declaring an axis on an alias must FAIL LOUDLY, not be ignored.** A row that says
`@Totality Partial` while the registry answers `Total` is worse than the defect being fixed: the
source lies and no reader can tell which won. If you cannot make it an error in a given parse path,
STOP and report that path.

**⛔ STOP-2 — resolve after ALL submissions are folded.** An alias whose target folds later must
still get the target's axes. If your resolution silently leaves defaults in place for such a row,
the gate in step 4 is the only thing that would catch it — build the gate so it would.

**⛔ STOP-3 — do not change any non-alias row's axes.** Exactly one row is an alias today. If the
change appears to require editing another row, that is a finding.

**⛔ STOP-4 — do not touch `src/rete/vocabulary.rs`, `dispatch_rete_op`, or 2a's target/chain gate.**
Unchanged by this stone.

**STOP-5 — verbatim otherwise.**

## Sabotage — report each as "predicted red, unverified"

1. add `@Totality Total` back to the alias row → **compile error**, or a `DocError` naming it?
   (STOP-1's proof — say which, and quote the message)
2. break the resolution pass (leave the alias's axes as parsed defaults) → what does the step-4 gate
   say?
3. point the alias at a target with *different* axes and confirm the alias's reported axes follow →
   ⚠ this is the differential: if the alias's axes do NOT change, the resolution is not running and
   the gate is passing on a coincidence.

## Report

Every parse path you changed, **with the count, and whether a third existed** · the error you raise
for a declared axis, verbatim · the resolution pass verbatim **and where it sits relative to the
fold loops** · the gate verbatim including its non-vacuity assertion · the witness's new prose ·
`render-doc` output for both names **after** the change, showing the five axes now agree · the three
sabotage predictions · and what surprised you.

## Prior comparable

`BRIEF-STONE-2a-the-alias-field.md` — the stone that added the field and minted the defect.
`@see` is still the structural precedent for a directive threaded across the same three layers.
