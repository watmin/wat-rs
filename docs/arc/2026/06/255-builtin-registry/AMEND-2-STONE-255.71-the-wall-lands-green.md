# AMEND 2 — STONE 255.71: the wall lands green (M1: the codemods and the Rust fixtures are programs too)

**Drawn 2026-09-30.** **Executor: a Sonnet subagent.** Continues the stone at the two local commits `2fb4578a9` (the wall,
the table) and `332dd3f3e` (its SCORE), plus the WEIGH `63a2518db`. Commit locally on `main`; **do not push**.

## The ruling (builder, 2026-09-30): M1

**The recorded codemods are wat programs run by the same runtime, so the wall applies to them like any other program.**
No path exemption, and none retired. The same holds for wat written inside Rust string literals.

## Read first

- `WEIGH-STONE-255.71-the-untyped-constructor-wall.md` (what is red and why) and `SCORE-STONE-255.71-…` (the full list).
- `TABLE-STONE-255.71-typed-constructors.edn` (the committed per-site table; its format is the model for new rows).
- `wat-scripts/fixes/typed-constructors.wat`, and `wat/fix.wat`'s header (the STASH-DANCE / BOOTSTRAP note).
- The red floor `.floor/2026-09-30T23-01-26Z` (62 failed / 6235): `every_recorded_migration_replays` 34,
  `wat_scripts_fixes_load`, `rete` 40, `rete_compile_gate` 32, `probe_arc278_seq1b_list_hofs` 4, `collection` 4,
  `types tuple` 2, `probe_arc170_wrong_service_compile_error` 2, `stdlib_door_reads_a_set_as_one_world` 2, `runtime` 2.

## The work

1. **The recorded codemods (`wat-scripts/fixes/**`, 84 files, 674 sites, measured by the first agent).** Convert them the
   way the corpus was converted: build rows for their untyped constructor sites (the checker's type record where it
   types them; the site's own code where it does not; a template's own type variable inside a quasiquote), append
   them to the committed table (or a sibling table beside it, said in the SCORE), and apply with `typed-constructors.wat`.
   **A tool is never its own input:** `typed-constructors.wat` does not convert itself. Convert it (and any codemod it
   depends on) under the stash-dance, with the wall lifted while the codemod runs, and say exactly how. The replay
   fixtures (`wat-scripts/fixes/replay/**` `before.pre` / `after.post`) are a codemod's **input and expected output
   text**: they change only if the codemod's own behaviour changed. Report each one you touch and why.
2. **Inline wat in Rust string literals.** First take a **census**: every Rust string literal in `src/**` and `tests/**`
   that wat's reader parses into a form containing an untyped collection constructor (the first agent's ~40-file list was
   a first pass). Then convert each. These are Rust files, so edit them directly; the codemod doctrine is for `.wat`
   files. Keep each literal's meaning; add only the bracket and its type.
3. **This stone's own reds.** Every red on `.floor/2026-09-30T23-01-26Z` is this stone's: include
   `probe_arc170_wrong_service_compile_error`, **which passed at 255.70** (`.floor/2026-09-28T22-13-36Z`), so it was not
   "pre-existing". Diagnose each from the kept log and cure it.
4. **The two genuine STOP-1 sites** in the table (a value that really holds different types, or a template with no
   type available): list them for the builder with file:line and why, and leave them unconverted. If the wall's reach
   gate counts them, record them as the gate's named exceptions with their reason.
5. **Gates:** the release floor (`scripts/floor.sh`, **one run at a time, in the foreground**), clippy, census `--diff`
   against `.census/2026-09-28T23-59-17Z.txt`, idempotence of the codemod, and the wall's reach (0 untyped constructors
   outside `.wat.bad`, `.wat.golden` and the two named STOP-1 sites). Append everything to the 255.71 SCORE.

## Reds and STOPs (checked against the work list: none fires on a site it orders changed)

- **A red caused by this stone's own gap:** capture the block **verbatim** from `.floor/<stamp>/`, cure it, run a **new**
  floor. Never re-run unchanged code for a green. A red you cannot trace to this stone: STOP.
- **Never call a red "pre-existing" without the prior floor's line for that test.** The prior green floor is
  `.floor/2026-09-28T22-13-36Z` (6225/6225); a test that passed there and fails now was caused here.
- **STOP-1:** more than five new sites (beyond the two known) whose type needs a design decision. List them all and STOP.
- **STOP-2:** converting a codemod changes what it produces on its own replay fixture (its behaviour, not just its
  spelling). Quote it and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you. Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number,
file:line or example you did not measure. If this amendment contradicts the code, the code wins: say so. Commit with
`git add -- <paths>`, never `-A`. **Do not push.**
