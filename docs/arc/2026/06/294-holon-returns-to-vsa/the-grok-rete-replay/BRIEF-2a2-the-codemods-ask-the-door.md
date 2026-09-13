# BRIEF 2a2 — the codemods ask the door, once per program; convert.sh runs each codemod once per commit

> Built on: 2a1 (`5edca1211`, the declaration door) and 2a1b (`92d63df31`, `type-of` answers every name
> `is-type?` admits), both verified by the orchestrator. The exclusion loop in D1 is measured below.

**The rulings this executes:** (B) field names from a program's declarations; variants from declarations
and the registry, never a frozen census; one door in `wat/fix.wat`; the declaration door (2a1); C (2a1b:
`type-of` answers every name `is-type?` admits).

## Why

Every eval-based codemod pays `eval-with-defs!` (two full startups, ~460 ms) per type QUESTION
(finding 9); a composition run took hours. After 2a1 and 2a1b nothing requires it:
- a program's own types come from `:wat::runtime::declared-types` on its forms (12–17 ms, one call);
- a stdlib name comes from `:wat::runtime::is-type?` then `:wat::runtime::type-of` in the codemod's own
  world — total since 2a1b.

## D1 — one door in `wat/fix.wat`

`:wat::fix::enum-fields` (the name is yours): a file's top-level forms plus the candidate enum paths its
scan found → `HashMap String (Vector String)`, variant path → field names. That is the map `fill-enum`
builds today (`match-arm-to-bracket-map-pattern.wat:811`, `positional-ctor-to-map.wat:264`, keyed
`Enum::Leaf`; positional-ctor also indexes by leaf and short name — keep those indexes).
- **The program's types:** `declared-types` on the file's TOP-LEVEL forms — the same scope today's
  `file-decls` has (`match-arm…:83`, `positional-ctor…:72`: top-level children only). Keep rows of kind
  Enum whose name is not a variant singleton (`variant-parent-of` answers `Some` for a singleton `E.V`).
- **A refusal excludes one form, never the file.** On `Refused`, drop `Refused.form` (exact since 2a1
  R3), print `[<tool>] UNREGISTERABLE <path> <cause tag>`, and ask again until `Ok`; each round drops
  one form, so it ends. Today's ladder salvages partial files the same way (finding 3).
  **Measured** (`bootstrap/era/probe-R/door-exclude.wat` on the 32 files the door refuses in the
  step-24 input, `deaeeb131`): all 32 reach `Ok`, none stuck, 39 forms excluded, 981 ms in total. The 6
  negative `defservice` fixtures recover 15–48 types each (an all-or-nothing door would lose them all);
  `surface-field-dispatch.wat` recovers 2; the 15 era-syntax fixtures exclude their one bad form.
  ⚠ **A stdlib file** (`wat/core.wat`, `grep.wat`, `runtime-meta.wat`, …) excludes the declarations
  that differ from HEAD's snapshot (`DuplicateMacro`/`DuplicateType`) and so answers those names from
  the running world — HEAD's fields, not the era file's. Today's tools do the same (`eval-with-defs!`
  runs on HEAD's world). Report every such exclusion; never silently answer an era record with HEAD's
  fields.
- **Stdlib:** each candidate path the program does not answer → `is-type?` then `type-of`, kept if
  Enum. No hand list of stdlib enums: positional-ctor's `seed-paths` (`:318`) and `stdlib-fmap` go.
  positional-ctor's corpus-invariant pass (`:975-1030`, `:wat::`/`:rust::` paths resolved once) may stay
  as the stdlib cache.
- **Nested `(:wat::core::forms …)` programs:** today's `file-decls` does not see them and the door does
  not either. A site whose type is declared only inside a nested program is REPORTED
  (`[<tool>] UNRESOLVED nested-program <path>:<line>`); resolving it is its own stone.

## D2 — the three codemods use it

- **match-arm** (`fmap-for-src :849`) and **positional-ctor** (`fmap-for-src :533`, `fill-paths :339`):
  the door replaces `try-type-of`. Delete what becomes dead: `try-type-of` (`match-arm :162`,
  `positional-ctor :166` with its fallback chain `:181-187`), `resolve-fields`/`resolve-fields-at`
  (`match-arm :177`, `:194`), `file-decls` and both `decl-head?` hand lists. match-arm's `alias-enum`
  (`:123`) stays (one ruling's table shared with `bare-variant-to-qualified`; its own stone).
- **positional-ctor's UNRESOLVED report** fires when the head's parent is not answered, or is an enum and
  the leaf is not among its variants — never by character case. `pascal-leaf?` (`:198`, used `:848`) is
  deleted (finding 4; the ruling: case carries no meaning).
- **variant-separator** retires the census (`variant-separator-to-dot.wat:62`, the 382-pair file). A
  keyword `:P::V` flips to `:P.V` iff the door (program ∪ stdlib) says `:P` is an enum declaring `V`. A
  `::` keyword whose `:P` is a declared enum and whose `V` is not one of its variants is REPORTED —
  main refuses it loudly (finding 7). Keep `rename-keyword-exact` (defenum declaration slots are already
  safe in it) and a prefilter.
- **Fixtures:** each changed codemod keeps its `wat-scripts/fixes/replay/<stem>/` fixture passing
  (`tests/cli/every_recorded_migration_replays.rs`: byte-exact, idempotent, non-vacuous, provenance) and
  gains cases for the new behaviour, each cited to its header spec line (the oracle is never the tool):
  a `holon::defrecord` constructor, a kwargs `defn`'s `::Kwargs`, an UNREGISTERABLE form, a non-variant
  `::` report, a nested-program report.

## D3 — step 14 reports a splice

`mandatory-typed-quasiquote-residual.wat`: an `unquote-splicing` in a type slot is REPORTED
(`[mandatory-typed-quasiquote-residual] SPLICE <path>:<line>`), never wrapped. `~@items` is one argument
that expands to N forms (finding 6). `unquote-wrapped?` (`:81`) keeps accepting `~x`.

## D4 — `convert.sh`: the corpus as context, one run per commit

- `scripts/replay/convert.sh <rev> <out-dir> <path>…` writes each `<path>` at `<rev>`, converted, to
  `<out-dir>/<path>`. Each codemod runs ONCE over that set's in-scope paths.
- `one-param-spec`'s CONTEXT is every `.wat` at `<rev>` outside `wat-scripts/fixes/`, minus files today's
  reader cannot lex, each REPORTED; its TARGETS are the given paths (today both are the one file).
- The replay recipe (`BRIEF-1-pilot-first-ten-commits.md` "One step") calls it twice per commit: the C^
  set and the C set. Update that recipe's text to the new form.
- Deterministic (two runs, byte-identical) and batch-independent (a file converted alone equals it
  converted in its commit's set). Prove both on pilot commits #1, #7, #9.

## The bar — derived, never "0 files differ"; the orchestrator re-runs `bootstrap/era/probe-S/run5.sh`

- **match-arm vs `probe-L/wL`** (same input): 0 files LOSING an arm. Every GAINING file is explained by
  a type the door reports that the ladder could not see.
- **positional-ctor vs `probe-L/pT`** (same input): 0 files LOSING a constructor; gains explained; every
  UNRESOLVED line names its reason.
- **variant-separator vs the census tool** (same input): 0 files LOSING a flip the census made; every
  gained flip's enum is declared in the file or the stdlib; the report lines listed.
- **The chain vs main:** identical ≥ 1,314 (run4); every file newly differing from main is explained.
- **Speed:** each tool is ONE process over all 1,489 files; report its wall seconds. `convert.sh`:
  per-commit wall for pilot #1, #7, #9 (the pilot's per file: 29 s probe, 51 s dense).
- Floor + clippy.

## STOP triggers

- **STOP-1:** a site the predecessor converted is lost and no undeclared type explains it. Report the
  file and site verbatim.
- **STOP-2:** the floor is red. Paste the whole block verbatim, and do not re-run.

## Tier

Commit on green (floor + clippy 0). **Do not push.** Yield with `SCORE-2a2.md`: a row per D-item, per
fixture case, per bar line.
