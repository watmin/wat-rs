# STONE P6-c-W2 — five readers across three namespaces

> Wave 2. Read `BRIEF-STONE-P6-c-W1-config.md` first — same shape, and its honest deltas are
> requirements here.

## The five

```
:wat::stream::empty     eval_seq_empty(args, list_span)                    arity 0   runtime.rs:11997
:wat::stream::cons      eval_cons(args, list_span, env, sym)               arity 2   runtime.rs:12018
:wat::stream::next      eval_stream_next(args, list_span, env, sym)        arity 1   runtime.rs:12140
:wat::program::env      eval_program_env(args, list_span)                  arity 0   runtime.rs:19890
:wat::stdlib::sources   crate::io::eval_stdlib_sources(args, …)            arity 0   src/io.rs:1837
```

All five are SHAPE=fits, checker-known (**no debt growth**), and each carries a **hand-rolled arity
guard** — the wave retires five more, exactly as W1 retired four `check_nullary` calls. Two subset
tails (`[list_span]`), three order tails (`[list_span, env, sym]`); all legal only because P6-c-1
landed.

⚠ **`:wat::stdlib::sources` lives in `src/io.rs`, not `runtime.rs`.** A brief that sizes its blast
radius around a *file* misses the *role* — O-iv-c-0's lesson, and this wave is deliberately built to
carry one out-of-file handler so the wave process meets that case early rather than at scale.

## ★ THE PREDICTION — and it is different from W1's

**`KNOWN_UNREVIEWED` must SHRINK by 4.** `stream::{empty,cons,next}` and `stdlib::sources` are on
that ledger; `program::env` is not. Homing gives a verb `intrinsic_meta` purity, so it leaves the
unreviewed set and the ledger must lose its line — the ratchet that went red in P6-c-1.
**`FROZEN_CHECKER_DEBT_LEDGER` stays at 50** (all five are checker-known).

A different number either way is a FINDING. Verify by asking the gate its own question
(`cargo nextest -E 'test(every_dispatched_verb_is_classified_or_disposed)' --no-capture` prints
`UNREVIEWED … ledger …`) — **not by grepping the array**, which gave three different answers across
three attempts in W1.

## ⛔ Two STOPs that are about these verbs specifically

**STOP-A — `eval_stdlib_sources` returns `Result<Value, RuntimeError>`, not `Result<Value,
EvalBreak>`.** The arm converts with `.map_err(Into::into)`. If homing it cleanly requires changing
its return type or its behaviour, **drop it from the wave and report** — four homed verbs and one
honest refusal beats five with a coerced error path. Its `_env`/`_sym` are also unused and are now
droppable under P6-c-1's subset rule; say whether you dropped them and what that retired.

**STOP-B — `:wat::stream::next` FORCES A THUNK.** Its purity is a real judgement, not a copy of its
neighbours': forcing a lazy cell can run arbitrary user code. `cons` and `empty` are constructors.
**If you cannot ground a purity declaration in what the body does, STOP and leave that verb
unhomed** — a wrong `@Purity` is not a doc nit; it feeds `intrinsic_meta`, the purity gate, and the
rete vocabulary's totality reasoning.

★ **And W1's hard-won coupling, stated so you do not pay for it again: the KIND of example you owe
is decided by the purity you declare.** A verb declared `Pure` AND `Deterministic` MUST carry at
least one *runnable* `@example`; `@example-norun` is refused by `purity_mandated_examples`. W1 lost
a full cycle to this. Decide purity first, then write the matching example.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK of all five: shape · checker-known · KNOWN_UNREVIEWED membership · every
      dispatch site. Disagreements with my table reported BEFORE any edit.
 1. ★ FIVE RULINGS with disk citations, added to DESTINATION_LEDGER. Instrument: HOMEABLE 0 → 5,
      AWAITING 107 → 102. Paste the summary. (It returns to 0 after homing — that is W1's finding,
      not a regression: the freshness FATAL fires when a ruled FQDN leaves the match.)
 2. ★ REAL ARITY PUBLISHED. `metadata-of` for all five: 0 · 2 · 1 · 0 · 0. Before and after.
 3. ★ THE ARITY ERROR SURVIVES for each — same op/expected/got, now raised by the shim. Quote one
      wrong-arity call per verb, before and after.
 4. ★ DIRECT CALLS BYTE-IDENTICAL, before and after. `git show HEAD:<path>` — never `git stash`.
 5. ★ FIVE HAND-ROLLED ARITY GUARDS DELETED. Name any shared helper that becomes dead and say
      whether you deleted it (W1's `check_nullary` had zero other callers and had to go).
 6. ★ `KNOWN_UNREVIEWED` SHRINKS BY EXACTLY 4, measured by the GATE'S OWN OUTPUT, not a grep.
      `FROZEN_CHECKER_DEBT_LEDGER` unchanged at 50.
 7. ★ PURITY GROUNDED. For each of the five, one sentence on what the body does that justifies its
      `@Purity`/`@Determinism` — and `stream::next`'s must address thunk-forcing explicitly.
 8. ★ Population 142 → 137, registry 386 → 391.
 9. cargo build --release --all-targets — clean; report any warning VERBATIM.
10. cargo nextest run --release -E 'test(stream) + test(intrinsic) + test(purity) + test(program) + test(stdlib)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Your pre-check table. The five rulings with citations. The instrument summary. `metadata-of` and
arity-error quotes before and after. The gate's own `UNREVIEWED … ledger …` line before and after.
The five purity justifications, `stream::next`'s in full. Then the honest deltas — especially
anything about `stdlib::sources`'s error type, and any purity you found harder to ground than
expected.
