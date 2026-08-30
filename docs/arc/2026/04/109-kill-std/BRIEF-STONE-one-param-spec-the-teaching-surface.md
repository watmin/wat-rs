# STONE — ONE PARAM-SPEC, stone 2b: the TEACHING SURFACE

> Everything the substrate *shows* a user. Stone 2a killed the lint that **wrote** the heresy; this
> kills the places that **display** it. Read
> `NOTE-a-parametric-literal-has-three-spellings-and-no-authority-names-all-three.md` for the ruling.

## The 42

```
A.  8 ctor diagnostics  —  "first argument must be a type keyword (e.g., :i64)
                            or a `(Head :- [T …])` type form"
       src/check.rs:12364, 12505, 12603, 15005
       src/collection/eval.rs:1224, 1338, 1634, 1655

B. 34 @example / @example-norun lines carrying a bare param-spec, inside `#[wat_intrinsic]`
       doc blocks — PUBLISHED by `render-doc` and `show-source` since Stone P6-a.
       e.g. src/intrinsic/string.rs:479  `@example (:wat::string::split "a,b,c" ",")
                                          #=> (:wat::core::Vector :wat::core::String "a" "b" "c")`
```

**A is the sharper half.** Those eight messages fire when a user gets it wrong, and they answer by
**offering the retired spelling first** — `(e.g., :i64)` before the canonical form. The substrate's
own error text has been the most persuasive advocate the bare form had.

⛔ **NOT in this stone: the ~159 remaining `.rs` sites** — prose, test fixtures, embedded source
strings. They have **no oracle**: nothing would catch a wrong edit, which is a different kind of work
and gets its own stone. Affirmative cut.

⛔ **NOT in this stone: the WALL.** Nothing here rejects anything. The bare form still parses when
this lands; these messages simply stop recommending it, so that when the wall goes up they are
already true.

## ★ A DISTINCTION THAT MUST NOT BE FLATTENED

**`:wat::core::i64` as a type is CORRECT.** A slot that takes ONE type legitimately takes a bare type
keyword — `(:wat::core::subtype? :my::Child :my::Parent)` is right, and `check.rs:2751/2760/2808/2847`
say "a type keyword or `(Head :- [args])` type form" **correctly**, because a non-parametric type IS
just a keyword.

**The heresy is a bare keyword in a PARAM-SPEC slot** — `(Vector :wat::core::i64 1 2 3)`. Only the
eight listed in A are param-spec messages. **If you find yourself editing a fifth message in
`check.rs:27xx`, stop — you have crossed into the correct ones.**

## ★ THE 2a LESSON, APPLIED BEFORE YOU START

Stone 2a's lint template was **asserted verbatim by a test**, and changing one without the other goes
red. **Before editing any of the 42, grep for a test asserting its text.** An error message is a
contract someone may have pinned. Report every asserting test you find and update it in lockstep.

★ **And `@example` lines are PUBLISHED DOCUMENTATION** (P6-a). `render-doc` prints them;
`tests/reflection/probe_arc255_spec_complete.rs:107` pins render-doc output **byte-identically** for
at least one verb. Expect a golden to move; **say which, with the new bytes — never edit a golden
silently.**

## STOP triggers — each REJECTS.

1. **A message you cannot classify** as param-spec-slot vs single-type-slot. Report it; leave it.
2. **An `@example` whose rewrite changes what it demonstrates.** The example's *point* must survive;
   if `:- [...]` makes it wrong or confusing, report it.
3. **A golden moves and you cannot show the new bytes.**
4. **You reach for the wall** — no parse/check behaviour changes here.

## Acceptance

```
 0. ★ YOUR OWN CENSUS of A and B, with the commands. My 8/34 is a starting point; a disagreement is
      a finding — my grep has been wrong in BOTH directions on this campaign (short 178 on stone 1,
      over by 66 on stone 2a).
 1. ★ THE ASSERTING-TEST SEARCH, done BEFORE editing. Every test pinning any of the 42, named.
 2. ★ ALL 8 DIAGNOSTICS advise only `:- [T …]`. Quote one before/after in full.
 3. ★ THE SINGLE-TYPE MESSAGES ARE UNTOUCHED — `check.rs:2751/2760/2808/2847` unchanged. Say it.
 4. ★ ALL 34 @example LINES rewritten, and each still demonstrates what it demonstrated. Any you
      could not rewrite: reported with the reason.
 5. ★ EVERY GOLDEN THAT MOVED, named, with new bytes.
 6. ★ REBUILD, then test. `--check` alone is not evidence (stone 1: a --check-clean edit under
      `wat/` broke 97 tests).
 7. ★ NO BEHAVIOUR CHANGE — the bare form still parses. Prove it: `(:wat::core::Vector
      :wat::core::i64 1 2 3)` still evaluates to `[1 2 3]` after your edits.
 8. ★ AFTER: your row-0 commands return 0 for A and B.
 9. cargo build --release --all-targets — clean; warnings VERBATIM.
10. cargo nextest run --release -E 'test(reflection) + test(intrinsic) + test(collection) + test(check)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.

## Report back with

Your census and commands. Every asserting test found. One diagnostic before/after in full. Proof the
single-type messages are untouched. The 34 example rewrites (or refusals with reasons). Every golden
that moved with its new bytes. The no-behaviour-change proof. Confirmation you rebuilt. Then the
honest deltas — especially any message whose slot you could not classify.
