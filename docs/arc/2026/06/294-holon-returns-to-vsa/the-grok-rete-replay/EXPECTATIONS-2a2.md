# EXPECTATIONS 2a2 — written before the strike (with `BRIEF-2a2-the-codemods-ask-the-door.md`)

The orchestrator re-runs every row on the SCORE commit, uncontended.

| # | what | how the orchestrator checks | expected |
|---|---|---|---|
| E1 | match-arm loses nothing | `bootstrap/era/probe-S/run5.sh` R-MA | `MA: files LOSING=0`; each GAINING file explained in the SCORE |
| E2 | positional-ctor loses nothing | run5 R-PC | `PC: files LOSING=0`; UNRESOLVED lines each carry a reason |
| E3 | variant-separator loses nothing | run5 R-VS | `VS: files LOSING=0`; gained flips each on a declared enum; report lines listed |
| E4 | the chain vs main | run5 R-CH + classify | identical ≥ 1314; every newly-differing file explained |
| E5 | no eval-with-defs! for types | `git grep -n 'eval-with-defs!' wat-scripts/fixes/{match-arm-to-bracket-map-pattern,positional-ctor-to-map,variant-separator-to-dot}.wat` | none |
| E6 | no hand lists left | read: `decl-head?`, `seed-paths`, `pascal-leaf?`, the census read | all gone; `alias-enum` stays (routed) |
| E7 | refusal excludes one form | a fixture case + the UNREGISTERABLE lines in run5's logs | the rest of the file's types still resolve |
| E8 | splice is reported | the quasiquote-residual fixture's new case | `SPLICE` line; no `:- [~@…]` |
| E9 | fixtures | `cargo nextest run --release -E 'test(every_recorded_migration)'` | all shards + positional-ctor pass; new cases cite a header spec line |
| E10 | speed | run5's per-tool wall seconds | minutes, unsharded, for the whole corpus |
| E11 | convert.sh | pilot #1, #7, #9: two runs + alone-vs-batch | byte-identical both ways; per-commit wall reported |
| E12 | walls | floor + clippy, uncontended | green; clippy 0 |

**Runtime prediction:** 2–4 h for grok (three codemods, one door, a script, fixtures).

**Trap doors:**
- the door returns variant singletons (`E.V`) as Enum rows — keep only non-singletons, or every variant
  gets an extra map entry;
- `Refused.form` falls back to the first form when no top-level form covers the error span: the
  exclusion loop must still terminate and report what it dropped;
- match-arm and positional-ctor key their maps with `::` (steps 23/26 run before the dot flip); the door's
  TypeInfo variant names are leaves — build keys the way `fill-enum` does today;
- `one-param-spec` with the whole corpus as context reads ~1900 files per commit; measure that cost.
