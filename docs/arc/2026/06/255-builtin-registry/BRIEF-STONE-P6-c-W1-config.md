# STONE P6-c-W1 — the first wave: `:wat::config`'s four readers

> The first stone to move the homeable counter off ZERO. Read `BRIEF-STONE-P6-c-3-default-deny.md`
> first: **a ruling is a `(destination, reason)` pair and the reason must trace to disk.** Ruling
> these four IS this wave's work, not a formality on the way to the code.

## The four, and why they are wave 1

```
:wat::config::dim-count      eval_config_dim_count                 runtime.rs:6290 -> :20173
:wat::config::dim-capacity   eval_config_dim_capacity              runtime.rs:6291 -> :20189
:wat::config::global-seed    eval_config_global_seed               runtime.rs:6292 -> :20224
:wat::config::noise-floor    eval_config_noise_floor_default_shim  runtime.rs:6293 -> :20210
```

Verified before drawing: all four are **SHAPE=fits**, **already in the checker** (`check.rs:18606`
and siblings — so homing adds NO checker debt), **absent from `KNOWN_UNREVIEWED`** (so nothing to
shrink), and **single-dispatch-site** — the other grep hits are the handler's own arity check, a
`resolve/mod.rs` test assertion, and the TypeScheme. Confirm all four yourself; a disagreement is a
finding.

⛔ **`:wat::config::set-redef!` and `set-eval-redef!` are NOT in this wave.** They are NEEDS-SHAPE,
genuinely multi-site (`config.rs:503/521`, `runtime.rs:2655/2663` freeze-time, `:5481` a deliberate
eval-time no-op), and want a destination this ledger does not yet have. Leave them. Naming them here
is the affirmative cut, not a deferral.

## ★ THE CONTRACT DECISION — home them with their REAL arity

All four share one shape:

```rust
fn eval_config_dim_count(args: &[WatAST], sym: &SymbolTable, list_span: &Span) -> … {
    check_nullary(":wat::config::dim-count", args, list_span)?;    // hand-rolled
```

**They are NULLARY verbs declaring a variadic `&[WatAST]` they use only to reject.** Homing them
as-declared would register `Arity::Variadic` for a 0-arg verb — publishing a fictional arity through
`metadata-of`, which is precisely the lie Stone H-1a spent 35 verbs correcting and P2 fixed for `if`.

So: **delete the `args` parameter, delete the `check_nullary` call, let the generated shim own the
arity.** The remaining tail is `(sym, list_span)` — a SUBSET in a non-canonical order, legal only
because Stone P6-c-1 landed. This wave is that stone's first real consumer.

⚠ This IS a signature change, and P6-c-1's "signatures untouched" was that stone's claim, not a
standing rule. Declaring a real arity is the H-1a treatment and it is the point.

## The work

1. **Rule each of the four** — a `(INTRINSIC, reason)` row in `DESTINATION_LEDGER`, reason traced to
   disk (the handler's body, its `check_nullary`, its single dispatch site, its TypeScheme). If a
   reason cannot be sourced, STOP: leave it `UNKNOWN-RULED-PENDING` and say so.
2. **Home each**: `#[wat_intrinsic(<fqdn>)]` + a full `///` block (`@added`, `@ret`, `@Purity`,
   `@Determinism`, `@Category`, ≥1 example), `args` and `check_nullary` deleted, arm deleted.
3. **Verify the ledgers did NOT move.** Predicted: `FROZEN_CHECKER_DEBT_LEDGER` unchanged (all four
   are checker-known) and `KNOWN_UNREVIEWED` unchanged (none listed). **A move is a FINDING** — it
   means one of my pre-checks was wrong, and P6-c-1's red proved this class is real.

## STOP triggers — each REJECTS.

1. **A reason you would have to invent.** Leave the verb unruled and unhomed, and say which.
2. **A verb turns out to be multi-site**, or its arity is not 0. Report; do not adapt the wave.
3. **Either frozen ledger moves.** Stop and report before continuing — that is a wrong pre-check,
   not a chore.
4. **`check_nullary` has callers outside these four that you would need to touch.** Out of scope.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK of all four: shape · checker-known · not in KNOWN_UNREVIEWED · single
      dispatch site. Every disagreement with my table reported BEFORE any edit.
 1. ★ THE FOUR RULINGS, each with its disk citation. Then the instrument: HOMEABLE 0 → 4,
      AWAITING 111 → 107. Paste the summary block.
 2. ★ REAL ARITY PUBLISHED. `(:wat::runtime::metadata-of :wat::config::dim-count)` before and
      after — it must report arity 0, not -1 and not variadic. All four.
 3. ★ THE ARITY ERROR SURVIVES THE MOVE. `(:wat::config::dim-count 1)` before and after: the SAME
      `ArityMismatch` shape (same op, expected, got), now raised by the shim rather than
      `check_nullary`. Quote both.
 4. ★ DIRECT CALLS BYTE-IDENTICAL for all four, before and after. `git show HEAD:<path>` for the
      pre-image — never `git stash`.
 5. ★ FOUR `check_nullary` CALL SITES DELETED, and `check_nullary` itself still compiles and is
      still used by its other callers (say how many remain).
 6. ★ REGISTRY +4, giant match −4 arms.
 7. ★ BOTH FROZEN LEDGERS UNCHANGED — show the before/after counts, not an assertion.
 8. cargo build --release --all-targets — clean; report any warning VERBATIM.
 9. cargo nextest run --release -E 'test(config) + test(intrinsic) + test(purity) + test(reflection)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never operate on a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Your pre-check table. The four rulings with citations. The instrument summary before and after. The
`metadata-of` and arity-error quotes, before and after. The two ledger counts. Then the honest
deltas — especially anything that made a ruling harder to write than expected, because that is the
signal the next wave needs.
