# DESIGN-STONE C0b.3b-e — process-tier `user.program` injection (env-fn as a source string)

> The process half of #238's parity, on the LIVE `spawn-program'` path (no fork-grave). `ProcessOpts`
> carries an `env-fn` — a wat SOURCE STRING — the spawned child evals in its own frozen world to
> produce `user.program`, then hands to the 3b-d seam (`invoke_user_main_with_program`). A string
> crosses the clone3 fork trivially; the child resolves everything against its own loaded forms.
> Co-design (builder, 2026-06-13): the env-fn is a source string (not a pre-resolved FQDN) — it
> unifies named call / bare anon fn / direct ctor expr, is CLI-friendly (3b-f reuses it), and
> trivial to test. (Syntax is current `:wat::…`; clojure surface arrives with arc-251, unchanged
> mechanism.)

## The contract decision (pinned)

`ProcessOpts` gains `env-fn <- :wat::core::String`. The child evals it and **dispatches on the
result**: a 0-arg fn → apply it (→ `:wat::Record`); a `:wat::Record` → use directly; anything else
→ a clean child startup death. The produced `:wat::Record` becomes `user.program` via
`invoke_user_main_with_program`. Default (`(process)`): `env-fn = "(:wat::program::EmptyEnv)"` (evals
to EmptyEnv — no minted fn, no sentinel, no Option).

Why a string + result-dispatch (builder's call, four-questioned): `eval_in_frozen` is the eval seam
(freeze.rs:1322; raw eval, no re-check). A **named** fn referenced in the string (`"(:my::make-env)"`)
is still type-checked — it's in the child's forms, checked at startup. An **inline/anon** fn or expr
is runtime-validated only. Honest spectrum: named = compile-safe, anon = flexible; either way the
result must be a `:wat::Record` (validated at install). The CONSUMER reading `user.program` stays
fully typed regardless.

## The algorithm (the name threads to the child seam)

The env-fn string flows: `ProcessOpts` → `spawn-program'` defclause extracts it → `spawn-process'`
(tier primitive) → captured in the clone3 child closure (a `String` crosses COW) →
`run_forms_as_server_child(forms, inherit_config, env_fn)` → `run_user_main_in_child(world, …, env_fn)`.

At the seam (`verbs.rs:274`, where `invoke_user_main(world, Vec::new())` is called today):
```rust
match env_fn {
    None => invoke_user_main(world, Vec::new()),          // CLI / other callers — unchanged
    Some(src) => {
        let ast = parse_one!(&src)?;                       // child-death on parse error
        let v = eval_in_frozen(&ast, world, &Environment::new())?.value_owned();
        let user_program = match v {
            Value::wat__core__fn(f) => apply_function(f, vec![], world.symbols(), span)?.value_owned(),
            r @ (Value::wat__Record { .. } | Value::wat__holon__Record { .. }) => r,
            other => /* clean child death: env-fn must produce a :wat::Record */,
        };
        invoke_user_main_with_program(world, Vec::new(), user_program)
    }
}
```
(Errors in the `Some` arm exit the child cleanly via the existing `emit_structured_exit` /
startup-error path, like a startup failure — they must not panic the seam.)

`spawn-process'` ALWAYS passes `Some(env_fn)` (default `"(:wat::program::EmptyEnv)"` → EmptyEnv,
the current behavior). The two NON-spawn `run_user_main_in_child` callers (verbs.rs:349, :563) pass
`None` → `invoke_user_main` unchanged.

## Names (builder-originated; intueri may refine)

`env-fn` (the `ProcessOpts` field), `(process/env "<src>")` (the constructor). Both are the builder's
own terms (`--env` / env-fn). `(process)` keeps its default.

## The type-loading invariant (substrate doctrine — builder, hard requirement 2026-06-13)

`user.program` is the **rich** config escape hatch — a `:wat::Record` (plain, holon, or any
bespoke subtype like `app::Env`), NOT string→string. Three rules govern how it moves:

1. **A rich value crosses the wire as EDN.** That is the universal, structural form.
2. **Only a consumer that has the type's CODE loaded can interface with it.** To read a record's
   fields / dispatch on it, the universe must have the `:wat::Record::def` (and its accessors)
   loaded. No code → no use. An **intermediary** (a relay) just passes the opaque EDN; it needs
   nothing.
3. **Therefore `user.program` is produced AND consumed in the same universe.** The env-fn runs
   IN the spawned universe (where the forms/types are loaded), post-load / pre-`user/main`; that
   same universe's `user/main` reads it. Nothing crosses a type boundary in normal use.

Consequences that bind this design (and future remote/cross-process work):
- The gate accepting `user.program` is a **value-variant match** (`Value::wat__Record { .. }` /
  `wat__holon__Record`), NEVER `class_fqdn == ":wat::Record"` — so every subtype passes. (Past
  bug class: exact `== ":wat::Record"` silently rejecting subclasses.)
- The honest TEST observes the env-fn **in-process**, against a world that HAS the type
  (`resolve_env_program` unit test). Shipping a record to a type-less consumer rightly FAILS
  (arc-085 "no registry") — that is the invariant working, not a gap to engineer around.

## Files touched

`wat/spawn.wat` (ProcessOpts `env-fn` field + `(process)` default + `(process/env)` ctor + defclause
extract+pass), `src/kernel/spawn.rs` (`eval_kernel_spawn_process_prime` + `spawn_process_peer` thread
the string), `src/process/verbs.rs` (`run_forms_as_server_child` + `run_user_main_in_child` gain the
arg + the eval-dispatch; the 2 other `run_user_main_in_child` callers pass `None`), `src/check.rs`
(`infer_spawn_process_prime` types the new String arg — arity bump). The probe is on disk.

## Out of scope (rejected — NOT deferred)

- **wat-cli `--env`** (the root flag) = 3b-f, gated on arc-213 (CLI → `spawn-program'`); this stone
  is the substrate it reuses.
- **Reading a `user.program` SUBTYPE field** (downcast off the `:wat::Record` slot) — separate
  consumer concern; the probe inspects the returned record's class_fqdn, not a subtype-field read.
- **Type-checking the inline env-fn string** — `eval_in_frozen` is raw eval by design; named fns
  carry the checked path.

## The gate (probe — RED at HEAD on exactly the gap)

`tests/probe_arc209_c0b3be_process_env_fn.rs` (verified RED — `UnknownFunction process/env`):
1. `env_fn_as_bare_fn` — `(process/env "(:wat::core::fn [] -> :wat::Record (:child::Cfg 99))")` →
   evals to a fn → child applies → `child::Cfg`. (the fn-dispatch branch)
2. `env_fn_as_call_expr` — `(process/env "(:child::make-env)")` → evals to a `:wat::Record` → used.
   (the record-dispatch branch)
Both: the child's `:user::main` ships `user.program` over its self-peer; the owner recv's the Value
and asserts class_fqdn `child::Cfg`.

Regression: c0b3aii / c0b3bb / c0b3bc green (the `(process)` default still spawns + serves); lib
915/36 (zero new); nursery 895/4; full surface compiles.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the `(process)` default (`"(:wat::program::EmptyEnv)"`) breaks an existing process
   spawn (c0b3aii/c0b3bb/c0b3bc red) — the default must produce EmptyEnv exactly as today.
2. **STOP-2:** a `:wat::Record` can't be sent over a `Peer'<:wat::Record, _>` self-peer (the probe's
   observable) — report; if EDN round-trip of a user record fails, the probe needs a different
   observable, not the feature changed.
3. **STOP-3:** an env-fn eval error panics the child seam instead of a clean structured death —
   route it through the existing `emit_structured_exit` path.

## Deadlock contract

A synchronous eval at child startup (one parse + one eval + maybe one 0-arg apply), before the
existing `invoke_user_main`. No new blocking, no lifecycle change.
