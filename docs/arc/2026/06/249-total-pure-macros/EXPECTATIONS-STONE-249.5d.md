# EXPECTATIONS — Stone 249.5d: ArgSpec carries the Identifier

Written BEFORE the strike. The Score (post-strike) grades against THIS, re-run
independently by the orchestrator — never the executor's say-so.

## Scorecard

| # | What | Command | Expected |
|---|---|---|---|
| 1 | The root contract holds | `grep -n "pub fixed_params" src/argspec/parse.rs` | `Vec<(Identifier, TypeExpr)>` |
| 2 | The rest-param hygiene bug is killed | `cargo test --release --test probe_argspec_rest_param_hygiene` | 1 passed (returns 10) |
| 3 | Prior hygiene contract still holds | `cargo test --release --test probe_macro_hygiene_capture` | 2 passed (105 + 7) |
| 4 | The re-walk class is gone | `grep -rn "scoped_arg_names\|scoped_params_from_args_vec\|extract_scoped_params" src/` | 0 hits |
| 5 | The inline rest-compensation is gone | `grep -n "Re-extract the rest-binder name" src/runtime.rs` | 0 hits |
| 6 | Library suite — no regressions | `cargo test --release --lib -p wat` | ≥ 907 passed, 0 failed (baseline 907/0/1) |
| 7 | check.rs untouched | `git diff --stat src/check.rs` | no change |
| 8 | No new public surface | `git diff src/` review | no new `pub` types/fns; only field-type + derivation changes |

Rows 1–6 are load-bearing (re-run by the orchestrator). 7–8 are scope-discipline
guards.

## Runtime prediction

8–14 min (Mode A). A representation change with a compiler-guided cascade across
~9 files; the derivations are pre-labeled in the BRIEF, so the executor applies
rather than designs. The `unzip` type-flow and the three `parse_fn_signature*`
fork points are the only spots needing care.

## Trap-doors named

- **The neutral-prefix fork.** If the executor makes `parse_fn_signature_prefix`
  itself `env_key` (scoped), the check-tier callers (`infer_fn`) get scoped names
  but look up bare → they'd silently mis-resolve (worse than today). The prefix
  MUST return identifiers; the env_key happens only in the eval-tier callers. The
  Score must confirm `infer.rs` maps to `as_str`, not `env_key`.
- **Partial move in `try_parse_variadic_def_fn_form`.** `spec.fixed_params.into_iter()`
  moves one field; `spec.rest_param?` reads another — legal (separate fields), but
  if the executor binds `spec` by reference somewhere first, the move fails. Sketch
  uses the field-by-field move; confirm it compiles as written.
- **A consumer the crawl missed.** Row 4's grep going to 0 AND a clean build is the
  proof the consumer set was exhaustive. If the build needs a fix outside the room
  list, STOP-2 fired — that's data (re-open the consumer map), not a silent patch.

## Out-of-scope confirmation (the affirmative cut)

The check-pass BIND-SCOPED/LOOKUP-BARE mismatch (DESIGN § Out of scope) is NOT
addressed here and MUST NOT be — Row 7 enforces it. It is tracked as the named
follow-on Stone 249.5e (DESIGN to draft at this stone's close), with its own
disconfirming probe (a macro-generated defclause body that type-checks imprecisely
today → precisely after).
