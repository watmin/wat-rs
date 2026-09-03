# BRIEF — STONE 1c-0a-ii: three repoints, one codemod, zero deletions

DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-0a-ii-the-capability-outlived-the-name.md`

## The work, in one paragraph

Three `.wat` artifacts each call a verb that was retired, and each has a live successor that
expresses the same capability. **Repoint all three, augment each file's header to record what
happened, and delete nothing.** The builder's ruling governs: *"deletions must clear a high bar…
these do not meet the requirement for deletion.. we augment as they need."* Two of the three then
type-check a real path again; the first becomes RUNNABLE again and you will run it.

## The three repoints

```
① wat-scripts/scratch-pad/bench-reduce-foldl-vs-seqable-walk.wat:30
     :wat::core::reduce-walk        →  :wat::core::foldl-spec-walk        (pure rename)

② wat-scripts/probes/arc-170/probe-cap2-process-grantpath.wat:10
     (:wat::spawn::process/grants (:wat::core::Vector :- [:wat::capability::Grantable]))
                                    →  (:wat::spawn::process)             (form rewrite)

③ wat-scripts/scratch-pad/arc109-2iii-fn-bracket-destinations.wat:55
     :wat::core::tuple-get          →  :wat::core::first                  (pure rename)
```

**Read these before writing anything**, so each repoint is grounded rather than trusted:
`wat/seq.wat:283` (`foldl-spec-walk`'s signature — confirm it matches ARM B's call shape) ·
`wat/bracket.wat:690–740` (confirm a plain pool still fires GRANT-BOOT on `peer-pid → Some`) ·
`src/check.rs`'s `infer_positional_accessor` (confirm `first` is polymorphic over Tuple).

## PART 1 — one codemod, three rules

`.wat` corpus rewrites go through the self-hosted codemod, never a hand-edit, never python or
sed. That is R21.

Write **`wat-scripts/fixes/repoint-retired-heads-to-live-spellings.wat`**. Read `wat/fix.wat`'s
header first, then:
- for ① and ③, copy `wat-scripts/fixes/deprime-telemetry-sqlite.wat`'s shape — composed
  `:wat::fix::rename-keyword-prefix` calls, each given the **whole head**, mindful of
  `rename-valid-match?`'s boundary rule (`wat/fix.wat:632`);
- for ②, this is a **form rewrite**, not a rename — a 2-element list headed
  `:wat::spawn::process/grants` becomes a 1-element list headed `:wat::spawn::process`. Copy the
  structural shape from an existing form-changing fix (`wat-scripts/fixes/defrule-then-to-vector.wat`
  or `first-of-drop-to-nth.wat`) rather than forcing a rename primitive to do it.

**Dry-run against a `/tmp` copy and `diff` first.** Confirm exactly three changed call sites and
that every other byte — comments, formatting, spacing — survives identically. Then apply, then
re-run to confirm idempotence (second run = 0 changes). Commit the codemod; it is the recorded
migration.

## PART 2 — augment each repaired file's header

This is the builder's *"we augment as they need"*, and it is not optional. Add to each of the
three files a short header note recording: **the verb that was called, that it was retired, the
live spelling now used, and that the artifact's own claim is unchanged.** Two or three lines,
in the file's existing comment voice. The point is that the next reader does not have to
re-derive the archaeology this stone cost.

For ② also note the second dead name it carried: `:wat::capability::Grantable` was renamed to
`:wat::capability::Capability` (stone A), and the repoint drops that reference along with the
retired combinator.

## PART 3 — run the bench, report the number, change no citation

Once ① is repointed the bench RUNS again. Run it and report the figure it produces.

Three documents currently carry its `5.1×` on 2026-08-18's authority alone —
`docs/arc/2026/04/118-lazy-seqs-vs-threaded-streams/DESIGN-STONE-118.B6-native-foldl-over-seqable.md:20`,
`wat/seq.wat:266`, `wat/seq.wat:612`. **Do not edit any of them.** A number that differs
materially from `5.1×` is a finding for the builder, not an edit you make. Report what you
measured, with both block orderings as the bench itself runs them.

## Blast radius

`wat-scripts/fixes/repoint-retired-heads-to-live-spellings.wat` (new) · the three call sites that
codemod rewrites · the three files' header comments. **No `src/` change. No registration. No file
deleted. No citation edited.**

## STOP triggers — halt and report, do not improvise

- **STOP-1.** The dry-run diff is not exactly the three intended sites. Report it; do not
  hand-edit a file the codemod missed.
- **STOP-2.** A repoint's successor does not actually match — `foldl-spec-walk`'s signature
  differs from ARM B's call, or a plain `(process)` pool does NOT reach the GRANT-BOOT branch, or
  `first` does not accept a Tuple. **Report it; do not adapt the call to fit.** The DESIGN's
  claim would then be wrong and I need to know before anything ships.
- **STOP-3.** A repaired file fails `--check` or fails to run for a reason the repoint did not
  cause. Report the exact error.
- **STOP-4.** You are tempted to delete a file, or to edit one of the three citation sites.
  Neither is in this stone. The builder ruled deletions must clear a high bar and these do not.
- **STOP-5.** A test outside the expected set goes red. Copy that test's entire stdout and stderr
  block verbatim from `.floor/latest/raw.log`, name the exact assertion that fired, and report —
  before re-running anything.

## Verification, in this order

```bash
./target/release/wat --check wat-scripts/scratch-pad/bench-reduce-foldl-vs-seqable-walk.wat
./target/release/wat --check wat-scripts/probes/arc-170/probe-cap2-process-grantpath.wat
./target/release/wat --check wat-scripts/scratch-pad/arc109-2iii-fn-bracket-destinations.wat
cargo build --release 2>&1 | tail -20
./scripts/floor.sh > /dev/null 2>&1; echo "EXIT=$?"
grep -E "^\s+Summary" .floor/latest/raw.log | tail -2
cargo clippy --all-targets -- -D warnings 2>&1 | tail -20
```

Read the Summary line, never a piped exit code.

## Acceptance — derived, not estimated

```
call sites rewritten          exactly 3, in 3 files, by codemod
codemod idempotent            second run reports 0 changes
files deleted                 0        ⬅ the ruling's bar
headers augmented             3
GAP_A / GAP_B / DEBT      60 / 68 / 106 — ALL UNCHANGED, nothing is registered here
floor                    5127/5127 → 5127/5127
clippy                                    0
the bench                     RUNS, and its number is reported (not applied)
```

## Working rules

Everything foreground. You may not spawn sub-agents. Do not background the floor run. No
worktrees, no `git stash`, no `git revert`, no commit, no push — leave the tree dirty and report;
the orchestrator commits. If a successor turns out not to match, **"I cannot tell" and a STOP are
worth far more than a repoint that compiles but changes what the artifact measures.**
