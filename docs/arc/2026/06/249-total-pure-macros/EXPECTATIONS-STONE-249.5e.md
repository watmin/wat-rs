# EXPECTATIONS — Stone 249.5e: the check pass keys locals by env_key

Written BEFORE the strike. The Score grades against THIS, re-run independently by
the orchestrator.

## Scorecard

| # | What | Command | Expected |
|---|---|---|---|
| 1 | The check imprecision is killed | `cargo test --release --test probe_check_scoped_param_resolution` | 2 passed (control + bug) |
| 2 | The lookup keys by env_key | `grep -n "env_key(ident)" src/check.rs` | hit at the `infer` Symbol arm (~:3397) |
| 3 | The bare-as_str binds are converted | `grep -n "as_str().to_owned()\|as_str().to_string()" src/check.rs` | no remaining hit at a `locals`/`*_locals`/`bindings`/`new_bindings.insert` key (only the A/C/diagnostic-walker sites, which are unchanged) |
| 4 | infer.rs flipped to env_key | `grep -n "env_key" src/function/infer.rs` | hit in `parse_fn_signature_for_check_diag` |
| 5 | Prior hygiene contracts hold | `cargo test --release --test probe_argspec_rest_param_hygiene --test probe_macro_hygiene_capture` | 1 + 2 = 3 passed |
| 6 | Library suite — no regressions | `cargo test --release --lib -p wat` | ≥ 907 passed, 0 failed |
| 7 | Bounded blast radius | `git diff --stat` | only `src/check.rs`, `src/function/infer.rs`, the probe |
| 8 | No new public surface / no `infer` signature change | diff review | `locals: &HashMap<String, TypeExpr>` unchanged; no new `pub` |

Rows 1, 5, 6 are load-bearing (orchestrator re-run). 2–4, 7–8 are
discipline/scope guards.

## Runtime prediction

10–16 min (Mode A). ~10 sites, uniform `as_str → env_key`, but three
(`:10564`, `:10639`, `:11430`) need the Identifier retained where it's currently
dropped — the only spots requiring thought. The `matches?` site may want a small
`logic_var_ident` helper.

## Trap-doors named

- **The CONTROL must keep passing.** If `handwritten_defclause_ret_mismatch_is_caught`
  ever goes red, the keying change broke resolution for unscoped (user) idents — a
  real regression, not a win. `env_key` of a bare ident must equal `as_str()`; if a
  site somehow double-keys, this fires. The Score confirms the control green.
- **Integration tier (not run here).** The ~190 pre-existing clojure-ification
  failures are unrelated (NoMatchingClause / UnresolvedReference / MalformedForm).
  This stone shouldn't move that count; the lib suite (907) is the regression gate.
  If a NEW lib failure appears, STOP-3 fired — surface it, don't absorb it.
- **A missed `(B)` bind.** If a `locals`-keyed bind keeps `as_str` after the strike,
  a macro-gen binder of that kind stays imprecise (no regression — same as today —
  but the class isn't fully closed). Row 3's grep is the completeness check.

## Out-of-scope confirmation (affirmative cuts)

The deadlock-diagnostic walker (`check.rs:9854-9889`) and the struct-field bind
(`:10734`) are NOT changed (Row 7 + Row 3 exclude them). The deadlock walker is
bare-internally-consistent (no mismatch); the struct field is a canonical name, not
a scoped identifier. If the strike finds either actually resolves a scoped param,
STOP-1 fired.
