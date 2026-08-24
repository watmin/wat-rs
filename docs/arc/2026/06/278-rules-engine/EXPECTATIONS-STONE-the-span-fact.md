# EXPECTATIONS — STONE: the Span fact

Written BEFORE the strike, so the result cannot move the goalposts.
BRIEF: `BRIEF-STONE-the-span-fact.md`. DESIGN: `DESIGN-STONE-the-span-fact.md`.

## The scorecard

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | `Span == Node` on every reported file | `./target/release/wat wat-scripts/scratch-pad/rules-corpus-03-source-to-facts.wat` | three `Node=N … Span=N` lines, N equal in each; `Named` still strictly below `Node` |
| 2 | a rule joins Node × Named × Span and binds a line | same run | a non-zero count for the new query + at least one bound line number printed |
| 3 | Span non-zero on a real file | same run | `wat/fix.wat  Node=4316  Named=<m>  Span=4316` |
| 4 | a RHS builds `:wat::core::Span` from bound `?line`/`?col` + a filename | `./target/release/wat wat-scripts/scratch-pad/probe-rhs-builds-core-span.wat` | a `#wat.core/Span {:file … :line … :col … :end …None}` nested inside the asserted fact |
| 5 | every touched/created file type-checks | `./target/release/wat --check <each file>` | EXIT=0 each |
| 6 | the loader gate still holds | ORCHESTRATOR, centrally: `scripts/floor.sh` | 5025→5025+, 0 FAIL; `every_wat_scripts_file_loads` green |
| 7 | clippy | ORCHESTRATOR, centrally | 0 under `-D warnings` |

Rows 1–5 are the rider's. **Rows 6–7 are the orchestrator's and are taken centrally, once, on a
quiescent tree** — the rider runs no cargo and no floor.

## Independent prediction

**Runtime: 20–35 minutes.** The edit is small and every room is named to the line; the cost is
concentrated in row 2 (authoring a three-way-join rule in an unfamiliar surface) and row 4 (nested
record construction inside a rete RHS).

**Line counts predicted:** corpus-03 `+45 / -8`; the row-4 probe `+40` new. No `src/`, no `wat/`.

## Trap doors, named before they open

1. **The five-field record in a rete condition.** Every existing corpus-03 condition binds from a
   2–4 field record. Nothing suggests five is different, and nothing has measured it. If it is
   different, STOP-3 fires and the finding belongs to rete's surface, not to this stone.
2. **The three-way join.** `:fx::type-pos` already joins `Node × Node × IsArrow` — three conditions
   with a shared variable — so the shape is proven. What is NOT proven is a join across three
   DISTINCT fact types on one `?id`.
3. **`Span == Node` could pass vacuously** if the rider guards the emit and the guard happens never
   to fire on the three sampled files. The control against this is `Named < Node` in the same output:
   `Named`'s guard demonstrably fires, so the two counts diverging while `Span` tracks `Node` exactly
   is the shape that means the emit is genuinely unguarded. **If `Span == Node == Named`, every
   reading is meaningless** and the run is void.
4. **Row 4 re-run.** Its predecessor ran beside a live writer. If the new output differs from the
   recorded `#p/Hit {:span #wat.core/Span {…}}` shape in any way, the DIFFERENCE is the finding — the
   old record is a claim, not a baseline.
5. **`:wat::core::Pos`.** Registered Rust-side via an EdnSchema drain, not as a `defrecord` in
   `wat/core.wat`. If the rider reaches for it, the row has drifted — `:end` is `None`.

## What would make me reject a green report

- Row 1 reported as a count with no per-file numbers. The control is per-file or it is not a control.
- Row 4 reported as "matches the recorded output" rather than reproduced verbatim.
- Any adjustment to `:fx::Named`'s guard. It is correct and it is this stone's only live control.
- `Span` emitted inside the `if` that guards `Named`. That is the exact defect rows 1 and 3 exist to
  catch, and it would still produce a plausible-looking run.
