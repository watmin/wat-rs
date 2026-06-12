# BRIEF — Stone 259.S2c-ii-b: the host-type `spawn-program'` defclause (THE KEYSTONE)

Full design: `docs/arc/2026/06/259-forced-hand/DESIGN-STONE-259.S2c-ii.md`. The hard
parts are already solved (S2c-ii.0 = class_fqdn dispatch; S2c-ii-a = apply-loop purged →
one thread clause, no overlap). This is now a mechanical sweep.

**The work, in one paragraph.** Make `spawn-program'` a wat `defclause` dispatching on the
host record type — 2-arg `(host prog)` — retiring the 3-arg Rust intrinsic and migrating
every remaining caller's signature. The committed probe
`s2cii_b_two_arg_host_dispatch` flips RED→GREEN.

**The defclause (add to `wat/spawn.wat`, after the host constructors):**
```clojure
(:wat::core::defclause :wat::kernel::spawn-program'
  ([host <- :wat::spawn::ThreadOpts
    prog <- [:wat::kernel::Peer'<S,R> :-> :wat::core::nil]] -> :wat::kernel::Thread'<R,S>
    (:wat::kernel::spawn-thread' prog))
  ([host <- :wat::spawn::ProcessOpts
    prog <- :wat::core::Vector<wat::WatAST>] -> :wat::kernel::Process'<I,O>
    (:wat::kernel::spawn-process' prog)))
```
(Mirror `wat/core.wat:58`, the `:wat::core::+` defclause. The clause bodies call the
S2c-i intrinsics `spawn-thread'`/`spawn-process'`. Verify the process clause's `prog`
type matches what `(:wat::core::forms …)` produces — `infer_process_prog_type` accepts
it as `Vector<wat::WatAST>`; if the literal type differs, use the exact type the forms
block infers to. STOP if it can't be expressed as a clause param — report it.)

**Retire the Rust intrinsic:**
- `src/runtime.rs:4509` — DELETE the `":wat::kernel::spawn-program'" =>` eval dispatch arm.
- `src/check.rs:4797` — DELETE the `":wat::kernel::spawn-program'" =>` check dispatch arm.
- `src/kernel/spawn.rs` — DELETE `eval_kernel_spawn_program_prime` (the 3-arg monolith).
- `src/check.rs` — DELETE `infer_spawn_program_prime`.
- KEEP `spawn_thread_peer` / `spawn_process_peer` / `infer_thread_prog_type` /
  `infer_process_prog_type` (used by `spawn-thread'`/`spawn-process'`).
- Remove `spawn-program'` from any builtin-name registry it was registered in (it's a
  wat defclause now, not a Rust intrinsic) — grep where you added `spawn-thread'` in
  S2c-i and check the `spawn-program'` registration there.

**Migrate the remaining callers (signature only — the progs are ALREADY self-peer/forms):**
`grep -rn "spawn-program' :thread\|spawn-program' :process"` across `tests/` + `src/`.
For each: `(:wat::kernel::spawn-program' :thread (:wat::program::Env …) <prog>)` →
`(:wat::kernel::spawn-program' (:wat::spawn::thread) <prog>)`; `:process {} <forms>` →
`(:wat::spawn::process) <forms>`. The `env`/`{}` arg DROPS. Keep `<prog>`/`<forms>`
verbatim. (~13 wat-source sites across the arc-214/259 probes + kernel tests + any
`src/` doc-strings — update doc-strings too.)

**STOP triggers:**
- **STOP-1:** if the process clause's `prog` type cannot be expressed as a defclause param
  (the forms block's type), STOP and report the exact type the checker assigns.
- **STOP-2:** if a caller's prog is NOT already a self-peer/forms prog (an apply-loop `[I]->O`
  survived S2c-ii-a), STOP and report it — do not rewrite progs here (S2c-ii-a did that).
- **STOP-3:** if deleting the intrinsic breaks the `spawn-thread'`/`spawn-process'` primitives
  or the peer verbs, STOP — only the `spawn-program'` *monolith* dies; the primitives stay.

**Done = green:**
- `cargo test --release -p wat --test nursery probe_arc259_s2cii_b` → passes (42).
- `cargo test --release -p wat --test nursery probe_arc259_s2a probe_arc259_s2cii_a probe_arc259_s2ci probe_arc214_stone46aii probe_arc214_stone46i probe_arc214_stone46b` → all green (migrated).
- `cargo test --release -p wat --lib kernel` → green.
- The process kernel tests (`tests/kernel/spawn_program_prime_process.rs`, `peer_verb_round_trip_process.rs`, `peer_select_prime_process.rs`, `probe_arc214_beta_forms_server.rs`, `probe_arc214_alpha_crash_autoraise.rs`) — migrate the `:process` sig + run them (they may be `#[ignore]`'d / process-tier; report status).
- `cargo build --release` clean.
- `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known
  pre-existing reds (arc-255 reflection ×2 + undefined-builtin ×2).
