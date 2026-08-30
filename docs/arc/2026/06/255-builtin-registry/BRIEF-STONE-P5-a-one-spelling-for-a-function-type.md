# STONE P5-a — a function type has ONE spelling in an `@arg`

> The prerequisite P5 did not know it had. Read `WORKLIST-open-stones.md`'s P5 row for where this
> sits: P5-b makes `@yields` mandatory for every fn-shaped `@arg`, and that rule cannot be written
> until "fn-shaped" has a single meaning.

## What is on disk

Five `@arg` declarations in the whole registry name a function type. **They use three spellings,
and one of them is not a wat type at all.**

```
src/intrinsic/kernel/resource.rs:326  prog           [(:wat::kernel::Peer :- [S R]) :-> :wat::core::nil]   ✅ canonical
src/intrinsic/kernel/resource.rs:327  init_fn        :wat::core::Fn()->wat::core::Record
src/intrinsic/kernel/resource.rs:328  post_spawn_fn  :wat::core::Fn(wat::spawn::ThreadLaunch)->wat::core::nil
src/intrinsic/kernel/resource.rs:379  post_spawn_fn  :wat::core::Fn(wat::spawn::ProcessLaunch)->wat::core::nil
src/intrinsic/kernel/source.rs:158    f              :wat::core::Fn                                        ⛔ see STOP-1
```

★ **Line 326 is the control, and it sits in the SAME doc block as 327 and 328.** The canonical form
is not hypothetical or proposed — it is already in use, three lines above two strings that are not.

⚠ **Why these four rotted and nothing noticed.** `:wat::kernel::spawn-thread`, `spawn-process` and
`fn-forms` are all three on **P4's `FROZEN_CHECKER_DEBT_LEDGER`** (verified by name; the ledger is
49). The doc/checker cross-check that would compare a declared `@arg` type against the real scheme
skips every entry absent from the checker — so these strings have **never been compared to
anything.** They are not a typo class; they are what a doc looks like when no instrument reads it.

## ★ THE CONTRACT DECISION — the spelling is DERIVED, never chosen

**The canonical rendering of a function type is whatever `typeexpr_to_doc_string` EMITS** for the
corresponding `TypeExpr::Fn` — `src/intrinsic/mod.rs:827`:

```rust
crate::types::TypeExpr::Fn { args, ret } => {
    if args.is_empty() { format!("[:-> {}]", …) }          // nullary
    else { format!("[{} :-> {}]", args_str.join(" "), …) } // otherwise
}
```

I am not picking a house style. That function is the same one the doc/checker gate uses to decide
whether a declared type matches the scheme, so a string written in its output form is a string that
will COMPARE EQUAL the day these entries leave the debt ledger. Any other spelling is a future
mismatch already on disk.

**Derive each correction from that renderer yourself and report any disagreement with my table:**

| site | is | should render as |
|---|---|---|
| `resource.rs:327` | `:wat::core::Fn()->wat::core::Record` | `[:-> :wat::core::Record]` |
| `resource.rs:328` | `:wat::core::Fn(wat::spawn::ThreadLaunch)->wat::core::nil` | `[:wat::spawn::ThreadLaunch :-> :wat::core::nil]` |
| `resource.rs:379` | `:wat::core::Fn(wat::spawn::ProcessLaunch)->wat::core::nil` | `[:wat::spawn::ProcessLaunch :-> :wat::core::nil]` |

★ **The leading colon is a real defect, not cosmetic.** `wat::spawn::ThreadLaunch` is not a keyword.
The oracle for the correct spelling is a **working, type-checked `@example`** at
`src/intrinsic/kernel/identity.rs:330`, which writes `:wat::spawn::ThreadLaunch` — with the colon —
inside a program the example runner executes. **Confirm it is a runnable `@example` and not
`@example-norun` before you cite it**; if it is norun, say so and find another oracle.

## ⛔ STOP-1 — `source.rs:158` is NOT a correction. Report it; do not guess a type.

`@arg f :wat::core::Fn` names **`ANON_FN_SYMBOL`** (`src/value/frame.rs:25`) — the string an
anonymous fn *value* renders as. It is a value rendering standing in a type position, so it has no
`TypeExpr::Fn` to derive from. And the arg's own prose says it accepts *"the fn value to reify (or a
keyword naming a registered fn)"* — two shapes, and wat has no union type.

**Do not invent a spelling for it and do not reach for a type-system feature.** Handle it the way
P4 handled its debt: name it on a frozen ledger (below) so the wall passes while the gap stays
visible and countable. Report what you found.

## The work — part 2, the WALL, and build it FIRST

A doc-only correction with no gate rots again the moment someone writes the next `spawn-*`. Ship a
lint alongside it, in the style of `tests/lint/`:

> **Every `@arg` whose declared type names a function is written in the canonical bracket form.**

Walk `registry().all_entries()`; for each `(name, ty, …)` in `entry.args`, a type is a fn-type claim
if it contains `->` **or** equals `ANON_FN_SYMBOL`. Require: the arrow is spelled `:->`, the whole
type is bracket-delimited, and `ANON_FN_SYMBOL` never appears as a type. Carry a **FROZEN NAMED
LEDGER** for STOP-1's site — names, never a count, and the failure text must name the offending
FQDN and its string.

★ **BUILD THE WALL BEFORE THE CORRECTIONS AND WATCH IT GO RED ON ALL FOUR.** That is the stone's
proof. A wall written after the fix has never been shown to catch anything.
`NISI FRANGAS, NIHIL PROBAS.`

## STOP triggers — each REJECTS. Ship nothing on that row and report.

1. `source.rs:158` — above. Ledger it, never guess it.
2. **A correction changes behaviour.** This is a DOC-ONLY stone: no signature, no body, no scheme.
   If any test result moves, something is wrong — stop and report it.
3. **The `identity.rs:330` oracle turns out to be `@example-norun`.** Then it proves nothing about
   type-checking; say so and name what you used instead.
4. **The wall's predicate needs an exception to pass.** An exception that is not STOP-1's single
   ledgered site means the predicate is wrong — report it rather than widening it.

## Acceptance

```
 0. ★ YOUR OWN CENSUS of fn-typed `@arg`s across the whole registry, derived from `entry.args`,
      not from my table. Every disagreement with my five reported — count AND sites.
 1. ★ THE WALL GOES RED FIRST, on all four, BEFORE any correction. Paste its failure verbatim.
 2. ★ THE WALL GOES GREEN after the three corrections + STOP-1's ledger entry.
 3. ★ EACH CORRECTED STRING IS THE RENDERER'S OUTPUT. Show how you derived it from
      `typeexpr_to_doc_string`, not from my table.
 4. ★ THE ORACLE IS CITED AND CHECKED — `identity.rs:330` runnable or not, and what follows.
 5. ★ ZERO BEHAVIOUR CHANGE. `(:wat::core::show-source …)` / render-doc output for spawn-thread,
      spawn-process and fn-forms before and after — the ONLY diff is the three type strings.
      `git show HEAD:<path>` for the pre-image — never `git stash`.
 6. ★ THE GOLDEN. `tests/reflection/probe_arc255_spec_complete.rs:107` pins render-doc output
      byte-identically. Say whether it moved. If it did, that is expected and the new bytes go in
      the report — do not edit a golden silently.
 7. cargo build --release --all-targets — clean; report any warning VERBATIM.
 8. cargo nextest run --release -E 'test(reflection) + test(lint) + test(kernel) + test(spawn)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never operate on a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands, not when a command starts.
- `cargo build`, scoped `cargo nextest`, `./target/release/wat` — yes. **The full floor and clippy
  are the orchestrator's**; do not run them.
- **You may not spawn sub-agents.**
- No `git stash`, in any form. Do not commit, push, revert, or create a worktree.
- New scratch `.wat` → `wat-scripts/scratch-pad/`, `--check` clean. Not the session scratchpad.
- ⚠ Your own added prose must not contain the literal pattern your wall greps for — three riders
  tripped their own acceptance check on their own comments in one day.

## Report back with

Row by row: the command, its actual output, PASS/FAIL. Your own census with every disagreement
against my five. The wall's RED output before the fix, verbatim. What `source.rs:158` really is.
Then the honest deltas — what surprised you, and anything you could not measure.
