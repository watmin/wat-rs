# BRIEF — STONE expand-T3: make `@ExpandTime` required, sweep every site

Read `DESIGN-STONE-expand-t3-declaring-nothing-is-illegal.md` first.

## The work, one paragraph

`@ExpandTime` is optional and defaults to `Unreviewed` at two resolution points. **Make it
required**, then give every registration that lacks it an explicit `@ExpandTime    Unreviewed`
line so the tree compiles again with every verb having declared. `@Total`'s own T3
(`56f95c5fb`) is the worked precedent — read that commit before starting.

## Read in order

```
crates/wat-doc/src/lib.rs:712    `parse`              .unwrap_or(Unreviewed) -> .ok_or(Missing…)?
crates/wat-doc/src/lib.rs:1045   `parse_special_form` the same. BOTH.
crates/wat-doc/src/lib.rs:~221   DocError — add MissingExpandTime beside MissingTotality
crates/wat-macros/src/wat_intrinsic.rs  render_doc_error — the THIRD exhaustive match. Your new
                                 variant WILL break it with E0004. Expected; add the arm.
src/intrinsic/ast.rs:~308        `fresh-symbol` — the ONE site already declaring. It reads
                                 `@ExpandTime    Legal` and MUST NOT be swept to Unreviewed.
```

## Placement — uniform

After `@Total`, before `@Category`, aligned like its siblings:

```
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Total         Unreviewed
/// @ExpandTime    Unreviewed
/// @Category      Arithmetic
```

## Method — a surgical Rust tool under `tools/`, deleted before you finish

Read file → insert one line after the `@Total` line of each doc block that has no `@ExpandTime` →
write. Every other byte untouched; never a whole-file rebuild. **Confirm each file's non-ASCII
character count is identical before and after** — this repo once lost 5,720 non-ASCII characters to
a round-trip while the suite stayed green.

## ⚠ FOUR TREES CARRY DOC FIXTURES, NOT TWO

`@Total`'s T3 was briefed against `-p wat-doc -p wat-macros` and MISSED three reds, because the
`tests/` tree belongs to the `wat` package. Measured before this sweep: **`tests/` holds 6 fixtures
carrying `@Total`.** They will need `@ExpandTime` too. So do the doc-string fixtures inside
`wat-doc` and `wat-macros`' own unit tests.

★ **`tests/reflection/probe_arc255_axes_are_declared_not_derived.rs` will break, and how you fix it
matters.** That file's thesis is *"the axes are DECLARED, not derived"* — it asserts `doc.purity`,
`doc.determinism` and (since totality's T3) `doc.totality` are read off the doc. **Extend its CLAIM
to assert `doc.expand_time` as well**, not merely add the directive to its fixture. A new axis
nothing there asserts is a new axis that file's thesis has quietly stopped covering.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **You are about to write `@ExpandTime Legal` (or `RuntimeOnly`, or `Preserving`) on a real verb.**
   No verb is adjudicated here. Doc blocks that plainly describe an expand-time-safe verb still get
   `Unreviewed`; note them in your report. STOP.
2. **You are about to seed values from `is_expand_time_legal`'s allow-list.** T4a's job, with
   attribution. STOP.
3. **`fresh-symbol` loses its `Legal`.** It is T2's deliberate annotation and this stone's control.
   STOP.
4. **A file's non-ASCII count changed.** STOP and report the file and delta.
5. **You are about to weaken or delete a test to get green.** STOP.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: the count of registrations lacking @ExpandTime, measured BEFORE any
      edit, by your own command. The design predicts 432 and is NOT authoritative.
 1. ★ BREAK THE DOOR FIRST, and keep the artifact as a test: a fixture registration with NO
      @ExpandTime must FAIL TO COMPILE with MissingExpandTime. Prove the requirement is real
      BEFORE sweeping — afterwards nothing in the tree can demonstrate it.
 2. ★ The rendered MissingExpandTime message names all four legal values. Quote it verbatim.
 3. ★ Every registration declares. Your own count after, by the same command as row 0.
 4. ★ `fresh-symbol` still reads `@ExpandTime Legal`, and
      `expand_time_is_carried_from_the_doc_into_the_registry_entry` still passes.
 5. ★ THE AXES PROBE'S CLAIM EXTENDED, not just its fixture. Quote the assertion you added.
 6. ★ NON-ASCII INTEGRITY: per-file counts identical. State how you checked.
 7. ★ `ls tools/` is EMPTY at the end.
 8. ★ `git diff --stat` outside the two crates shows ONLY doc-comment additions — no body, no
      signature, no test logic beyond row 5's assertion. Say so.
 9. cargo build --release --all-targets — clean; warnings VERBATIM if any.
10. ★ THE FULL TARGETED SUITE, not two crates:
      cargo nextest run --release -E 'test(intrinsic) + test(macro) + test(reflection) + test(lint) + test(types)'
```

★ **Row 1 must come FIRST.** Once every site declares, "absence is an error" has nothing left in the
tree to fire on. Prove the wall while something can still hit it.

★ **Row 10 exists because row 9 is not enough** — and because the identical stone for `@Total`
shipped with acceptance rows that could not see the reds it caused.

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.
- Build any transform tool as a Rust Cargo binary under repo-local `tools/`.

## Report back with

Your row-0 count. Row 1's compile failure, verbatim. The rendered message. Your row-3 count.
`fresh-symbol`'s status. The assertion you added to the axes probe. How you verified non-ASCII
integrity. Confirmation `tools/` is empty. Row 10's result. Then the honest deltas — especially
**any verb whose own prose made you want to write something other than `Unreviewed`**, because
that list is T4a's head start.
