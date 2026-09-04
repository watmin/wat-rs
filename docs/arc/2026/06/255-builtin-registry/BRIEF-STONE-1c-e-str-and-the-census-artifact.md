# BRIEF — STONE 1c-e: register `str`, then re-derive the census honestly

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-e-str-and-the-census-artifact.md`

## Two deliverables

**① Register `:wat::core::str`** — the last ordinary verb in the namespace, 135 corpus sites.
**② Re-derive the corpus census** and report it with non-verb names separated out.

The second is the more valuable. The number this campaign has been quoting is arithmetic on a
2026-09-03 sweep, never re-run, and never audited for names that are not verbs at all.

## ① `str`

```
runtime.rs:2925    ":wat::core::str" => eval_str(args, list_span, env, sym)
runtime.rs:9705    fn eval_str(args: &[WatAST], list_span: &Span, env: &Environment, sym: &SymbolTable)
```

`eval_str` is single-use and already carries `#[wat_intrinsic]`'s canonical signature — **annotate
it in place**. No wrapper, no extraction, no delegate. Then delete the arm (the no-literal-arm
gate will demand it) and apply what the ledger ratchets name.

**Read `src/collection/transform.rs`** for the shape — six rows landed there at 1c-a-i exactly
this way.

⚠ **`str` has NO checker knowledge at all.** Measured exactly, with the closing quote:

```
check.rs mentions of ":wat::core::str"   0
register_builtins scheme                 0
```

So there is nothing to mirror `@arg`/`@ret` against. Ground them in `eval_str`'s own body, and
**report what a `(str x)` call actually does at check time today** — that absence is a finding in
its own right, and it decides whether this row is honest.

⛔ **`str` IS A PREFIX OF `struct`.** `grep -F ':wat::core::str'` matches
`:wat::core::struct`, `struct-new` and `struct->form` and returns 9 where the answer is 0. This
orchestrator made exactly that error while crawling this stone and caught it only on a re-run.
**Terminate every pattern** — `'":wat::core::str"'` with the closing quote.

## ② The census — re-derive it, and separate the non-verbs

The procedure is recorded at the top of
`docs/arc/2026/06/255-builtin-registry/WORKLIST-the-121-the-registry-cannot-vouch-for.md`:
patch `is_resolvable_call_head`, `cargo build --release --bin wat`, sweep every `.wat` under
`wat/` and `wat-scripts/`, **then REVERT the patch and verify `git diff` is empty.**

Run it after `str` lands. Then, for **every** name it reports, answer one question:

> **Is this a verb the registry should hold, or a name another authority already answers?**

Two kinds of non-verb are already known and must be counted separately rather than folded in:

- **`:wat::type::{Tuple,i64,String,Vector}`** — a type path in arc 251's dual-read spelling. The
  source says `wat.type/Tuple`; `types.rs:5172` strips `wat::type::` → `wat::core::`. Zero corpus
  text spells the `::` form.
- **`:wat::core::None`** — a declared unit variant of `Option` (`types.rs:1248`,
  `EnumVariant::Unit("None")`). Every corpus site is a match **pattern** — `(:wat::core::None body)`
  — or a value. It is never a call head; the census sees it because a match arm's pattern occupies
  the head position syntactically.

Both are answered by the frozen `TypeEnv`, which the RULING exempts by name: *"`constructor_meta`/
`accessor_meta` DERIVE from the frozen TypeEnv … Derivation from one source is not duplication."*

**Look for a third kind.** Check every remaining name the same way — a name that another authority
already answers is a census artifact, not population, and finding one is worth more than the
registration.

Update the WORKLIST with a dated re-derivation section in the style of the two already there,
reporting three numbers: **total names, verb population, non-verb artifacts** — with the artifacts
listed and each one's real authority named.

## Blast radius

`src/runtime.rs` (one annotation + one doc block + one arm retirement) · `src/intrinsic/mod.rs`
and `src/rete/purity.rs` (ledgers, per the ratchets) · the WORKLIST (a new dated section).
**The census patch is temporary and MUST be reverted — verify with `git diff` before you finish.**

## STOP triggers — halt and report, do not improvise

- **STOP-1.** `eval_str` will not take `#[wat_intrinsic]`. Report the exact error; do not reshape it.
- **STOP-2.** You cannot ground `@arg`/`@ret` from `eval_str`'s body. Say what you read and what
  you could not determine — with no checker arm to mirror, "I cannot tell" is a real outcome here.
- **STOP-3.** The census patch does not revert cleanly, or `git diff` is non-empty at the end.
  Report immediately — a left-in measurement patch is a silent behaviour change.
- **STOP-4.** DEBT grows by anything other than exactly 1, or `KNOWN_UNREVIEWED`/`GAP_A` move.
- **STOP-5.** A test outside the ledger ratchets goes red. Copy its entire stdout and stderr block
  verbatim from `.floor/latest/raw.log`, name the exact assertion that fired, and report — before
  re-running anything.

## Verification, in this order

```bash
cargo build --release 2>&1 | tail -20
./scripts/floor.sh > /dev/null 2>&1; echo "EXIT=$?"
grep -E "^\s+Summary" .floor/latest/raw.log | tail -2
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
git diff --stat src/resolve/walk.rs     # MUST be empty — the census patch reverted
```

## Acceptance — derived, not estimated

```
registry rows      549 → 550     +1 attribute site, counted ANCHORED
GAP_A               49 → 49      `str` is not on it
GAP_B               45 → 44      `str` IS on it
DEBT               118 → 119     +1 — no CheckEnv scheme exists for it
KNOWN_UNREVIEWED    13 → 13      `str` is not on it — checked against the constant
literal arms deleted  —  → 1
floor        5129/5129 → 5129/5129
clippy                    0
the corpus              RE-DERIVED — report total / verb population / non-verb artifacts.
                        ⛔ No number is predicted here. Every prior figure was arithmetic;
                        replacing arithmetic with a sweep is this stone's point.
```

## Working rules

Everything foreground. You may not spawn sub-agents. Do not background the floor run. No
worktrees, no `git stash`, no `git revert`, no commit, no push — leave the tree dirty and report;
the orchestrator commits. Report the census in full: the raw name list, your verb/non-verb
judgement for each, and the authority you found for every non-verb.
