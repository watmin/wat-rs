# BRIEF — Stone C0b.3b-c: the post-spawn hook (owner-side, per-env record)

**Executor:** Shadowdancer (sonnet). **Anchor:** `/home/watmin/work/holon/wat-rs/` (verify `pwd`;
operate ONLY here; `git -C /home/watmin/work/holon/wat-rs`). Design (read it fully):
`DESIGN-STONE-C0b.3b-c-post-spawn-hook.md`. The three RED probes are on disk + verified RED at
HEAD (`tests/probe_arc209_c0b3bc_post_spawn.rs`): `process_…`/`thread_…` fail with
`UnknownFunction(process/post-spawn | thread/post-spawn)`; `accessor_typechecks_at_parse_time`
fails because the bogus accessor slips through check at HEAD (deferred to eval). Do NOT commit —
the Inquisitor weighs.

## The work in one paragraph

Add a `post-spawn-fn` to every host opts record — a fn the OWNER supplies that runs owner-side,
after the peer is spawned, before `spawn-program'` returns, for effects. It receives a per-env
launch record: `ThreadLaunch` (empty) on the thread tier, `ProcessLaunch [pid]` on the process
tier. The hook follows the `init-fn` path exactly — the `spawn-program'` wat defclause extracts it
from the opts and passes it to the tier primitive (`spawn-thread'`/`spawn-process'`), which
applies it owner-side in Rust (`spawn_thread_peer` after thread spawn with an empty record;
`spawn_process_peer` in the parent branch with `ProcessLaunch{pidfd.pid()}`). Records are built
Rust-side via `format!`→`parse_one!`→`eval` (the `spawn.rs:448` precedent) + `apply_function`.

## Read in order (the rooms)

1. `wat/spawn.wat` (whole file, ~99 lines) — the opts records, the `(thread)`/`(thread/init)`/
   `(process)` ctors, and the `spawn-program'` defclause (`:90-99`). The `init-fn` field +
   `ThreadOpts/init-fn` accessor at `:33`/`:95` is the EXACT pattern to mirror.
2. `src/kernel/spawn.rs:258-364` — `eval_kernel_spawn_thread_prime` (2-arg `prog init-fn`) +
   `eval_kernel_spawn_process_prime` (1-arg `forms`). These gain the `post-spawn-fn` arg and pass
   it down.
3. `src/kernel/spawn.rs:378-458` — `spawn_thread_peer`. The init-fn is applied CHILD-side at
   `:425` (`apply_function(init_fn, vec![], ...)`); the peer-env record is built via
   format→parse→eval at `:448-457`. Your post-spawn hook applies OWNER-side: in the PARENT flow
   after the `std::thread::Builder::spawn` at `:408` returns the handle, before the peer value is
   returned.
4. `src/kernel/spawn.rs:539-668` — `spawn_process_peer`. The PARENT branch is `:642-667` (after
   the fork; `pidfd` in hand). `Pidfd::pid()` is `clone.rs:217`. Apply the hook here with
   `ProcessLaunch{pid}`, before returning the wrapped peer.
5. `src/runtime.rs:4521/4524` — eval head-match for `spawn-thread'`/`spawn-process'`.
6. `src/check.rs:4800/4808` (dispatch) + `infer_spawn_thread_prime` (`:10572`) /
   `infer_spawn_process_prime` (`:10622`) — these validate the primitives' args; they gain the
   `post-spawn-fn` arg typed `Fn(<EnvLaunch>) -> nil`.
7. `src/runtime.rs:17349` — `pub fn apply_function(...)` signature (you call it to fire the hook).

## Implementation sketch

### (1) `wat/spawn.wat` — records, opts fields, ctors, defclause

```clojure
;; per-env launch records (what each env hands the post-spawn hook)
(:wat::Record::def :wat::spawn::ThreadLaunch [])
(:wat::Record::def :wat::spawn::ProcessLaunch [pid <- :wat::core::i64])

;; opts records gain post-spawn-fn (mirror init-fn). Confirm the 1-arg Fn type syntax against
;; init-fn's `:wat::core::Fn()->wat::Record` — a 1-arg fn is `:wat::core::Fn(<Arg>)->wat::core::nil`.
(:wat::Record::def :wat::spawn::ThreadOpts
  [init-fn       <- :wat::core::Fn()->wat::Record
   post-spawn-fn <- :wat::core::Fn(:wat::spawn::ThreadLaunch)->wat::core::nil])
(:wat::Record::def :wat::spawn::ProcessOpts
  [post-spawn-fn <- :wat::core::Fn(:wat::spawn::ProcessLaunch)->wat::core::nil])

;; ctors — every ctor sets BOTH fields (default the one it doesn't take). No-op defaults:
(:wat::core::defn :wat::spawn::thread [] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts
    (:wat::core::fn [] -> :wat::Record (:wat::program::EmptyEnv))
    (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)))
(:wat::core::defn :wat::spawn::thread/init [f <- :wat::core::Fn()->wat::Record] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts f (:wat::core::fn [_l <- :wat::spawn::ThreadLaunch] -> :wat::core::nil nil)))
(:wat::core::defn :wat::spawn::thread/post-spawn [g <- :wat::core::Fn(:wat::spawn::ThreadLaunch)->wat::core::nil] -> :wat::spawn::ThreadOpts
  (:wat::spawn::ThreadOpts (:wat::core::fn [] -> :wat::Record (:wat::program::EmptyEnv)) g))
(:wat::core::defn :wat::spawn::process [] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts (:wat::core::fn [_l <- :wat::spawn::ProcessLaunch] -> :wat::core::nil nil)))
(:wat::core::defn :wat::spawn::process/post-spawn [f <- :wat::core::Fn(:wat::spawn::ProcessLaunch)->wat::core::nil] -> :wat::spawn::ProcessOpts
  (:wat::spawn::ProcessOpts f))

;; defclause: extract post-spawn-fn + pass to the tier primitive (mirror the init-fn extraction)
;;   thread:  (:wat::kernel::spawn-thread'  prog (ThreadOpts/init-fn host) (ThreadOpts/post-spawn-fn host))
;;   process: (:wat::kernel::spawn-process' prog (ProcessOpts/post-spawn-fn host))
```

### (2) `eval_kernel_spawn_thread_prime` / `eval_kernel_spawn_process_prime` — accept + thread the arg

Add an arg: thread becomes `(spawn-thread' prog init-fn post-spawn-fn)` (3-arg); process becomes
`(spawn-process' forms post-spawn-fn)` (2-arg). Eval the new arg to an `Arc<Function>` (mirror the
`init_fn` extraction at `spawn.rs:307`) and pass it to `spawn_thread_peer`/`spawn_process_peer`.

### (3) `spawn_thread_peer` — apply the hook owner-side

`spawn_thread_peer(program_fn, init_fn, post_spawn_fn, sym, list_span)`. After the thread is
spawned (the `join_handle` at `:408` exists) and the parent-side peer value is built, BEFORE
returning it:
```rust
// Build the empty ThreadLaunch + fire the owner-side hook (mirror the :448 record-build pattern).
let launch_ast = crate::parse_one!("(:wat::spawn::ThreadLaunch)").expect("ThreadLaunch ctor parses");
let launch = crate::runtime::eval(&launch_ast, &Environment::new(), sym)
    .expect("ThreadLaunch evals").value_owned();
apply_function(post_spawn_fn, vec![launch], sym, list_span.clone())?;  // owner-side, effects
```

### (4) `spawn_process_peer` — apply the hook owner-side with the pid

`spawn_process_peer(forms, post_spawn_fn, sym, list_span)`. In the PARENT branch (`:642-667`),
after `peer`/`bundle` is built and `pidfd.pid()` is available, BEFORE the final `Ok(make_rust_opaque(...))`:
```rust
let child_pid = peer.pidfd.pid() as i64;   // Pidfd::pid(), clone.rs:217
let launch_src = format!("(:wat::spawn::ProcessLaunch {child_pid})");
let launch_ast = crate::parse_one!(&launch_src).expect("ProcessLaunch ctor parses");
let launch = crate::runtime::eval(&launch_ast, &Environment::new(), sym)
    .expect("ProcessLaunch evals").value_owned();
apply_function(post_spawn_fn, vec![launch], sym, list_span.clone())?;
```
(`peer` is moved into `bundle` at `:657` — read `pidfd.pid()` BEFORE the move, or read it off the
bundle. Order so the pid is captured before `peer` is consumed.)

### (5) `infer_spawn_thread_prime` / `infer_spawn_process_prime` — type the new arg

Accept the extra arg; unify it with `Fn(<EnvLaunch>) -> :wat::core::nil` (thread: `ThreadLaunch`,
process: `ProcessLaunch`). The result type (`Thread'<R,S>` / `Process'<I,O>`) is unchanged. The
per-env record types come from the `:wat::Record::def`s in spawn.wat. The accessor type-check
(`ProcessLaunch/pid` ok; `ProcessLaunch/bogus-field` rejected) is then automatic — it's the
checker's standard record-field discipline applied to the new records.

## Blast radius

`wat/spawn.wat` (2 records + 2 opts fields + 2 ctors + the defclause), `src/kernel/spawn.rs` (2
eval primitives thread the arg; `spawn_thread_peer` + `spawn_process_peer` apply owner-side),
`src/check.rs` (2 infer primitives type the new arg). NO `comms` change. NO change to the gate
(#236) or the connection surface.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** the default no-op hook breaks a bare `(thread)`/`(process)` spawn — any existing
   spawn/c0b probe goes red. STOP; the bare ctor's default must be a clean no-op.
2. **STOP-2:** the parse-time accessor check does NOT fire (`accessor_typechecks_at_parse_time`
   stays red after the build — the bogus field compiles). STOP; the payoff is the point — report
   the checker gap.
3. **STOP-3:** `pidfd.pid()` cannot be read owner-side before `peer` is moved into the bundle, or
   `apply_function` cannot be called from `spawn_process_peer` (no `sym`). STOP, report. (The
   init-fn precedent has `sym` + applies a fn; expected available.)

## The gate (report each exact `test result:` line; do NOT commit)

```
cargo test --release -p wat --test probe_arc209_c0b3bc_post_spawn -- --test-threads=1   # 3 passed
cargo test --release -p wat --test nursery -- --test-threads=1                          # 895 passed / 4 failed (baseline; ZERO new)
cargo test --release -p wat --lib spawn -- --test-threads=1                             # spawn lib tests green
cargo test --release -p wat --test probe_arc209_c0b3aii_process_service_loop -- --test-threads=1   # still GREEN
cargo test --release --workspace --no-run                                               # full surface compiles
```
Run `cargo test` PLAINLY (no setsid/timeout). The harness may show stale rust-analyzer
diagnostics mid-edit that contradict a clean `cargo build` — trust your own build.

## Prior comparable (copy the shape)

`BRIEF-STONE-C0b.3b-b.md` (the just-shipped sibling — same `src/kernel/spawn.rs` + `src/check.rs`
neighborhood, same probe-driven cycle) and the `init-fn` wiring in `spawn.wat` + `spawn.rs:425`
(the exact pattern this mirrors, owner-side instead of child-side).
