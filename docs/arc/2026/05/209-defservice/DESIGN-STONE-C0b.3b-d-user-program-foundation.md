# DESIGN-STONE C0b.3b-d (foundation) — `user.program` injection: the seam

> The one gap underneath all three injection tiers: `invoke_user_main` (freeze.rs) — the
> chokepoint the root main AND every process child run through — hardcodes the 7th field of
> `:wat::program::Env` to `(:wat::program::EmptyEnv)` (`freeze.rs:1095`) with no way to supply a
> `user.program`. This stone opens an ADDITIVE seam so a produced `user.program` Record can be
> injected. Consumers (root `wat-cli --env`, process `ProcessOpts` env-fn) build on it. Tracked
> #238. Co-design: `DESIGN-C0b.3b-provisioning-and-spawn-hooks.md`.

## Why

`user.program` is the structured-data injection slot (the `-J`/system-properties analog, a real
`:wat::Record`). Today only thread children can populate it (via the `init-fn` closure). The root
universe and process children can't — `invoke_user_main` always builds `EmptyEnv`. Every consumer
the builder wants (root `--env fqdn/fn`, process env-fn) needs ONE thing first: a way to hand
`invoke_user_main` a produced `user.program`. That seam is this stone.

## The contract decision (pinned) — ADDITIVE, not a signature change

`invoke_user_main(frozen, args)` stays EXACTLY as is (it has ~30 callers across src + tests; a
3-arg change would force `None` into every one — a sum-modeled-as-product smell). Instead add a
sibling:

```rust
/// Like `invoke_user_main`, but installs `user_program` as the `user.program` field of the
/// ambient `:wat::program::Env` (instead of the `EmptyEnv` default) before `:user::main` runs.
/// `user_program` must be a `:wat::Record` (any subtype). The root (wat-cli `--env`) and process
/// children supply the result of running their env-producing fn here.
pub fn invoke_user_main_with_program(
    frozen: &FrozenWorld,
    args: Vec<Value>,
    user_program: Value,
) -> Result<Value, RuntimeError>
```

`invoke_user_main` delegates: it builds the `EmptyEnv` default and calls the shared orchestrator
with it (or passes `None` and the orchestrator defaults). Either way the public surface is two
clean fns — no `Option` in the API, no ripple to existing callers.

## The algorithm (mirror spawn.rs:439–457 — the init-fn record-build pattern)

`invoke_user_main_orchestrated` (the private fn holding the `freeze.rs:1095` env build) takes the
injected `user.program` (as `Option<Value>` internally, or a `Value` with the default built by the
caller). At the 1095 build, instead of the literal `(:wat::program::EmptyEnv)` in the source
string, BIND the value as a local and reference it by name (exactly how `spawn_thread_peer` binds
`user-program` at `spawn.rs:441–447`):

```rust
let user_program_val = user_program.unwrap_or_else(|| {
    // the EmptyEnv default — eval the 0-arg ctor (current behavior)
    eval_in_frozen(&parse_one!("(:wat::program::EmptyEnv)"), frozen, &Environment::new())
        .expect("EmptyEnv constructs").value_owned()
});
let ctor_env = Environment::new().child()
    .bind_unknown_span("user-program", TrackedValue::from(user_program_val)).build();
let env_src = format!(
    "(:wat::program::Env (:wat::time::at-nanos {boot_nanos}) (:wat::time::now) \
     {pid} {tid} :wat::program::PeerKind::process {cpu_count} user-program)"  // <- the bound local
);
let program_env = eval_in_frozen(&env_ast, frozen, &ctor_env)?.value_owned();
```

Everything downstream (install_program_env, run main) is unchanged. The default path produces the
identical `EmptyEnv` env it does today.

## Files touched

`src/freeze.rs` ONLY — the new `invoke_user_main_with_program` (pub), `invoke_user_main`
delegates, `invoke_user_main_orchestrated` threads the injected value into the 1095 build. The
probe is on disk. NO `check.rs` (no wat surface — the program reads `user.program` via the
existing `(:wat::program::env)` + `:wat::program::Env/user.program` accessor). NO `spawn.wat`. NO
caller changes (the seam is additive).

## Out of scope (rejected — NOT deferred)

- **Root `--env fqdn/fn`** (wat-cli resolves+runs a named fn → the Record) = C0b.3b-f.
- **Process env-fn** (`ProcessOpts` carries the name; `run_user_main_in_child` resolves+runs it
  and calls `invoke_user_main_with_program`) = C0b.3b-e.
- **Reading a `user.program` SUBTYPE field** (downcast off the `:wat::Record`-typed slot) — a
  separate consumer concern; this stone proves the record FLOWS (the probe inspects the returned
  value's class_fqdn), not subtype-field access.

## The gate (probe — RED at HEAD on exactly the gap)

`tests/probe_arc209_c0b3bd_user_program_foundation.rs` (verified RED at HEAD — `E0432:
invoke_user_main_with_program` absent):
1. `injected_user_program_flows_to_main` — inject `(:user::MyEnv 42)`; `:user::main` returns
   `(:wat::program::Env/user.program (:wat::program::env))`; assert the returned record's
   class_fqdn is `user::MyEnv` (the injected record flowed, not EmptyEnv).
2. `default_user_program_is_empty_env` — the unchanged 2-arg `invoke_user_main` → main sees
   `EmptyEnv` (the default preserved; the regression guard).

Regression: the ~30 existing `invoke_user_main` callers compile UNCHANGED (additive seam); nursery
895/4 (zero new); full surface compiles.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** binding `user-program` as a local + referencing it in the env_src does not work
   (the spawn.rs:441 pattern fails here) — STOP, report (it is the proven thread-tier pattern).
2. **STOP-2:** the default path no longer produces `EmptyEnv` (the regression-guard fn goes red) —
   STOP; `invoke_user_main`'s behavior must be byte-identical to today.
3. **STOP-3:** any existing `invoke_user_main` caller needs editing — STOP; the seam is additive,
   the old signature is preserved.

## Deadlock contract

A synchronous eval at startup (the env build), unchanged in shape from today's 1095. No new
blocking, no lifecycle change.
