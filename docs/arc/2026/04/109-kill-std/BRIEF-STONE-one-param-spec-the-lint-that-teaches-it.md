# STONE — ONE PARAM-SPEC, stone 2a: the sites a codemod cannot see, starting with the lint that TEACHES the heresy

> Stone 1 rewrote 1675 sites across 386 `.wat` files. It could not reach **string literals** — a
> form-tree codemod walks forms, and a string is a leaf. This stone takes the ones that matter most:
> **the lint whose suggested fix emits the bare form**, and its asserting tests.

## The subject

```
wat/lint.wat:258   " literals — use (:wat::core::contains? (:wat::core::HashSet :T lit…) var) instead"
wat/lint.wat:260   "(:wat::core::contains? (:wat::core::HashSet :wat::type::Infer {lits}) {var})"
```

`:258` is the **advice** a user reads. `:260` is the **autofix template** — the lint does not merely
recommend the heresy, **it writes it into the user's file.** Every application of this autofix
creates a new bare-form site, forever, in a corpus that was just cleaned of 1675 of them.

★ **This one has an ORACLE, which is why it is the right stone to take first.** The template is
asserted verbatim by tests:

```
tests/lint/probe_arc277_1b_ladder_autofix.rs:35            the expected autofix output
tests/collection/probe_arc215_collection_literal_inference.rs
tests/lint/probe_arc277_1b_ladder_autofix.wat              the fixture the autofix is applied to
```

Change the template without the test and it goes RED; change both consistently and it goes green.
Contrast the ~125 prose sites in stone 2b, where **nothing** would catch a wrong edit — which is
precisely how the doc lies W3 found (seven in ten verbs) accumulated.

## The work

1. **Census first** — every string literal in `.wat` carrying a bare or unmarked param-spec.
   My count is **68** (`grep -rnE '"[^"]*\(:HEAD :[A-Za-z]' --include=*.wat`, `;;`-lines excluded)
   but ⚠ **my `.wat` census was short by 178 on stone 1** and I do not trust this one either.
   **Your census governs; a disagreement with 68 is a finding, not a rounding.**
2. **Fix `wat/lint.wat`'s two templates** to `:- [...]`, and every test that asserts them, in lockstep.
3. **Fix the remaining string sites** your census finds — reporting any you cannot, with the reason.

⛔ **Do NOT touch `src/`.** The checker wall, the runtime diagnostic that recommends the bare form,
and the `.rs` doc comments are stones 2b and 3.

## ★ THE HAZARD — an autofix's output must still be what the autofix CLAIMS

`:258` and `:260` must agree with each other **and** with what the fixed corpus now looks like. A
lint whose message says one form and whose template writes another is worse than either alone. And
the fixture (`probe_arc277_1b_ladder_autofix.wat`) is the *input* to the autofix — check whether its
**expected output** is stored anywhere, and update that too.

★ **Stone 1's lesson applies here and is not optional:** `wat/lint.wat` is **stdlib, frozen into the
binary at build time.** A `--check`-clean edit there proved nothing in stone 1 — one bad rewrite in
`wat/rete/oracle/pass.wat` passed `--check` and broke 97 tests. **Rebuild, then run the scoped
suite.** `--check` alone is not evidence for anything under `wat/`.

## STOP triggers — each REJECTS.

1. **A test asserts the old string and you cannot find its fixture's expected output.** Report it;
   a half-updated autofix is worse than an un-updated one.
2. **A string site is ambiguous** — you cannot tell whether it is a param-spec or prose about one.
   Report it; do not rewrite.
3. **You are about to touch `src/`.**
4. **You reach for a hand-edit on a FORM site.** Stone 1's codemod owns those; a string is the only
   thing hand-editable here, and only because a codemod structurally cannot see inside one.

## Acceptance

```
 0. ★ YOUR OWN CENSUS of string-embedded param-specs in `.wat`, with the command. Disagreement with
      my 68 reported as a finding.
 1. ★ BOTH lint templates fixed, and EVERY asserting test updated in lockstep. Name each file.
 2. ★ THE LINT'S MESSAGE AND ITS TEMPLATE AGREE — quote both, before and after.
 3. ★ THE AUTOFIX STILL WORKS END TO END: run the ladder-autofix probe and show it produces the
      `:- [...]` form now. Paste the before/after expected string.
 4. ★ REBUILT, then scoped tests — NOT `--check` alone. State that you rebuilt.
 5. ★ EVERY OTHER STRING SITE fixed or reported with a reason. A zero-skip run is a finding.
 6. ★ `git diff --stat -- src/` EMPTY. Say it.
 7. ★ AFTER: your row-0 command returns 0 for the sites you claimed to fix.
 8. cargo build --release --all-targets — clean.
 9. cargo nextest run --release -E 'test(lint) + test(collection) + test(wat_scripts) + test(arc277)'
```

## How to work

- Work only in `/home/john/work/holon/wat-rs`. `pwd` first. Never a `.claude/worktrees/` path.
- **Everything FOREGROUND. Ending your turn ENDS you** — nothing wakes you, no notification is coming.
- **You may not spawn sub-agents.** The full floor and clippy are the orchestrator's.
- No `git stash`. Do not commit, push, revert, or create a worktree.

## Report back with

Your census and its command. The two templates before and after. Every test file updated. The
autofix end-to-end proof. The list of any string site you did not fix, with reasons. Confirmation
you rebuilt before testing. Then the honest deltas — especially any string you could not classify.
