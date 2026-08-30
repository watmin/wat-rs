# STONE — ONE PARAM-SPEC 5: THE LAST DOORS. Close every one, in one strike.

> **Builder: *"the heresy must be purged — i grow tired of this fight — it's become a zombie — the
> dead must die."***
>
> Four stones cleared 1675 corpus sites, stopped the lint writing it, stopped the diagnostics
> recommending it, annihilated 185 prose sites, and walled the value-construction path for three
> heads. **Three doors were left open. This closes all of them. There is no stone after this one.**

## The three doors, located and verified open

```
DOOR 1  TYPE-ANNOTATION POSITION — src/types.rs:5101 `parse_type_form`
        Accepts `(Head [A B])` (unmarked, "args tail is EXACTLY ONE WatAST::Vector") AND the bare
        args tail, for EVERY head. A wholly separate path from the value-construction checkers
        stone 3 closed. VERIFIED OPEN:
            (:wat::core::defn :my::f [x <- (:wat::core::Vector :wat::core::i64)] -> …)
            --check clean, runs.

DOOR 2  VALUE POSITION, THE OTHER THREE HEADS — src/check.rs:12258
        `split_type_param_bracket`'s `[WatAST::Vector(inner, _), rest @ ..]` arm.
        Stone 3 closed `unwrap_type_param_bracket` (Vector/HashMap/HashSet) and left this one.
        ★ Its own doc at :12220/:12229 says "③ deletes this arm" — IT NEVER HAPPENED. VERIFIED OPEN:
            (:wat::core::PersistentVector [:wat::core::i64] 1)              ACCEPTED
            (:wat::core::Tuple [:wat::core::i64 :wat::core::String] 1 "a")  ACCEPTED

DOOR 3  BUNDLE IS COUPLED TO THE CORPSE — src/holon/ast.rs:1131, src/lower.rs:243
        `is_holon_arg_canonical` requires `matches!(items[1], Keyword(_,_))` with elements at
        `items[2..]`. Under `:- [T]`, items[2] is the bracket Vector → `_ => false`.
        ⛔ BOTH SPELLINGS NOW FAIL: the bare one is walled out of source; the canonical one gives
        `no-step-rule for op: :wat::core::Vector`. Bundle's single-step path is dead for any program
        a user can write. See `NOTE-bundle-is-coupled-to-the-retired-spelling.md`.
```

## ★ THE FIX FOR ALL THREE IS THE SAME SHAPE

**Require the `:-` marker; peel it with `peel_param_spec` (`src/types.rs:4793`); delete the arm that
accepts anything else.** Doors 1 and 2 are deletions. Door 3 is the mirror image — a consumer that
must LEARN the marker instead of assuming its absence.

⚠ **Door 3 is a FIX, not a wall.** Do not "close" it by re-admitting the bare form. `Bundle` must
recognise `(:wat::core::Vector :- [T] elems…)` — peel, then require the remaining elements canonical.
`lower_bundle` has the identical dependency and its `:397` test must move to the canonical spelling.

★ **Row 4 is the proof Bundle is ALIVE again, and it has never been true**: a canonical Bundle must
step. That test has only ever passed on a shape the checker now forbids.

## THE METHOD — the same one that took 2765 to 0

**Put all three up. Read the failures. Waterfall.** Do not census first; my greps have been wrong
FOUR times on this campaign (short 178 · over 66 · short 2 · short 5, each a different mechanism).
The floor cannot make those mistakes.

`docs/SUBSTRATE-AS-TEACHER.md` + FM 15: **a large fail count is the progress meter.** Stone 3 opened
at 2765 and closed at 0 in three rounds. Do not stash, do not revert, do not ask whether to step back.

⚠ **A `.wat` corpus red is a codemod finding** — stone 1's `one-param-spec.wat` or stone 3's
`mandatory-typed-quasiquote-residual.wat` missed a shape. Extend a codemod, dry-run, diff, apply.
**Never hand-edit a `.wat` file.**
★ **Expect type-position reds the earlier codemods never touched** — they rewrote VALUE constructors;
door 1 is the annotation path, and the corpus may carry bare/unmarked spellings in `<-` and `->`
slots that nothing has ever rejected.

## STOP triggers — each REJECTS.

1. **You cannot close a door without re-admitting a spelling.** Report the mechanism; do not ship it.
2. **A red you cannot trace to a door you opened.** Report before changing anything else.
3. **You reach for stash/revert because the count is large.** FM 15.
4. **You hand-edit a `.wat` file.**

## Acceptance

```
 0. ★ ALL THREE DOORS CLOSED. Show each diff.
 1. ★ TYPE POSITION REJECTS both heretical spellings, for Vector AND one other head:
        [x <- (:wat::core::Vector :wat::core::i64)]     -> rejected
        [x <- (:wat::core::Vector [:wat::core::i64])]   -> rejected
        [x <- (:wat::core::Vector :- [:wat::core::i64])] -> ACCEPTED, and the fn runs.
 2. ★ VALUE POSITION REJECTS the unmarked bracket for Tuple, PersistentMap AND PersistentVector.
      Canonical still works for each — show a value for all three.
 3. ★ THE STALE DOC IS TRUE NOW: check.rs:12220/12229 said "③ deletes this arm". Delete the arm and
      make the comment true, or rewrite it. Say which.
 4. ★ BUNDLE STEPS, CANONICALLY — the row that has never been true:
        (:wat::eval-step! '(:wat::holon::Bundle (:wat::core::Vector :- [:wat::holon::HolonAST]
                             (:wat::holon::Atom "a") (:wat::holon::Atom "b"))))
      must produce a Bundle, not `no-step-rule`. And `step_holon_constructor_bundle` must be moved
      to the canonical spelling and pass — it is currently green on an unreachable input.
 5. ★ THE FIRST POST-WALL FLOOR, as a number, before any fixing.
 6. ★ THE WATERFALL — each round's count and what class it fixed.
 7. ★ EVERY `.wat` red fixed by a CODEMOD, named, dry-run and diffed. Zero hand-edits.
 8. ★ THE FINAL SWEEP: no spelling but `:- [...]` is accepted anywhere — value position, type
      position, every head. State the probes you ran and any place you could NOT close.
 9. cargo build --release --all-targets — clean; warnings VERBATIM.
10. cargo nextest run --release — 0 failed.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** Clippy is the orchestrator's; the floor is yours (row 10).
- No `git stash`. Do not commit, push, revert, or create a worktree.
- ⚠ `wat/*.wat` is FROZEN INTO THE BINARY — rebuild before believing any result there.

## Report back with

The three door diffs. The type-position and value-position probes, each spelling, each head you
tested. The Bundle step proof. The first post-wall count and the waterfall. Every codemod extended.
**And row 8 in full — anywhere a heretical spelling is still accepted, named.** Then the honest
deltas.
