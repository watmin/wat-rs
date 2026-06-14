# BRIEF — Stone C0b.3b-e: process-tier `user.program` injection (env-fn source string)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`;
operate ONLY here; `git -C /home/watmin/work/holon/wat-rs`). Design (read fully):
`DESIGN-STONE-C0b.3b-e-process-env-fn.md`. The RED probe is on disk + verified RED at HEAD
(`tests/probe_arc209_c0b3be_process_env_fn.rs` — `UnknownFunction process/env`). Do NOT commit —
the Inquisitor weighs.

## The work in one paragraph

`ProcessOpts` gains `env-fn` — a wat SOURCE STRING the spawned child evals in its own frozen world
to produce `user.program`. The child dispatches on the eval result: a 0-arg fn → apply it; a
`:wat::Record` → use directly; else clean child death. The result goes to `invoke_user_main_with_program`
(shipped in 3b-d). `(process)` defaults env-fn to `"(:wat::program::EmptyEnv)"`. The string threads
from `ProcessOpts` → `spawn-process'` → the clone3 child closure → `run_forms_as_server_child` →
`run_user_main_in_child`, exactly mirroring how 3b-c threaded `post-spawn-fn`.

## Read in order (the rooms)

1. `git show 4bf7e6ea -- wat/spawn.wat src/kernel/spawn.rs` — the 3b-c SHIPPED diff: how
   `post-spawn-fn` was added to `ProcessOpts`, defaulted in `(process)`, given a `process/post-spawn`
   ctor, extracted in the `spawn-program'` defclause, and threaded through `eval_kernel_spawn_process_prime`
   → `spawn_process_peer`. **Mirror this pattern exactly for `env-fn`.**
2. `wat/spawn.wat` — `ProcessOpts` record + `(process)` / `(process/post-spawn)` ctors + the
   `spawn-program'` defclause (process arm). Add `env-fn <- :wat::core::String`; `(process)` →
   `"(:wat::program::EmptyEnv)"`; new `(process/env s <- :wat::core::String)`; defclause extracts
   `env-fn` and passes it to `spawn-process'`.
3. `src/kernel/spawn.rs` — `eval_kernel_spawn_process_prime` (currently 2-arg: forms + post-spawn-fn
   after 3b-c) gains the `env-fn` String arg (→ 3-arg); `spawn_process_peer` gains an `env_fn: String`
   param, captures it in the clone3 child closure, passes it to `run_forms_as_server_child`.
4. `src/process/verbs.rs:362` — `run_forms_as_server_child(forms, inherit_config)` → add
   `env_fn: String`; pass `Some(env_fn)` to `run_user_main_in_child`.
5. `src/process/verbs.rs:256` — `run_user_main_in_child` gains `env_fn: Option<String>`; implement
   the eval-dispatch (sketch below) at the `invoke_user_main(world, Vec::new())` site (:274). The
   OTHER two callers (`:349` run_forked_child, `:563`) pass `None`.
6. `src/check.rs` — `infer_spawn_process_prime` (the spawn-process' infer; bumped to 2-arg by 3b-c)
   gains the String arg (→ 3-arg). Mirror the 3b-c infer change.
7. `src/freeze.rs:1316` — `eval_in_frozen` (the eval seam); `src/runtime.rs:17349` — `apply_function`.

## Implementation sketch (the seam — run_user_main_in_child)

```rust
fn run_user_main_in_child(world, stdin, stdout, stderr, env_fn: Option<String>) -> ! {
    // ... existing keepalive ...
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        match &env_fn {
            None => invoke_user_main(world, Vec::new()),
            Some(src) => {
                let ast = crate::parse_one!(src).map_err(/* -> RuntimeError */)?;
                let v = crate::freeze::eval_in_frozen(&ast, world, &crate::runtime::Environment::new())?
                    .value_owned();
                let user_program = match v {
                    crate::runtime::Value::wat__core__fn(f) =>
                        crate::runtime::apply_function(f, vec![], world.symbols(), Span::unknown())?
                            .value_owned(),
                    r @ (crate::runtime::Value::wat__Record { .. }
                        | crate::runtime::Value::wat__holon__Record { .. }) => r,
                    other => return Err(/* RuntimeError: env-fn must produce a :wat::Record; got … */),
                };
                crate::freeze::invoke_user_main_with_program(world, Vec::new(), user_program)
            }
        }
    }));
    // ... existing finish_forked_child(world, outcome) ...
}
```
Confirm the exact `Value` fn/record variant names against `src/kernel/spawn.rs` (the 3b-c
`eval_allow_prime` matches `Value::wat__core__fn`; record matching is in `eval_listener_prime`'s
host dispatch). The `Some`-arm errors must surface as a child death via the existing outcome/
`finish_forked_child` path — do NOT panic the seam (route the `?` into the catch_unwind's Result so
`finish_forked_child` reports it, like a startup error).

## Blast radius

`wat/spawn.wat`, `src/kernel/spawn.rs`, `src/process/verbs.rs`, `src/check.rs`. NO `comms` change.
NO change to 3b-d's `invoke_user_main_with_program` or to the CLI (`fork_program_from_source` is a
separate path, 3b-f/arc-213). The two non-spawn `run_user_main_in_child` callers gain a `None` arg
only.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the `(process)` default (`"(:wat::program::EmptyEnv)"`) breaks a bare process spawn —
   c0b3aii / c0b3bb / c0b3bc go red. The default must produce EmptyEnv exactly as today.
2. **STOP-2:** a `:wat::Record` can't be sent over a `Peer'<:wat::Record, _>` self-peer (the probe's
   observable) — STOP and report; do not change the feature to dodge it.
3. **STOP-3:** an env-fn eval error panics the child seam instead of a clean structured death —
   route it through the outcome/`finish_forked_child` path.

## The gate (report each exact `test result:` line; do NOT commit)

```
cargo test --release -p wat --test probe_arc209_c0b3be_process_env_fn -- --test-threads=1   # 2 passed
cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop -- --test-threads=1  # 1 passed
cargo test --release -p wat --test probe_arc209_c0b3bb_bounced -- --test-threads=1          # 2 passed
cargo test --release -p wat --test probe_arc209_c0b3bc_post_spawn -- --test-threads=1       # 3 passed
cargo test --release -p wat --lib -- --test-threads=1                                       # 915 passed / 36 failed (PRE-EXISTING; ZERO new)
cargo test --release -p wat --test nursery -- --test-threads=1                              # 895 passed / 4 failed (zero new)
cargo test --release --workspace --no-run                                                   # full surface compiles
```
NOTE: the lib unit suite has 36 PRE-EXISTING failures (`check::tests`/`runtime::tests`) — NOT yours;
confirm the count stays 36. Run `cargo test` PLAINLY (no setsid/timeout). The harness may show stale
rust-analyzer diagnostics that contradict a clean `cargo build` — trust your own build.

## Prior comparable (copy the shape)

`git show 4bf7e6ea` (3b-c — the EXACT ProcessOpts-field + defclause + spawn-process'-arity pattern,
one tier over) and `BRIEF-STONE-C0b.3b-c.md`.
