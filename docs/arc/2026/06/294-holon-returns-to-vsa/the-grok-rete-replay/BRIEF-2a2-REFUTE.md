# REFUTE — 2a2: a nested program's own types are dropped; and five things that keep it from closing

Verified by the orchestrator on `9d887f753`, uncontended:
- floor 5431/5431 (`.floor/2026-09-13T21-38-25Z`), clippy 0;
- `bootstrap/era/probe-S/run5.sh` (each tool vs its predecessor on IDENTICAL input, 1489 files):

| tool | wall (one process) | files LOSING | files GAINING | UNRESOLVED |
|---|---|---|---|---|
| match-arm vs `probe-L/wL` | 146 s | **1** | 1 | 12 (ladder 15) |
| positional-ctor vs `probe-L/pT` | 28 s | 1 (not a loss, below) | 7 | 209 (today 7663) |
| variant-separator vs the census tool | 50 s | 0 | 62 (335 tokens) | 37 UNREGISTERABLE lines |

- the chain vs main: identical **1367** (run4 1314), differs 122; the main-binary classifier's
  CHAIN-FAILS **28** (run4 84); 54 files fixed vs run4, **1 new** (below);
- every positional-ctor gaining file is now IDENTICAL to main (acronym, macro-generated service, the
  four do/let splice files, `arming_is_internal_only_control`); positional-ctor's one "LOSING" file
  (`probe_arc278_macro_generates_service.wat`) is the same site converted deeper (`(…EchoResponse::Ok c)`
  → `{:n c}`), an artifact of run5's outer-text regex;
- `convert.sh` #9 (two files), run twice by the orchestrator: 14 s / 13 s, byte-identical.

## R8 — a nested program's own types are dropped (THE ONE REAL LOSS)

`wat-scripts/probes/arc-170/probe-m1-ann-erase.wat`: its child program (inside `(:wat::core::forms …)`)
declares `:probe::CMsg` and matches on it. Main has `[:probe::CMsg::Setup {:addr addr}` and
`[:probe::CMsg::Work {:s s}`; the chain now leaves `((:probe::CMsg::Setup addr) …)` positional. The ladder
(`wL`) converted them. It is the ONE file that newly differs from main.
- **This is the orchestrator's brief's error, not the strike's:** BRIEF-2a2 D1 said a nested program's
  sites are REPORTED, reasoning from today's top-level `file-decls`. The bar's predecessor resolved them,
  and `--check` never checks a child (finding 8) — so in the replay an unconverted child passes every gate.
- **The shape:** each `(:wat::core::forms …)` literal is its OWN program. Ask the door on ITS children
  and apply that map within ITS subtree only; the parent's map applies outside it. Never merge the two:
  a parent and a child may declare the same name with different fields (finding 3's two-`PoolMsg` case).
  A child whose declarations refuse is handled by the same exclusion loop.
- **Pins:** `probe-m1-ann-erase.wat` identical to main after the chain; a fixture where the parent and a
  child declare `:t::M` with DIFFERENT fields and each side's constructor converts with its own fields.

## R9 — positional-ctor still carries the pre-door filter and its two hand lists

`positional-ctor-to-map.wat:255-310`: `macro-decl-head?` (3 heads), `plain-type-head?` (7 heads),
`has-macro-decl?`, `plain-type-names`, feeding `keep-local-eps`. Its comment still says it exists because
"a local path is worth `try-type-of` only if THIS FILE could declare it". The door answers a program's
whole type set in one call, so the filter saves nothing and can only drop a path the door would answer
(e.g. an enum declared in a top-level `do`, which `plain-type-head?` does not see). Retire it; ask the
door's answer for every local candidate. run5's PC comparison cannot see this (pT had the same filter):
report how many sites change.

## R10 — the deadlock guard stopped looking inside `defmacro` bodies

`src/check.rs` `form_calls_declared_types` (the `stdlib_snapshot_is_once_and_stdlib_does_not_call_the_verb`
walk) now returns false inside `defn` AND `defmacro` bodies. A `defn` body runs later, after the
snapshot — skipping it is right. A `defmacro` body runs DURING the stdlib's own expansion (`build_env`),
the exact re-entry the guard exists for. Skip `defn` bodies only. It must go RED once: a temporary
stdlib `defmacro` whose body calls `:wat::runtime::declared-types` (reverted).

## R11 — the exclusion loop's fallback drops a form without naming it

`wat/fix.wat` `drop-form`: when no top-level form matches the refused one, it drops the FIRST form, and
`register-loop`'s `UNREGISTERABLE <path> <cause>` line names neither. Name the dropped form (its head,
its declared name, its line) on every UNREGISTERABLE line, and say when the fallback fired.

## R12 — variant-separator's header still describes the census

`variant-separator-to-dot.wat`: the SHAPE, IDEMPOTENCE and NO CASCADE sections still read "the pair list
… 382 (old,new) tuples, parsed from the census file below". Rewrite them to the door-based shape the code
implements (including why idempotence and order-independence still hold).

## R13 — the `""` sentinel key

The door returns `HashMap String (Vector String)` with the answered-enum list stored under key `""`, and
every consumer (`known-enum?`, `fill-stdlib`, …) must know that `""` is special. Return a record (e.g.
`:wat::fix::EnumFields {fields answered}`) so the two facts are two named fields.

## STOP triggers

- **STOP-1:** after R8/R9 a site the predecessor converted is lost and no undeclared type explains it.
  Report the file and site verbatim.
- **STOP-2:** the floor is red. Paste the whole block verbatim, and do not re-run.

## Tier

Commit on green (floor + clippy 0). **Do not push.** Yield with `SCORE-2a2-REFUTE.md`, one row per R,
plus your own run of `bootstrap/era/probe-S/run5.sh` (the orchestrator re-runs it).
