# SCORE — Stone 249.5d: ArgSpec carries the Identifier

Graded against `EXPECTATIONS-STONE-249.5d.md`, every load-bearing row re-run
**independently by the orchestrator** (not the executor's report).

## Scorecard

| # | What | Result |
|---|---|---|
| 1 | Root contract holds | ✓ `pub fixed_params: Vec<(Identifier, TypeExpr)>` / `rest_param: Option<(Identifier, TypeExpr)>` (argspec/parse.rs:17,20) |
| 2 | Rest-param hygiene bug killed | ✓ `probe_argspec_rest_param_hygiene` → 1 passed (returns 10) — **orchestrator re-run** (RED→GREEN; the `UnboundSymbol("x")` is gone) |
| 3 | Prior hygiene contract holds | ✓ `probe_macro_hygiene_capture` → 2 passed (105 + 7) — orchestrator re-run |
| 4 | Re-walk class gone | ✓ grep `scoped_arg_names\|scoped_params_from_args_vec\|extract_scoped_params` over src/ → 0 hits |
| 5 | Inline rest-compensation gone | ✓ grep "Re-extract the rest-binder" → 0 hits |
| 6 | Library suite — no regressions | ✓ `cargo test --release --lib -p wat` → 907 passed / 0 failed / 1 ignored (= baseline) — orchestrator re-run |
| 7 | check.rs untouched | ✓ `git diff --stat src/check.rs` empty |
| 8 | No new public surface | ✓ diff is field-type + derivation changes only; no new `pub` type/fn |

## Trap-doors (all cleared)

- **The neutral-prefix fork** — verified at `function/infer.rs:46`:
  `let p = idents.iter().map(|id| id.as_str().to_owned()).collect();` — the check
  tier maps to **bare** (`as_str`), NOT `env_key`. `parse_fn_signature_prefix`
  returns `Vec<Identifier>` (neutral). The eval tier (`parse_fn_signature`) maps to
  scoped. The fork is resolved correctly at the callers, exactly as designed.
- **Partial move in `try_parse_variadic_def_fn_form`** — compiled clean; the
  field-by-field move (`spec.fixed_params.into_iter()` then `spec.rest_param`) holds.
- **A missed consumer** — none. Build was clean on the first cascade pass with no
  outside-room errors (STOP-2 never fired); Row 4's 0-hit grep + clean build proves
  the consumer set was exhaustive.

## Shape

11 files changed, **+38 / −168 (net −130 lines)**. The deletion-heavy shape is the
signature of a class eliminated, not a symptom patched: `scoped_arg_names` (−63 in
resolution.rs), `scoped_params_from_args_vec`, `extract_scoped_params`, and the
fn-path inline rest-compensation all removed; consumers derive bare/scoped inline
from the one source.

## Honest deltas

- **None material.** The rooms, derivations, and blast radius were fully accurate.
- One structural note (executor-surfaced, verified): deleting `scoped_arg_names`
  also retired `use crate::ast::WatAST` in `resolution.rs` — it was that fn's
  exclusive dependency. A clean consequence, not a deviation.

## Disposition

Behavior-preserving (scoped strings byte-identical via the same `env_key` over the
same identifiers) except the rest-param case, which goes latent-`UnboundSymbol` →
correct. The strip-and-re-walk CLASS is annihilated (`feedback_strip_and_rewalk_is_bandaid`).

**Open, named (NOT regressions):**
- **Stone 249.5e** — the pre-existing check-pass BIND-SCOPED/LOOKUP-BARE mismatch
  (`check.rs:3397`), affirmatively scoped out of this stone; DESIGN to draft next.
- **249.5 ward-close still owed** — R3 re-cast `src/scope/` → L1+L2=0 → apply the
  `src/scope/` vigilatum stamp + the HELD `macros/` stamp (double-blocked on the
  incoming vigilia update — cast the complete updated guard before stamping).
