# BRIEF — STONE 255.62: MEASURE by wall, layer 3 — the lib links; two walls up; run everything

**Drawn 2026-09-27 against `main` @ `43eb28dde`.** **Executor: grok, via pulsare.** **MEASUREMENT BY WALL**, continuing
255.60/.61. All code changes in a **`git worktree` under the session scratchpad** (symlink `../holon-rs` beside it).
**Nothing lands on `main` but the SCORE** and records under `docs/arc/2026/06/255-builtin-registry/probes-255.62/`
(the diffs of the two link cures and of the second wall). Commit those only; **do not push**. Then
`pulsare_yield kind=scored`.

## The rulings

- **The flip** (arc 251.8d): `::` keywords become faithful-Clojure symbols. The `::` wall (255.60) makes the old
  spelling unspeakable.
- **F1 (builder, 2026-09-27):** the dual type spelling ends **with the flip**. Every builtin type is spelled
  `wat.type/…` in every type position; **`wat.core/<type>` in a type position is illegal.** Scope, from the builder's
  2026-09-20 ruling (`68cefdebf`): only the **container** is ruled. Types keep their current names under `wat.type/`.
  Short names, case convention and the missing Rust scalars stay deferred. The equivalence `wat.type/X` ≡ `wat.core/X`
  through the denotation door (255.8; `src/types.rs:130-141`, `:326-330`) is what the second wall removes for type
  positions.

## Steps (in the worktree)

1. **Rebuild layer 2's state:** apply `probes-255.60/the-wall.diff`; convert the corpus exactly as 255.60/.61 did (the
   codemod, two-phase, with the no-wall binary); convert the doc directives with `probes-255.61/convert-doc-fragments.wat`.
2. **The two link cures** that 255.61 named (verified by the orchestrator):
   - the doc grammar accepts a type token that is a symbol (`crates/wat-doc/src/lib.rs:641` `@arg`, `:693` `@ret`),
     still gated by the reader check that follows;
   - `fqdn_of` stops re-minting `::` (`crates/wat-macros/src/edn_doc.rs:286`, `format!(":{wat_ns}::{name}")`): it keeps
     the dotted spelling the fence already wrote.
3. **The second wall (F1).** Where a type is parsed from a type position (the type parser and every type-position
   resolution the checker uses), a symbol `wat.core/X` whose `X` is a builtin type (a `wat.type` member; 255.1 made the
   members real) is refused, naming the token and saying types live in `wat.type`. The call-position meaning of
   `wat.core/…` is untouched. Report every place a type position is parsed, so the wall covers all of them.
4. **Link, then run with both walls up:** the release floor (`scripts/floor.sh`) and a census of every tracked `.wat`.
   If one blocker hides everything behind it, name it and stop there.

## Class every failure

- **(a)** a `::` not converted; **(b)** a name minted at run time with `::` (which of 255.60's listed sites fire, and who
  calls them); **(c)** `src/` reaching a name through a spelling that now fails; **(d)** a substrate break for another
  reason (include 255.60's `ReservedPrefix` on `--check`, and the 255.14 residue classes); **(e)** a meaning change
  (a namespaced keyword used as a value, now a symbol); **(f)** tests that pin the old spelling as their contract;
- **(h) F1: `wat.core/<type>` in a type position.** Count them, and **group them by the codemod rule that emitted them**
  (the syntactic position: a type argument inside `(Head :- [...])`, a field type, a bare type token, a doc
  directive, a variant payload, and so on). For each group, name the codemod change that would emit `wat.type/`
  instead (the position signal 255.5 established).
- **(g)** other.

Per class: counts and **the smallest set of cure sites that clears the most failures**. The converted floor number with
both walls up, against 78/6014 at 255.14.

## Doctrine

- A measurement. The walls, the conversions and the cures do not land. Do not fix what screams beyond the two named link
  cures; record it.
- Read, measure, cite file:line. There is no known flake; quote failing blocks verbatim. Capture `rc=$?` on the **next**
  statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Remove the worktree when done. Write `SCORE-STONE-255.62-the-wall-layer-three.md` beside this brief; commit it and
  `probes-255.62/` with `git add -- <paths>`. **Do not push.**
