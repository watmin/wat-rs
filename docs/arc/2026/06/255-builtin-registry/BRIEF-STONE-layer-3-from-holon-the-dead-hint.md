# BRIEF — STONE layer-3: retire `from-holon`'s 3-arg hint form

Read `DESIGN-STONE-layer-3-from-holon-the-dead-hint.md` first — it carries the four witnesses that
this form is dead, and you should confirm each one yourself before deleting anything.

## The work, one paragraph

`:wat::holon::from-holon` accepts 1 or 3 arguments. The 3-arg form's parsed result is assigned to
`_hint_is_hashmap` and never read. **Make the verb arity-1 only** — delete the hint parsing, delete
the checker's 3-arg fork, and delete the doc-block paragraph and stale comment that describe the
behaviour it never had.

## Read in order

```
src/intrinsic/holon/atom.rs:110-133   the doc block. The paragraph beginning "(from-holon h ->
                                      (:wat::core::HashMap :- [K V])) disambiguates" describes a
                                      dead feature and goes.
src/intrinsic/holon/atom.rs:145-196   the arity fork and the ~40-line `_hint_is_hashmap` block.
src/intrinsic/holon/atom.rs:~261      the stale comment "The `-> (HashMap :- [K V])` consumer-hint
                                      form is preserved for empty-Map classifier."
src/intrinsic/holon/atom.rs:~290      "Empty Map always returns empty HashMap regardless of hint."
                                      ← THIS ONE IS CORRECT AND STAYS. It is the evidence.
src/check.rs:3702-3717                the checker's 3-arg fork, whose own comment calls the `->`
                                      and type keyword "syntactic decoration".
src/remedy/retirement.rs:83-110       RETIREMENT_TABLE and its doc. Decide whether a row belongs
                                      here — see the open question below.
```

## ★ CONFIRM BEFORE YOU CUT — row 0 is this

Do not take the design's word for any of it. Verify yourself, and report each:

1. `_hint_is_hashmap` has exactly one occurrence in the file (its assignment) — no reads.
2. Zero `.wat` callers use the 3-arg form.
3. The two comments contradict each other, and the "regardless of hint" one matches the code.
4. `check.rs`'s fork treats the extra args as decoration and returns the same type either way.

**If any of the four does not hold, STOP.** The cut rests on all four.

## The open question you must ANSWER, not assume

`RETIREMENT_TABLE`'s doc says *"HARD CUT stones append entries at ship time."* But its rows map a
retired **name** to a replacement **name** — `:wat::core::struct` → `:wat::core::defstruct`. This
stone retires an **arity variant** of a verb that survives; there is no retired name and no
replacement name.

**Decide, and justify in your report.** "No row, because the schema is name→name and nothing is
being renamed" is a perfectly good answer. So is a row, if you find the schema accommodates it.
What is not acceptable is silence, or a row bent to fit.

## Blast radius

`src/intrinsic/holon/atom.rs`, `src/check.rs`, and `src/remedy/retirement.rs` only if your answer
above says so. **No `.wat` corpus file** — there are no callers to migrate.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **Any of row 0's four confirmations fails.** STOP — the cut's evidence is wrong and that matters
   more than the cut.
2. **You find a caller of the 3-arg form anywhere** — corpus, test, doc example, Rust. STOP.
3. **You are about to add a bespoke "retired form" diagnostic.** The ordinary arity error is the
   pinned contract. STOP.
4. **You are about to touch the classifier dispatch** or any other verb. Arc 228's mechanism is
   correct and is why the hint is redundant. STOP.
5. **You are about to split or relocate `from-holon`'s decode body.** That is the next stone. STOP.

## Acceptance

```
 0. ★ THE FOUR CONFIRMATIONS above, each with the command you ran and its output.
 1. ★ THE FORM IS NOW REJECTED. Show `(from-holon h -> (:wat::core::HashMap :- [K V]))` failing,
      and quote the diagnostic verbatim. Say whether it is a CHECK-time or RUN-time refusal — and
      if the checker no longer forks, it should be check-time.
 2. ★ THE 1-ARG FORM IS UNCHANGED. Exercise it on at least three shapes (a leaf, a Map-classified
      Bundle, and an EMPTY Map-classified Bundle) and show identical results before and after.
      ★ The empty-Map case is the one the dead hint claimed to serve — prove it still decodes to an
      empty HashMap without it.
 3. ★ THE DOC NO LONGER PROMISES IT. No `///` line in atom.rs mentions the 3-arg form or
      disambiguation. The stale `:127`-style comment is gone; the correct "regardless of hint" one
      REMAINS.
 4. ★ YOUR RETIREMENT_TABLE ANSWER, justified.
 5. ★ LINE ACCOUNTING: atom.rs and check.rs, before and after.
 6. ★ REGISTRY 429 and the gate's UNREVIEWED 217, both unchanged.
 7. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 8. cargo nextest run --release -E 'test(holon) + test(check) + test(types)'
```

★ **Row 2 is the load-bearing one.** Deleting a dead feature is easy; proving the live path that
replaced it actually covers the case the dead one claimed is what makes the cut safe.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean.

## Report back with

Row 0's four confirmations with their commands. The rejection diagnostic, verbatim, and whether it
is check- or run-time. Row 2's three shapes, before and after, with the empty-Map case called out.
Confirmation the doc is clean and which comment survived. Your RETIREMENT_TABLE answer and its
reasoning. Line accounting. Then the honest deltas — especially anything that suggested the form was
LESS dead than the design claims.
