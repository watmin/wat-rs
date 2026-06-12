# BRIEF — Stone 259.S2c-i: the per-tier kernel primitives `spawn-thread'` / `spawn-process'`

**The work, in one paragraph.** Extract the two per-tier spawn primitives out of the
monolithic `spawn-program'` (`:tier env prog`, tier-keyword dispatch) into standalone
1-arg kernel verbs — **`:wat::kernel::spawn-thread'`** (takes a self-peer prog → returns
`Thread'<I,O>`) and **`:wat::kernel::spawn-process'`** (takes forms → returns
`Process'<I,O>`). No tier keyword, no env arg. These are the targets the coming host-type
`defclause` (S2c-ii) will dispatch to. **Additive** — the monolithic `spawn-program'`
stays live (S2d migrates + cuts it). Share the per-tier logic with the monolith (extract
helpers; do NOT duplicate). The committed probe `s2ci_spawn_thread_prime_round_trip` flips
RED→GREEN.

**The contract:**
- `(:wat::kernel::spawn-thread' prog)` — `prog : [self <- :wat::kernel::Peer'<S,R>] -> nil`
  (the S2a self-peer model) OR the legacy apply-loop `[I] -> O`; returns `Thread'<R,S>`
  (self-peer) / `Thread'<I,O>` (apply-loop) — i.e. the SAME projection
  `infer_spawn_program_prime` already computes for its `:thread` branch.
- `(:wat::kernel::spawn-process' forms)` — `forms` = the forms-server program (a vec);
  returns `Process'<I,O>` — the same as the monolith's `:process` branch.

**Read in order (the rooms):**
1. `tests/nursery/probe_arc259_s2ci_spawn_thread_prime.rs` — the GREEN target (RED at HEAD:
   `spawn-thread'` resolves to `:?0`). Make it pass.
2. `src/kernel/spawn.rs` — `eval_kernel_spawn_program_prime` (the `:thread` branch calls
   `spawn_thread_peer(program_fn, …)`; the `:process` branch calls `spawn_process_peer(forms, …)`).
   **Add** `eval_kernel_spawn_thread_prime(args, list_span, env, sym)` (1 arg: eval `args[0]`
   as the prog fn → `spawn_thread_peer`) and `eval_kernel_spawn_process_prime` (1 arg: eval
   `args[0]` as forms via `expect_vec_ast_pub` → `spawn_process_peer`). Factor the shared body
   so the monolith's branches and the new verbs call ONE path each (no duplicated logic).
3. `src/runtime.rs:4509` — the `":wat::kernel::spawn-program'" =>` eval arm. Add two sibling
   arms: `":wat::kernel::spawn-thread'" => eval_kernel_spawn_thread_prime(...)` and
   `":wat::kernel::spawn-process'" => eval_kernel_spawn_process_prime(...)`.
4. `src/check.rs:9715` — `infer_spawn_program_prime`. Its `:thread` path projects the prog
   fn → `Thread'<…>` (the Peer'/legacy dual logic, S2a); the `:process` path → `Process'`.
   **Extract** `infer_spawn_thread_prime(args, head_span, …)` (1 arg) and
   `infer_spawn_process_prime` (1 arg) that REUSE that projection. (Factor the per-tier
   projection into a shared helper the monolith + the new verbs both call.)
5. `src/check.rs:4797` — the `":wat::kernel::spawn-program'" =>` check arm. Add two sibling
   arms routing to `infer_spawn_thread_prime` / `infer_spawn_process_prime`.
6. **Builtin registry** — grep for where `:wat::kernel::spawn-program'` is registered as a
   known builtin name (so it is not flagged "undefined"): `grep -rn "spawn-program'" src/ |
   grep -iE "builtin|known|register|KERNEL_|resolve"`. Add `:wat::kernel::spawn-thread'` +
   `:wat::kernel::spawn-process'` to the same registry/set.

**Blast radius:** `src/kernel/spawn.rs` (two new eval fns + factored helpers),
`src/runtime.rs` (two eval dispatch arms), `src/check.rs` (two infer fns + two check dispatch
arms + the shared projection helper), and the builtin-name registry. The monolithic
`spawn-program'` (eval + infer) stays untouched and live. No parser changes; no wat files.

**STOP triggers (halt + report; do not work around):**
- **STOP-1 (no duplication):** the per-tier eval + check logic must be FACTORED and shared
  between the monolith and the new verbs — not copy-pasted. If you cannot share cleanly,
  STOP and report (a duplicated projection is a solvere braid that will drift).
- **STOP-2:** if making the probe green requires changing the monolithic `spawn-program'`'s
  behavior, the parser, the `:process` model, or the peer verbs (`send'`/`recv'`/`close'`),
  STOP and report — S2c-i only ADDS the two primitives.

**Done = green:**
- `cargo test --release -p wat --test nursery probe_arc259_s2ci` → passes (42).
- `cargo build --release` clean.
- `cargo test --release -p wat --test nursery probe_arc259_s2a` + `probe_arc214_stone46aii_peer_verbs`
  still green (the monolith + S2a unregressed).
- `cargo test --release -p wat --test nursery -- --test-threads=1` → only the 4 known
  pre-existing reds (arc-255 reflection ×2 + undefined-builtin ×2).

**Mirror for shape:** `eval_kernel_spawn_program_prime` (the `:thread`/`:process` branches)
+ `infer_spawn_program_prime` (the per-tier projection) ARE the logic you factor + reuse.
The new verbs are 1-arg slices of the existing 3-arg monolith.
