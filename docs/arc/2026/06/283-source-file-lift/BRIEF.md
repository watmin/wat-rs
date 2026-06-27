# BRIEF — Arc 283: lift SourceFile → :wat::source::File (dogfood the rename)

You are a single-hop sonnet executor in `/home/watmin/work/holon/wat-rs`. **Do NOT spawn sub-agents.
Do NOT run `git`.** Build, run the migration + tests, report. The orchestrator weighs independently.

## The work (one paragraph)

Lift the `SourceFile {path, source}` record out of `deporder` into a neutral home named by intueri:
`:wat::source::File` in `wat/source.wat` (loaded before deporder). The `.wat` ref-renames are DOGFOODED
through `fix::rename-keyword-prefix` — you write a one-shot wat driver that rewrites the corpus, NOT hand
edits. Then relocate the def, fix load order, and rename the 3 Rust fixtures (manual — the codemod can't
reach `.rs` strings).

## The contract — follow the DESIGN's migration order EXACTLY

Read **`docs/arc/2026/06/283-source-file-lift/DESIGN.md` § "The migration"** and execute its 7 steps in
order. The order is load-bearing: build the renaming binary at HEAD FIRST, run the dogfood, THEN move
the def + rebuild.

## Read in order (the rooms)

1. `docs/arc/2026/06/283-source-file-lift/DESIGN.md` — THE SPEC (7-step migration).
2. `wat/deporder.wat:25-27` — the `(:wat::Record::def :wat::deporder::SourceFile [path source])` to lift.
   `SymDef` (`:29-31`) and `Violation` STAY.
3. `wat/fix.wat:512-560` — `rename-keyword-prefix(old-prefix new-prefix src) -> migrated-src` (the
   dogfood tool) — note the arg order (old, new, src).
4. `wat/io.wat:15-25` — `read-file [path] -> String` and `write-file [path content] -> nil` (the driver
   uses these).
5. `src/stdlib.rs:277-289` — the registration block; insert `wat/source.wat` AFTER core.wat and BEFORE
   `wat/deporder.wat`.
6. `tests/probe_arc277_lint_if_ladder.rs`, `tests/probe_arc277_lint_concat_abuse.rs`,
   `tests/probe_arc277_1b_ladder_autofix.rs` — the `:wat::deporder::SourceFile` strings to manually rename.
7. `tests/probe_arc283_source_file_lift.rs` — un-ignore (remove the `#[ignore = "arc 283 …"]`).

## The dogfood driver (write, run, delete — do NOT commit it)

Create `wat/_rename_sourcefile.wat` exactly as in the DESIGN (rename-file helper + main renaming
`wat/deporder.wat`, `wat/lint.wat`, `wat-tests/lint.wat`). After `cargo build --release --bin wat`, run
`cargo wat wat/_rename_sourcefile.wat`. Then **read the diff** of each renamed file to confirm
ONLY the `:wat::deporder::SourceFile` prefix changed to `:wat::source::File` (comment-faithful — nothing
else moved). Delete the driver file when done.

## STOP triggers (halt + report, do not improvise)
1. If `rename-keyword-prefix` changes ANYTHING beyond the `:wat::deporder::SourceFile` → `:wat::source::File`
   prefix (a mangled comment, a moved unrelated token) — STOP, report the diff. (Content-integrity is
   separate from tests-green.)
2. If after the migration `grep -rn ":wat::deporder::SourceFile" wat/ tests/ src/` finds ANY survivor —
   STOP, the rename is incomplete; report which files.
3. If any floor count moves (lib/nursery/deftest) — STOP, the lift changed behavior; report. The lift
   MUST be behavior-preserving.
4. If the deporder gate goes non-zero (load order: source.wat must precede deporder.wat) — STOP, report.

## Blast radius
NEW `wat/source.wat`; EDIT `wat/deporder.wat`, `wat/lint.wat`, `wat-tests/lint.wat` (refs via dogfood),
`src/stdlib.rs` (register), 3 Rust fixtures (manual), un-ignore the lift probe. Temp `wat/_rename_sourcefile.wat`
(deleted). No other changes. No git.

## Verify (run these, paste output verbatim)
```
cargo test --release -p wat --test probe_arc283_source_file_lift         # 1/1 GREEN (File/path → "t.wat")
grep -rn ":wat::deporder::SourceFile" wat/ tests/ src/ | wc -l           # MUST be 0 (zero survivors)
cargo test --release --test test_stdlib_load_order 2>&1 | grep result    # deporder gate: 1 passed / 0 failed
cargo test --release -p wat --lib -- --test-threads=1 2>&1 | grep "test result"   # 929 passed / 36 failed (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"               # 260 passed / 1 failed (UNCHANGED)
cargo test --release -p wat --test nursery -- --test-threads=1 2>&1 | grep "test result"  # 893 passed / 4 failed (UNCHANGED)
cargo test --release -p wat --test probe_arc277_lint_if_ladder           # GREEN (fixture renamed)
cargo test --release -p wat --test probe_arc277_1b_ladder_autofix        # GREEN (fixture renamed)
```
Report: the per-file diff summary (esp. confirming the dogfood was comment-faithful), the command
outputs verbatim, and any delta. Do not claim green you did not see.
