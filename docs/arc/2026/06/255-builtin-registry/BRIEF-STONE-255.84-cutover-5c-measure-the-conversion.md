# BRIEF — STONE 255.84: cutover 5c, measured — what the head conversion does to today's tree

**Drawn 2026-10-02 against `main` @ `167256d8b`.** **Executor: grok via pulsare, working solo.** A **measurement**:
**no change to the tree.** Every conversion runs in a separate clone (`git clone --shared . /tmp/wat-5c` at
`167256d8b`; never a worktree) or on `/tmp` copies. The only commit is the SCORE (`git add -- <path>`); **do not push**.

## Why (H2, builder 2026-10-02)

5a refused an unbound slash-less head; 5b made every head decision go through its identity (ledger A/B 0). **5c** converts
~98,000 keyword call heads in `.wat` (and the embedded wat in Rust) to symbols. Before it is drawn, measure what the
recorded conversion does to **today's** tree.

## What exists (read first)

- The converter: `wat-scripts/fixes/to-faithful-clojure.wat` (and `-net`, `-rete`; say when each applies), driving
  `:wat::fix::fix-text` (`wat/fix.wat`). Span-faithful, comment-faithful, idempotent, and fail-safe (a file it cannot
  convert is left byte-unchanged).
- The prior measurement: `251-types-as-forms/BRIEF-STONE-251.8d-the-corpus-flip.md` (2026-09-19: three failure classes,
  ~1.5% of files on a corpus census, batch exits on first failure, ~2.7 s/file per-file) and the 8d-ii line
  (`BRIEF-STONE-251.8d-ii-*`, their SCOREs and WEIGHs: why each draw stopped). The SEAM's 8d sections. The committed delta
  sample `251-types-as-forms/delta-sample-179.txt` (**use the file; never rebuild it by index**).
- `wat/` is `include_str!`'d into the binary (`src/load/stdlib.rs`), and `wat/fix.wat` is the tool: **the stdlib cannot
  flip in the same pass as the corpus** (`wat/fix.wat`'s STASH-DANCE note).

## The measurements (report each as a table)

1. **Time one file first**, then choose batch, per-file, or a resumable driver (skip on failure, record it, exit 0),
   and say which and why.
2. **The corpus census** on copies of every tracked `.wat` outside `wat/`: per file OK (with the head count changed) or
   FAIL (class and first error). Group failures by class with counts and three examples each; compare the classes with
   2026-09-19's three.
3. **The stdlib bootstrap**, in the clone: convert `wat/` (with a pristine copy of the tool, never the tool over itself),
   build the clone's release binary, then run its floor (`scripts/floor.sh` in the clone, in the foreground, nothing else
   running). Report the summary and the reds **grouped by mechanism** (not cured). This is the question 8d-ii kept
   stopping on: does a converted stdlib load and run under today's checker?
4. **The corpus on the converted stdlib**, in the same clone: convert the delta sample's 179 files and run
   `scripts/replay/delta.sh` (NEW and RECOVERY, each listed by file and mechanism).
5. **Embedded wat:** `wat-fix-rust` with each converter, `--dry-run`, over every tracked `.rs`: files changed, edits found,
   refused splices (by reason).
6. **What the conversion produces that 5a/5b would now refuse or treat differently:** any converted head that is
   slash-less (5a refuses it), any converted name the identity door maps elsewhere, any `DuplicateMacro`-shaped
   transitional collision (the SEAM's `wat/holon/Ngram.wat` finding).

## STOPs

- **STOP-1:** a conversion changes a file in the main tree. Restore it and STOP.
- **STOP-2:** a measurement cannot run (the converter, the clone build, the floor). Report exactly where and STOP that
  measurement; finish the others.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number,
file:line or example you did not measure; every count names the command and the log. Write
`SCORE-STONE-255.84-cutover-5c-measure-the-conversion.md` beside this brief, commit it, **do not push**.
