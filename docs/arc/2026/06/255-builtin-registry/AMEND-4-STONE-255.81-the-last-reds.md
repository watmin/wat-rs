# AMEND 4 — STONE 255.81: the last reds

**Drawn 2026-10-02.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `91e868982`. Commit
locally on `main`; **do not push**.

Amendment 3 is accepted as measured: both STOP-2 arms pass, retired names refuse at the runtime doors, the driver now
reads `{{`/`}}` templates, rules L/M/N are recorded. What remains, in order:

1. **The 132 reds of `.floor/2026-10-02T07-50-06Z`.** Classify every one from the kept log. Each whose only change is the
   printed spelling of the 24 is re-captured from the program's new output **and** passes amendment 2's audit (the
   old and new expected values, parsed as data with the 24's spellings normalized, are equal). Any red that is not that
   (a different error, a different value, a different count, a test that now passes vacuously) is **STOP-2**: quote it.
   Report the classification as a table (test · arm · class).
2. **`tests/types/probe_arc255_81_retired_name_refuses.rs`:** its programs are fixed text, so they belong in a co-located
   `.wat` fixture driven by `startup_beside` (the lint's rule), not under `rune:lint(no-inlined-wat)`, whose reason must
   be a world built at run time. Move them and drop the rune.
3. **The floor**, in the foreground, nothing else running; clippy; the `wat-fix-rust` dry run of every recorded codemod
   used in this stone over all tracked `.rs` (`0 changed`); `census.sh --diff` against the clone pre-image
   (`.census/2026-10-02T06-48-31Z.txt`), every flip listed and explained.

STOPs as before. A STOP means STOP. Append to the SCORE, commit, **do not push**.
