# BRIEF — Stone 6b-DEP: generic-method type-argument application (call site)

> Single-hop sonnet Shadowdancer. Do NOT spawn sub-agents. Work only in `~/work/holon/wat-rs`. Commit
> nothing; the orchestrator weighs the diff + re-runs the gate. Grounded against HEAD `6e206850`,
> branch `arc-170-gap-j-v5-deadlock-state`. Full design:
> `docs/arc/2026/06/272-…/DESIGN-STONE-6b-DEP-generic-method-type-application.md`.

## The work (one paragraph)

A generic protocol method (`(m<S,R> [self <- :P …] -> …)`) cannot be called with explicit type-args:
`(:P/m<i64,i64> recv …)` fails check with `UnknownCallee ":P/m<i64,i64>"`. Make it work — the method
call-head must (1) strip the `<T,T>` suffix to match the registered (bare) method name, and (2) BIND the
explicit type-args to the method's type-params so the call's args/return check under that substitution
(NOT inference — the target case has no value arg carrying the type-params). The gate is the committed
RED probe `tests/probe_arc232_generic_method_type_application.rs` (`#[ignore]`) going GREEN (returns 42),
with the `#[ignore]` removed.

## Diagnose first (one fact decides the shape)

The instantiation at `check.rs:5524-5540` currently maps each type-param to `fresh.fresh()` (inference).
**Determine whether explicit call-site type-args are bound anywhere today** (do generic FNs bind `<…>` or
just strip-and-infer? — inspect `canonical_callable_name` `runtime.rs:2987` usage + the fn-call check at
`check.rs:5692-5814`). The probe's `mk` has NO value arg carrying S/R, so inference cannot resolve them —
the explicit `<i64,i64>` MUST be parsed and bound. Build that binding (parse the suffix types via the
existing type-form parser, map `sig.type_params[i] → parsed[i]`, then `rename`). If explicit type-args
are already bound for fns, reuse that path.

## The seams (read in order, grounded)

1. `src/runtime.rs:5883-5889` — extend-type impl method-name parse: stores the name **without** stripping
   `<T>`. defprotocol (`runtime.rs:5736-5748`) strips via `split_name_and_type_params`. Make extend-type
   consistent — store the BARE method name as the `impl_clauses` key.
2. `src/runtime.rs:4942-5008` — runtime protocol-method dispatch: `method_name = &other[slash_pos+1..]`
   includes `<T,T>`; `impl_clauses.get(method_name)` (line ~5008) then misses. Strip `<T,T>` before the
   lookup (reuse `split_name_and_type_params` / `split_type_params` `runtime.rs:9983` / `canonical_callable_name`).
3. `src/check.rs:5491-5496` — checker protocol-method head: `method_name` includes `<T,T>`;
   `methods.iter().find(|s| s.name == method_name)` (5496) misses. Strip the suffix before `find`, and
   capture the explicit type-arg string for step 4.
4. `src/check.rs:5524-5540` — the instantiation: when explicit type-args were captured, build the mapping
   `sig.type_params[i] → parsed-type[i]` and `rename` with THAT (instead of `fresh.fresh()`); keep the
   fresh-var path when no explicit args are given (preserves existing inference behaviour).

Reuse helpers (named by the map): `split_name_and_type_params` (`runtime.rs:2999`), `split_type_params`
(`runtime.rs:9983`), `canonical_callable_name` (`runtime.rs:2987`), `rename` (`check.rs:13900`),
`instantiate` (`check.rs:13876`). The resolve pass already accepts `:P/m<T>` (`resolve/walk.rs:228-248`) —
no change there.

## STOP triggers (halt + report; ship nothing)

1. STOP if making the probe green requires statically type-checking the extend-type impl **body** (today
   `check.rs:3753-3759` only form-validates it). If `(listener' self :S :R)` in the body needs new
   body-level type-param resolution, that is a SEPARATE seam (DEP-iii) — report exactly what's needed and
   stop; do NOT bolt body-checking onto this strike.
2. STOP if the fix would change the monomorphic (non-generic) method path's behaviour — the empty-`type_params`
   branch must stay a no-op (existing methods unaffected).
3. STOP if binding explicit type-args requires a parser change beyond calling the existing type-form
   parser on the suffix — report it.

## Gate (orchestrator re-runs)

- `cargo test --release -p wat --test probe_arc232_generic_method_type_application -- --include-ignored --test-threads=1`
  → GREEN (returns 42), `#[ignore]` removed.
- `cargo test --release -p wat --lib -- --test-threads=1 | grep "test result"` → **929 passed / 36 failed**
  (zero new).
- `cargo test --release -p wat --test nursery -- --test-threads=1 | grep "test result"` → **893/4**
  baseline (the protocol-method + generic tests live here — confirm no regression).
- `cargo build --release -p wat` → clean.

Report: the diagnosis (does explicit-arg binding exist today? — file:line), the exact files+lines changed,
the four gate results from your OWN runs (paste them), and any STOP hit.
