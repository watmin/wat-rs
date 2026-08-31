# BRIEF — STONE total-T3: `@Totality` becomes REQUIRED

Read `DESIGN-STONE-total-t3-declaring-nothing-is-illegal.md` first. This brief is the strike path.

## The work, one paragraph

`@Totality <Variant>` is currently optional and defaults to `Totality::Unreviewed`. **Make it
required.** Absence becomes `DocError::MissingTotality`, which makes the registration macro refuse
to expand. Then give all 437 registrations that lack the directive an explicit
`@Totality         Unreviewed` line, so the tree compiles again with every verb having declared.
`@Purity` and `@Determinism` are the exact templates — every site you touch is one where
`MissingPurity` / `MissingDeterminism` already appear.

## Read in order — the rooms, and why each

```
crates/wat-doc/src/lib.rs:221    DocError::MissingDeterminism — the variant to mirror
crates/wat-doc/src/lib.rs:~678   `parse` resolution point. Today:
                                 `totality_val.unwrap_or(Totality::Unreviewed)`.
                                 Becomes `.ok_or(DocError::MissingTotality)?`
crates/wat-doc/src/lib.rs:~996   `parse_special_form` — the SECOND resolution point, a sibling
                                 struct with its own parse fn. Both must change.
crates/wat-macros/src/wat_intrinsic.rs:~578   render_doc_error's MissingDeterminism arm.
                                 ★ THIS IS A THIRD EXHAUSTIVE MATCH and it will fail to compile
                                 (E0004) the moment you add the variant. Expected; add the arm.
src/intrinsic/i64.rs:~171        `:wat::i64::/` — the ONE verb that already declares. It shows
                                 the exact placement and column alignment to reproduce.
```

## Placement — uniform, all 438

`@Totality` goes **immediately after `@Determinism`, before `@Category`**, aligned like its siblings:

```
/// @added         1.0.0
/// @Purity        Pure
/// @Determinism   Deterministic
/// @Totality         Unreviewed
/// @Category      Arithmetic
```

## Method — a surgical Rust tool, built under `tools/`

437 edits is well past hand-editing. Build a small Cargo binary under repo-local `tools/` that
reads a file, inserts one line after the `@Determinism` line of each doc block that has no `@Totality`,
and writes it back — every other byte untouched. `read_to_string` → targeted insert → `write`, never
a char-by-char rebuild. **Delete the tool before you finish**; it is scaffolding, not substrate.

★ **Confirm each file's non-ASCII character count is identical before and after.** A whole-file
round-trip in this repo once silently dropped 5,720 non-ASCII characters while the suite stayed
green — content integrity is a separate axis from tests-green, and it is yours to check per file.

## Blast radius

`crates/wat-doc/`, `crates/wat-macros/`, and doc comments across `src/` and `crates/`. **Only doc
comment lines are added.** No function body, no signature, no test assertion, no `.wat` file, and no
`@Totality` value other than `Unreviewed` — with the single exception that `:wat::i64::/`'s existing
`Partial` is left exactly as it is.

## STOP triggers — each REJECTS. Ship nothing; report.

1. **You are about to write `@Totality Total` or `@Totality Partial` on any verb.** No verb is adjudicated
   by this stone. If a doc block makes you certain a verb is partial, say so in your report and
   still write `Unreviewed`. STOP.
2. **You are about to migrate `is_pure_total` or `intrinsic_meta` membership into `@Totality` values.**
   Those lists answer different questions; the DESIGN measures why. STOP.
3. **You are about to change a consumer** — `src/rete/purity.rs`, `src/macros/eval.rs`,
   `src/rete/vocabulary.rs`. They keep their hand-lists this stone. STOP.
4. **A file's non-ASCII count changed.** STOP and report the file and the delta.
5. **You are about to delete or weaken a test to get green.** STOP.

## Acceptance

```
 0. ★ YOUR OWN PRE-CHECK: the count of registrations lacking @Totality, measured BEFORE any edit,
      by your own command. Report it. The design predicts 437 and it is not authoritative.
 1. ★ BREAK THE DOOR FIRST, and keep the artifact as a test: a fixture registration with NO
      @Totality must FAIL TO COMPILE with MissingTotality. Prove the requirement is real BEFORE
      sweeping — a sweep that lands first can make the requirement untestable.
 2. ★ The rendered MissingTotality message names all four legal values. Quote it verbatim.
 3. ★ Every registration declares. Your own count after the sweep, by the same command as row 0.
 4. ★ `:wat::i64::/` still reads `@Totality Partial` — the sweep must not flatten it. The existing
      test `totality_is_carried_from_the_doc_into_the_registry_entry` covers this; say it passed.
 5. ★ NON-ASCII INTEGRITY: per-file counts identical before/after. State how you checked.
 6. ★ `ls tools/` is EMPTY at the end. The tool is deleted.
 7. ★ `git diff --stat` shows ONLY doc-comment line additions outside the two crates changed for
      the requirement. No body, no signature, no test edits. Say so.
 8. cargo build --release --all-targets — clean; warnings VERBATIM if any.
 9. cargo test --release -p wat-doc -p wat-macros — green, counts reported.
```

★ **Row 1 is the load-bearing one and it must come FIRST.** Once every site declares, "absence is
an error" has nothing left to demonstrate it on. Prove the wall before you remove everything it
would catch. `[[feedback_impose_the_check_and_read_the_screams]]`

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is
  coming. Your turn ends when the numbers are in your hands.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- Do not commit, push, revert, stash, or create a worktree.
- Build any transform tool as a Rust Cargo binary under repo-local `tools/`.

## Report back with

Your row-0 count. Row 1's compile failure, verbatim. The rendered message. Your row-3 count. The
`i64::/` readback. How you verified non-ASCII integrity and the result. Confirmation `tools/` is
empty. Then the honest deltas — especially any doc block whose shape defeated the tool and needed
a hand edit, and any verb whose own prose made you want to write something other than `Unreviewed`.
