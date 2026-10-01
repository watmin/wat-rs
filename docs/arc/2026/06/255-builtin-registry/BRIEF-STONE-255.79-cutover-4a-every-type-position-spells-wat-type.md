# BRIEF — STONE 255.79: cutover 4a — every type position spells `wat.type/`

**Drawn 2026-10-01 against `main` @ `eca5933bf`.** **Executor: a Sonnet subagent, working solo** (it runs the floor; pass
`timeout: 600000` on that Bash call, or the harness moves it to the background after 120 s). A codemod strike over `.wat`
(and wat embedded in Rust string literals), no change to the checker's keys. Commit locally on `main`
(`git add -- <paths>`, never `-A`); **do not push**. Your final message is your report.

## The rulings

- **C1 (2026-09-27):** the new spelling is the canonical key. **S-b + P-surface (builder, 2026-10-01):** stone 4 is two
  stones. **This one (4a)** converts every remaining type-position spelling of the 24 hard primitives to `wat.type/X`; it
  lands green under today's door (`type_denotation`, `src/edn/render.rs:3706`, maps `:wat::type::X` to the registered
  `:wat::core::X`). **4b** (next) flips the key, deletes that mapping, refuses `:wat::core::X` in type positions, and makes
  types print as written.
- `.wat` migrations go through the recorded codemod; **a tool is never its own input**; M1: the codemods and wat embedded
  in Rust are programs too.

## The measurement (orchestrator, at `eca5933bf`)

7,335 tokens `:wat::core::<one of the 24>` remain in 447 `.wat` files (approximate classifier: 496 in comments, 260 inside
strings, ~2,770 directly after `(`, ~4,200 other code). Of the non-`(` code tokens, by tree: `wat-scripts/fixes/` 2,038
(the recorded codemods' own headers: stone 2 excluded that tree), `wat-scripts/scratch-pad/` 149, `tests/types` 82,
`tests/resolve` 33, `wat-scripts/probes` 26, the rest under 25 each, `wat/` about 100. Stone 2 (`WEIGH-STONE-255.67-…`
§ Residue) also left **79 `:nature :wat::core::Struct`** type positions its rules (A–E) do not see, and the typed-constructor
stones (255.69–.71) added `(:wat::core::X :- [...])` heads after stone 2 ran; its rule (A) converts those.

## The work

1. **Census first** (report it): run `wat-scripts/fixes/types-to-wat-type.wat` in dry-run over **every** tracked `.wat`
   file (including `wat-scripts/fixes/**`, `scratch-pad/**`, probes, tests) on `/tmp` copies, and count what it would
   change per tree. Separately count wat programs embedded in Rust string literals (`src/**`, `tests/**`) that contain a
   type-position `:wat::core::<24>` (255.71's AMEND-2 census of embedded wat is the method). If the embedded count is more
   than 600 sites, STOP-1 with the census.
2. **The missed rule:** add to `types-to-wat-type.wat` the `:nature :wat::core::Struct` (and any `:nature` value among the
   24) position, as rule F, with its own replay case. Recorded codemods are amended, not forked; record it in the header.
3. **Apply** to every `.wat` file it changes, listing every path. **The codemods themselves** (`wat-scripts/fixes/**`):
   run a pristine `/tmp` copy of the tool over them, never a file over itself; for `types-to-wat-type.wat` and any
   codemod it `load!`s, the copy that runs is the one from before this stone's edits. Their replay fixtures
   (`wat-scripts/fixes/replay/**`) are a codemod's input and expected-output **text**: they change only if a codemod's
   behaviour changed. Report any you touch and why.
4. **Embedded wat in Rust:** convert the type positions directly (Rust is edited by hand; keep each literal's meaning).
5. **Out of this stone, untouched:** comments (prose; stone 7), function and form heads that are not `(X :- …)` (stone 5),
   data keywords (a map key, `type-of`'s argument), the Rust key literals and printers (4b).

## Gates

| what | how | expected |
|---|---|---|
| idempotent | the codemod over the whole corpus after applying | 0 changes |
| residue | the census in item 1, re-run | every remaining code-position `:wat::core::<24>` listed by class (head, data, string) with file:line, none a type position |
| census | `scripts/replay/census.sh` pre (unmodified draw) and `--diff` after | no rc flips |
| delta | `scripts/replay/delta.sh` on the converted files | NEW 0, RECOVERY 0 |
| release floor | `scripts/floor.sh`, **in the foreground, `timeout: 600000`**, nothing else running | all passed; the count against 6349 at `f12c10995` (`.floor/2026-10-01T18-41-07Z`) |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |

## Reds and STOPs

- A red caused by this stone's own change (a converted site, a codemod's replay, a golden whose only change is the spelling):
  capture it **verbatim** from `.floor/<stamp>/`, cure it, run a **new** floor. Never re-run unchanged code for a green.
- **STOP-1:** the embedded-Rust census exceeds 600 sites. Report it and STOP before converting Rust.
- **STOP-2:** converting a codemod changes what it produces on its own replay fixture. Quote it and STOP.
- **STOP-3:** a conversion changes what a program computes or whether it checks (a census rc flip, a delta NEW). Quote it
  and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you; read `wat/fix.wat`'s header (STASH-DANCE / BOOTSTRAP). Capture `rc=$?` on the next statement.
Never wait with `pgrep -f`. Run every build, floor and clippy in the foreground and block on it. Never write a number,
file:line or example you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.79-cutover-4a-every-type-position-spells-wat-type.md` beside this brief, commit it, **do not push**.
