# BRIEF — STONE 255.70: a constructor's type bracket reads through the type door; `List` takes `:-`; convert the rest

**Drawn 2026-09-28 against `main` @ `f99789bc7`.** **Executor: a Sonnet subagent** (grok's credits are out). A strike:
`src/` (the checker), the corpus sites 255.69 could not convert, tests. Commit locally on `main`
(`git add -- <paths>`, never `-A`); **do not push**. Your final message is your report.

## The rulings (builder, 2026-09-28)

- **One constructor shape** for every collection: `(wat.type/X :- [T…] items…)`, empty or not; an untyped constructor call
  is illegal (a wall, next stone, enforces it).
- **Types are read at one door:** K1's `canonical_type_key` and the type parser (`parse_type_node`, `src/types.rs`). A
  hand recognizer that accepts only `WatAST::Keyword` for a type is the class 255.67 retired five times.

## What 255.69 left (`WEIGH-STONE-255.69-typed-constructors.md` and its SCORE; read both)

- **35 sites with a compound element type**, e.g. `(wat.type/PersistentVector :- [(wat.type/Tuple :- [A B])])`:
  `parse_bracket_type_keyword` (`src/check.rs`, arc 109) accepts only a bare keyword in each slot of a constructor's own
  `:-` bracket. Confirmed in isolation, empty and with values.
- **74 `List` sites:** `infer_linked_list_constructor` (`src/check.rs` ~16486) does not peel the `:-` binder, so
  `(wat.type/List :- [wat.type/i64] 1 2 3)` does not check. (The run-time side: check it too.)
- **1 site** (`wat-tests/gen.wat:354:3`) kept identical to its duplicate-`defn` twin at `:837:3`, which is not checked.
- The full lists of the 35 and the 74 are in 255.69's SCORE.

## The work

1. **The constructor bracket reads through the type door.** Each slot of a constructor's `:-` bracket is parsed by the
   type parser (so a symbol, a keyword, a nested `(Head :- [...])` form and a fn-type bracket all read the same), not by
   a keyword-only recognizer. Cure it at `parse_bracket_type_keyword` (or retire it for the door). Search for any other
   constructor/bracket path with the same keyword-only shape (`unwrap_type_param_bracket` and the constructor intrinsics'
   run-time peel included), and route each through the door. Report each site.
2. **`List` takes `:-`** in the checker and at run time, exactly as `Vector`/`PersistentVector` do (the same peel, the same
   element type). `(wat.type/List :- [wat.type/i64] 1 2 3)` checks and runs; the empty `(wat.type/List :- [T])` is the
   empty list in value position.
3. **Convert the 109 sites** (35 + 74) with the recorded codemod `wat-scripts/fixes/typed-constructors.wat` (extend it if
   the compound element or `List` needs a rule; build its type table the way 255.69 did). For the duplicate-`defn` pair
   `wat-tests/gen.wat:354:3` / `:837:3`: convert both identically, or leave both, and say which.
4. **Tests:** driven rows for a compound element type (a vector of tuples, a map of string to vector) and for `List` with
   `:-`, in value and type position, each checking and running.

## Gates

| what | how | expected |
|---|---|---|
| release floor | `scripts/floor.sh`, **one run at a time, in the foreground** | all passed; the count against 6219 at `b3e5c0dbc`, plus your tests |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | rc 0 |
| census | pre-census on the unmodified draw; `scripts/replay/census.sh --diff` after | no rc flips |
| delta | `scripts/replay/delta.sh` on the converted files (restore their pre-conversion content first, as 255.69 did) | NEW 0, RECOVERY 0 |
| idempotent | the codemod over the converted files | 0 changes |
| coverage | converted vs the 109 | stated; the remaining untyped constructor count corpus-wide, re-derived |

## Reds and STOPs (checked against the work list: none fires on a site it orders changed)

- **A red caused by this stone's own gap** (its own lint hit, its own fixture, a golden whose only change is this stone's
  spelling): capture the failing block **verbatim** from `.floor/<stamp>/` into the SCORE, diagnose it, cure it, and run a
  **new** floor. Never re-run unchanged code to get a green.
- **STOP-1:** any other red. Do not re-run. Quote it verbatim, name the arm, and STOP.
- **STOP-2:** reading a constructor bracket through the type door changes what an existing, old-spelling program is admitted
  or refused (a census flip, or a red outside this stone's rows). Quote it and STOP.
- A STOP means STOP.

## Doctrine

`holon/CLAUDE.md` binds you (codemods for `.wat`, never sed/python/hand edits; the floor via `scripts/floor.sh`; no
known flake). Capture `rc=$?` on the next statement. Never wait with `pgrep -f`. Never write a number, file:line or example
you did not measure. If this brief contradicts the code, the code wins: say so. Write
`SCORE-STONE-255.70-constructor-brackets-through-the-door.md` beside this brief, commit it, **do not push**.
