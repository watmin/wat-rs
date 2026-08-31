# BRIEF — STONE: the Option/Result siblings get homes

Home and rule `:wat::core::Option/try`, `:wat::core::Result/expect`, `:wat::core::Result/try` — the
three that remain of the family homed earlier today. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-the-option-result-siblings.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or
`scripts/floor.sh`. You may run the pre-existing `target/release/wat` for a fast read, remembering it
does not contain your Rust changes. **You may not spawn sub-agents.** Work only in
`/home/john/work/holon/wat-rs`; verify with `pwd` first. Do not commit, push, stash, revert, or
`git checkout --` anything.

## Read in order

1. The DESIGN above — the ruling's split, and why the two `try` verbs get the OPPOSITE verdict from
   their `expect` sibling.
2. `docs/arc/2026/06/255-builtin-registry/RULING-a-raise-is-not-an-outcome-so-a-raising-verb-is-partial.md`
   — the rule you are ruling by.
3. `src/intrinsic/option.rs` — **the template**, written earlier today: `Option/expect` and `Some` as
   thin delegates with full directive blocks and grounding prose per axis. `Option/try` belongs here.
4. `src/intrinsic/result.rs` — same, for `Ok`/`Err`. `Result/expect` and `Result/try` belong here.
5. The three implementations, all named fns in `src/runtime.rs`: `eval_option_try`,
   `eval_result_expect`, `eval_try`, reached from arms at `:5829`, `:5820`, `:5819`.
6. `src/runtime.rs:19455-19462` — the comment that decides the `try` verdict. Read it before writing
   `@Totality` for either `try`.

## The work

### 1 — home the three

One thin `#[wat_intrinsic]` delegate each, calling straight into the existing named fn. **Bodies do
not move.** Declare the real arity so the hand-rolled `args.len() != N` guards retire:
`Result/expect` is **2**, both `try` verbs are **1**. Remove their three literal dispatch arms.

⚠ `src/intrinsic/collection.rs`'s header carries the 1-arity delegate idiom
(`std::slice::from_ref(v)`, not `&[v.clone()]`). Both `try` verbs are 1-arity — read it, or clippy
will catch you the way it caught the last two stones.

### 2 — the rulings, from the bodies

| verb | @Purity | @Determinism | @Totality |
|---|---|---|---|
| `Result/expect` | Pure | Deterministic | **Partial** — `expect_panic` on `Err` |
| `Option/try` | Pure | Deterministic | **Total** |
| `Result/try` | Pure | Deterministic | **Total** |

The `try` verdict rests on a measured fact, and your grounding prose must cite it: a propagate
**signal** is not a raise — `runtime.rs:19458` says `TryPropagate` is *"wrap[ped] in the function's
own `Err(e)` return"*, and the type checker **guarantees** the enclosing return type is
`(Result :- [_ E])` whenever the body contains a `try`. So the outcome is always a matchable enum
arm, which the RULING calls total.

### 3 — satisfy the ratchet

Delete the three `KNOWN_UNREVIEWED` rows (48 → 45).

### 4 — the predicted debt rows

All three are checked by hand-written `check_call` arms and **none carries an `env.register()`
TypeScheme** — measured. Expect `checker_skip_debt_is_named_and_frozen` to demand a
`FROZEN_CHECKER_DEBT_LEDGER` row for each (59 → 62), with a reason, following the `Option/expect`
row added earlier today.

### 5 — the probe

`wat-scripts/scratch-pad/255-probe-the-option-result-siblings.wat`, following the shape of the others
there: the three verbs still behave as before, and `metadata-of` shows the split — `expect` `Partial`
against both `try` verbs `Total`.

## Blast radius

`src/intrinsic/option.rs` · `src/intrinsic/result.rs` · `src/runtime.rs` (three arms out; the three
fns become `pub(crate)`) · `src/rete/purity.rs` (three rows out) · `src/intrinsic/mod.rs` (three
ledger rows) · the new probe. No body moves. No `.wat` corpus change.

## STOP triggers — each rejects; ship nothing further on that point and report

**STOP-1 — do not rule the `try` pair by family symmetry.** They share a namespace and a naming
convention with `expect` and do the OPPOSITE thing. If your reading of `eval_try`/`eval_option_try`
finds an actual raise past the arity check, that is a real finding — report it with the line, and do
not ship either verdict until it is resolved.

**STOP-2 — `Unreviewed` is not the safe answer.** You are being sent to rule these. `Unreviewed`
means *nobody looked*; recording it about a body you just read is the lie the fourth variant exists
to prevent.

**STOP-3 — the debt prediction is falsifiable.** If any of the three DOES carry an `env.register()`
TypeScheme, STOP and report which — the measurement is wrong, and an unearned ledger row is the
laundering the ledger's own doc forbids.

**STOP-4 — no body moves.** If any of the three cannot be a thin delegate over its existing named fn,
STOP and report what forced it.

## Report

Per-file diff summary; the three rulings **with the line you read for each**, especially the two
`try` verdicts; whether the debt prediction held; and the probe's output from the pre-existing
binary. Then the part the orchestrator cannot reconstruct: what surprised you — a body that did not
match its sibling, a signal path the design did not name, or an arity that was not what the design
said.
