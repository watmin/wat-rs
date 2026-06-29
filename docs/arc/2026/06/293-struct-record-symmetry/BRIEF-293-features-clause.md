# BRIEF — 293: the `:features` clause introduces a surface's structural members (one canonical path)

**The work, in one paragraph.** A `defsurface`'s structural member vector is now introduced by an explicit
**`:features`** keyword clause (builder-crowned, 2026-06-29; an intueri cast preferred `:requires` but the builder
chose `:features` — "the most intuitive label; these are the surface features"). It pairs beside `:holder` as two
parallel clauses (categorical, then structural). This is **one canonical path**: the bare member-vector form
(`(defsurface :S [members])`) and the `:holder X [members]` form (member vector positional-last) **RETIRE** — a
member vector NOT preceded by `:features` is a malformed declaration. `parse_defsurface` is rewritten to read the
members from the `:features` clause; the ~15 existing `defsurface` decls migrate; the empty case `:features []` stops
being a cryptic naked `[]`.

## THE ONE CONTRACT DECISION (pinned)
The ONLY valid `defsurface` shapes are:
- `(:wat::core::defsurface :Name :features [members])`                              — no holder bound
- `(:wat::core::defsurface :Name :holder :<holder-root> :features [members])`        — holder + structural

`:holder` stays **optional** and comes **before** `:features`. `:features` is **mandatory** (it introduces the member
vector — even when empty: `:features []`). A member vector that is not the value of a `:features` clause → `MalformedDecl`.

## Read in order (rooms)
1. **`src/types/surface.rs:292 parse_defsurface`** — currently arity 2 (`name [members]`) / 4 (`name :holder :<kw>
   [members]`), member vector positional. Rewrite the arg walk: after the name keyword, the next token is EITHER
   `:holder` (then consume `:<holder-root>` via `Holder::from_root_keyword`, then REQUIRE `:features`, then the vector)
   OR `:features` directly (then the vector). Anything else → `MalformedDecl`. The member-vector parse + the
   293.4d-fix "nothing follows the vector" invariant are UNCHANGED — only the clause that introduces the vector moves
   from positional to the `:features` keyword. Update the `expected (...)` reason strings to the two new shapes.
2. **The ~15 existing `defsurface` decls** (`grep -rln 'defsurface' --include=*.wat . | grep -v target | grep -v
   wat-scripts/fixes`) — migrate each: insert `:features` immediately before the member vector.
   - `(defsurface :S [members])`            → `(defsurface :S :features [members])`
   - `(defsurface :S :holder X [members])`  → `(defsurface :S :holder X :features [members])`
   This is a FORM edit (insert a keyword before the last vector), NOT a keyword rename — `rename-keyword-prefix` can't
   do it; hand-edit each (they are few). Touch ONLY the `defsurface` forms; leave the surrounding fixture untouched.
3. **The committed RED probe** `tests/types/probe_arc293_features_clause.{rs,wat}` — goes GREEN when room 1 lands.

## Gate (EXPECTATIONS — fixed before the strike)
| what | command | expected |
|---|---|---|
| `:features` probe GREEN | `cargo nextest run --release -p wat features_clause_introduces_surface_members` | 1 passed |
| existing surface suite GREEN (post-migration) | `cargo nextest run --release -p wat probe_arc293` | all pass |
| bare-vector form retired (REJECTED) | a `(defsurface :S [m])` form now MalformedDecl | (verify via a negative arm or by the migration leaving none) |
| whole gate, floor 0 | `cargo nextest run --release` | `0 failed` |

Runtime estimate: 20–30 min. Trap-door: a `defsurface` whose member vector is genuinely empty already (the new 0-member surfaces) — make sure `:features []` parses.

## STOP triggers (rejection criteria)
- **STOP-HOLDER-ORDER:** `:features` MUST come after `:holder :<kw>` (not before). If any existing surface has an
  unusual clause order, surface it — do not guess.
- **STOP-LEFTOVER:** if migrating a surface leaves a member vector NOT introduced by `:features` (e.g. a multi-line
  decl you mis-edited), the parser will reject it — that is the gate catching an incomplete migration; finish it.
- **STOP-OTHER-FORMS:** touch ONLY `defsurface`. Do NOT touch `defrecord`/`defstruct`/`aggregatetype` (their field
  vectors are a different grammar, no `:features`).

## Blast radius (bounded)
`src/types/surface.rs` (the parser arg-walk + reason strings) · ~15 `.wat` fixtures (insert `:features`) · the RED
probe goes green. No new types. No runtime/codegen change (purely the declaration surface).

## You are a LEAF
Do NOT spawn subagents. Work only in `/home/watmin/work/holon/wat-rs/`. `pwd` first; `.claude/worktrees/` is illegal.
`cargo nextest run` (NEVER `cargo test`). Do NOT commit — leave the tree for the orchestrator to weigh.

## Pairs
`AGGREGATE-MODEL.md` §6 (the surface = `:holder` + structural members) · the breadcrumb `255/CURRENT-STATE.md`
RESUME POINT (the `:features` decision + the build order) · `tests/types/probe_arc293_features_clause.{rs,wat}`.
