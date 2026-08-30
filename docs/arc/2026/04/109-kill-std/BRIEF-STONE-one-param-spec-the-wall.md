# STONE — ONE PARAM-SPEC, stone 3: THE WALL

> **Builder: *"stone 3 - build the wall"*.** The last stone. Every authority now says `:- [...]`;
> only the parser still accepts the other two. This makes the ruling structurally true.
> Read `NOTE-a-parametric-literal-has-three-spellings-and-no-authority-names-all-three.md`.

## The two doors

```
BARE      src/check.rs:14962   `let elem_ty = match &args[0] { WatAST::Keyword(k, _) => …`
                                and its HashSet / HashMap siblings — a bare keyword taken as
                                the element type.
UNMARKED  src/check.rs:12175   `unwrap_type_param_bracket`'s `[WatAST::Vector(inner, _), rest @ ..]`
                                arm. ★ Its own comment already says "③ deletes it."
```

## ★ THE CONTRACT DECISION — the bare form becomes UNREPRESENTABLE, internally too

⚠ **The checker MANUFACTURES the bare form.** `src/check.rs:2124`: a `[1 2 3]` literal is inferred by
**synthesizing `:wat::type::Infer` as `args[0]`** and calling `infer_list_constructor`. So the
bare-keyword arm is load-bearing for the checker's own synthesis, not only for user source — and
`{...}` / `#{...}` reach it the same way.

There are two ways to close the door, and **only one of them is a wall**:

- ⛔ **Keep the arm, accept only `:wat::type::Infer` through it.** One line, and it leaves the door
  open with a convention holding users out. That is the CONVENTION rung.
- ✅ **Close the arm entirely, and move the checker's own synthesis to `:- [...]`.** Then the bare
  form has **no representation anywhere** — not in source, not in a synthesized AST — and the
  checker's own literals are the proof the canonical form works. **This is the stone.**

Same choice `unwrap_type_param_bracket` faces: delete the unmarked arm, do not special-case it.

## ★ THE METHOD — the wall's red list IS the census

**Do not grep for what to fix first. Put the wall up and read the failures.**

My greps have been wrong three separate ways on this one campaign — short by 178 (blind to bare
type-reference position), over by 66 (could not tell a kwargs field name from a param-spec), and
short by 2 (a line-level filter dropped lines carrying BOTH forms). **The compiler and the floor
cannot make those mistakes.** Every site that still matters will name itself.

`docs/SUBSTRATE-AS-TEACHER.md` is the doctrine and FM 15 is the discipline: **a large fail count is
the progress meter, not a crisis.** Expect a cascade; watch it waterfall to zero. Do not stash, do
not revert, do not ask whether to step back.

⚠ **~159 `.rs` sites carry the old form in prose, fixtures and embedded source strings.** Only the
ones that actually PARSE will go red — and those are exactly the ones that matter. **Fix what the
wall names.** Dead prose is a follow-up sweep, not this stone's blocker; report the count you leave.

## STOP triggers — each REJECTS.

1. **You cannot close a door without special-casing `:wat::type::Infer` in the bare slot.** That is
   the convention rung; report it rather than shipping it.
2. **A failure you cannot explain from the wall.** Report it before changing anything else — a red
   you did not cause is a finding.
3. **You reach for `git stash` or a revert** because the count is large. FM 15. The failures are the
   work.
4. **A `.wat` corpus file needs editing.** Stone 1's codemod cleared them; a red there means the
   codemod missed a shape, which is a FINDING about stone 1, not a hand-edit licence.

## Acceptance

```
 0. ★ BOTH DOORS CLOSED, and the checker's own synthesis emits `:- [...]`. Show both diffs.
 1. ★ THE BARE FORM IS REJECTED: `(:wat::core::Vector :wat::core::i64 1 2 3)` now fails --check.
      Paste the diagnostic — and confirm it is the message stone 2b already rewrote, so it is now
      TRUE rather than aspirational.
 2. ★ THE UNMARKED FORM IS REJECTED: `(:wat::core::Vector [:wat::core::i64] 1 2 3)` fails --check.
 3. ★ THE CANONICAL FORM STILL WORKS: `(:wat::core::Vector :- [:wat::core::i64] 1 2 3)` -> [1 2 3].
 4. ★ LITERALS STILL WORK — `[1 2 3]`, `{:a 1}`, `#{1 2 3}` all check and evaluate. These go through
      the synthesis you just moved; they are the proof it landed.
 5. ★ THE FIRST FULL FLOOR AFTER THE WALL, reported as a NUMBER, before any fixing. That count is
      the stone's honest size.
 6. ★ THE WATERFALL — each round's count, and what class each round fixed. Not a summary: the
      sequence.
 7. ★ EVERY SITE THE WALL NAMED, fixed. Any `.wat` corpus red reported as a stone-1 finding.
 8. ★ THE PROSE SITES YOU DID NOT TOUCH, counted, with the command. They are a follow-up.
 9. cargo build --release --all-targets — clean; warnings VERBATIM.
10. cargo nextest run --release  (the FULL floor — this stone earns it) → 0 failed.
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** Clippy is the orchestrator's; **the floor is YOURS this once** —
  it is the instrument that names your work, and row 10 is the acceptance.
- No `git stash`. Do not commit, push, revert, or create a worktree.
- ⚠ `wat/*.wat` is FROZEN INTO THE BINARY. Rebuild before believing any result there — stone 1 lost
  97 tests to a `--check`-clean edit under `wat/`.

## Report back with

Both door diffs and the synthesis change. The three form probes (bare rejected, unmarked rejected,
canonical works) and the literal probes. The first post-wall floor count. The waterfall, round by
round. Every site fixed, grouped by class. The untouched-prose count. Then the honest deltas —
especially any shape the wall rejected that you think it should not have.
