# AMEND 2 — STONE 255.87: group B, and the arms group A unmasked

**Drawn 2026-10-03.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `ed2b7fb2d`
(group A cured; `.floor/2026-10-03T08-02-27Z`: 6373 passed, 26 failed, 4 timed out). Commit locally on `main`;
**do not push**.

Group A is accepted as measured (six cures, each with its probe; #9 cleared as downstream of #1). The cost table is
accepted: **1.47× / 1.48×** on two isolated workloads, startup's phase `4-register-defmacros` 6.9 s of 13.9 s. **The
cost is amendment 3**, next; this amendment does not touch it and raises no limit.

**Write every new fixture in the target spelling** (symbol heads **and** symbol names: `probe.a3/Hit`, not
`:probe::a3::Hit`), so it needs no conversion in 5c-iii.

## The unmasked arms (substrate first)

- **`wat grep` matches nothing on the converted stdlib** (7: `g1` "sample fixture must have at least one node; got 0",
  `g4`–`g7`, `written_refuses_a_string_literal`). Likely `wat/grep.wat` classifying nodes or heads by keyword text: find
  it, route through `:wat::core::canonical-identity` (5b's door), with a keyword/symbol pair test.
- **Emitted `defn` binders read back empty** (`probe_arc255_24_defservice_declares_what_it_emits.rs:87`, `left: []`):
  say whether the emitted `defn`s lost their binders (substrate) or the test reads them by keyword head (reader), and
  cure that.
- **An example renders 1154 wide** (`metadata_of_example_formats.rs:48`, limit 120): say why the converted form is
  rendered on one line (or what it now includes) and cure the renderer or the reader; never raise the limit.

## Group B (readers and goldens)

- **#5 nested-program census, #10 stdio gate, #15 `structtype` text:** each reads keyword text; read by identity, proven
  on both spellings.
- **#12 faithful-surface non-vacuity:** its own doc says it is re-aimed when the conversion lands. Re-aim it at what it
  must now discriminate (say what), never loosen its floor.
- **#4 span goldens (7), #13 doc-row bytes, #14 lost-arm text:** re-capture **only** after showing, as data, that each
  differs from its pre-image only by the stdlib's spelling or by positions in the converted stdlib (the 255.81 audit:
  normalize, compare as data, list each). #14's `:wat.seq/remove-at` / `:wat.spawn/` (a keyword with dots and a slash)
  is worth one sentence: is that a spelling the expansion should still emit?

Then the floor (the time-limit rows remain until amendment 3; list them), clippy, `git status` clean. STOPs as before.
A STOP means STOP. Append to the SCORE, commit, **do not push**.
