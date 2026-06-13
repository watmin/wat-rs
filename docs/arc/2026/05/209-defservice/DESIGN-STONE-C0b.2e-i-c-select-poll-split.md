# DESIGN-STONE C0b.2e-i-c — split `select'` → `select'` + `poll'`; `SelectEvent` → `ServiceEvent`

> The naming-honesty stone (intueri cast → weighed → builder-ratified). `select'` is one
> name with two meanings (the return type depends on arity); split it. Pure rename+split,
> no behavior change — the 3-arg service logic moves verbatim under the honest name `poll'`,
> and its event sum is renamed to name its domain. Do this BEFORE C0b.3a-ii so the socket
> service multiplexer is built on `poll'`. (Numbered i-c because it completes the C0b.2e-i
> connection-surface; functionally task #233.)

## Why

`(select' peers) -> Tuple<i64,O>` (arity-1, first-ready fan-in over homogeneous peers) and
`(select' self listener clients) -> SelectEvent<I,O>` (arity-3, service multiplexer
returning a tagged event sum) are two different operations sharing one name — the return
TYPE depends on the arity (the one-name-two-meanings smell). Decision (made): `select'`
keeps the canonical fan-in (`alts!`/Go-`select`); the 3-arg form becomes **`poll'`**
(`poll(2)` semantics — block until an event on any of these inputs, classify it; names the
event-getter, not the loop, so it avoids the rejected `serve'` verb). The event sum
`SelectEvent<I,O>` → **`ServiceEvent<I,O>`** (names the domain — service-lifecycle events —
not the operation that generates them).

## Grounded this session (HEAD `6fb7a833`)

- Kernel verbs are recognized solely by their dispatch match arms — no separate allowlist:
  runtime.rs:4546 `":wat::kernel::select'" => eval_peer_select_prime(...)`; check.rs:4851
  `":wat::kernel::select'" => infer_select_prime(...)`. Adding the arm IS the registration.
- `eval_peer_select_prime` (runtime.rs:23451) arity-dispatches: `args.len()==3` →
  `eval_peer_select_prime_3arg` (runtime.rs:23873, builds the `SelectEvent` variants).
- `infer_select_prime` (check.rs:10837) routes `args.len()==3` → `infer_select_prime_3arg`
  (check.rs:10950, returns `Parametric{"wat::kernel::SelectEvent",[I,O]}`).
- `SelectEvent` defenum: `wat/spawn.wat:67` (`:Shutdown :Connection :Message :Closed :Lost`).
- `SelectEvent` references: runtime.rs (`SELECT_EVENT_TYPE` const :23873 + variant comments),
  check.rs (4 head strings :10964/10995/11022/11032 + comment :6575), test
  `tests/nursery/probe_arc209_c0b1b_select_listener.rs` (uses `(select' self l clients)` +
  `SelectEvent::Shutdown/Connection/Message/Closed/Lost`).
- The 1-arg `select'` callers (UNCHANGED): `wat/bracket.wat:54`, `connection_primitive`,
  `probe_arc214_stone46b`, `peer_select_prime_process`.

## The contract decision (pinned)

**`poll'` is a new verb routing to the existing 3-arg logic; `select'` becomes 1-arg-only.**
1. **runtime.rs:** add `":wat::kernel::poll'" => eval_poll_prime(...)`. Rename
   `eval_peer_select_prime_3arg` → `eval_poll_prime`. `eval_peer_select_prime` drops its
   3-arg branch — `args.len() != 1` is an error naming `poll'` (the honest hard-cut, not a
   silent reinterpret).
2. **check.rs:** add `":wat::kernel::poll'" => infer_poll_prime(...)`. Rename
   `infer_select_prime_3arg` → `infer_poll_prime`. `infer_select_prime` drops the
   `args.len()==3 → _3arg` routing — 3 args is a clear CheckError ("`select'` takes one
   peer vector; the 3-arg service multiplexer is `poll'`").
3. **`SelectEvent` → `ServiceEvent`** everywhere: the `wat/spawn.wat` defenum head + all
   variant references; the runtime `SELECT_EVENT_TYPE` const + variant construction; the
   check.rs head strings; the c0b1b test's `(select' …)` → `(poll' …)` and
   `SelectEvent::*` → `ServiceEvent::*`. No behavior change — same variants, same fields.

`select'` 1-arg is byte-for-byte unchanged in behavior (the fan-in). `poll'` is the 3-arg
logic verbatim under the new name. This is a rename+split, not a reimplementation.

## The gate (rename+split — structural disconfirm)

1. **Structural:** `grep -rn "SelectEvent" src/ wat/ tests/` → EMPTY (all `ServiceEvent`);
   `:wat::kernel::poll'` present (runtime + check arms); a `(select' a b c)` 3-arg form is
   now a CheckError.
2. **Capability/regression — the migrated c0b1b is the proof `poll'` works:**
   `probe_arc209_c0b1b_select_listener` (migrated to `(poll' …)` + `ServiceEvent`) GREEN.
3. **1-arg `select'` unchanged:** `connection_primitive`, `probe_arc214_stone46b`,
   `peer_select_prime_process`, and the brackets path (`probe_arc209` bracket tests) GREEN.
4. Nursery serial **895/4** (baseline only) + full workspace test surface compiles.

## Files touched

`src/runtime.rs` (poll' arm + rename eval fn + select' 1-arg-only), `src/check.rs` (poll'
arm + rename infer fn + select' 1-arg-only + 4 head strings), `wat/spawn.wat` (defenum
rename), `tests/nursery/probe_arc209_c0b1b_select_listener.rs` (migrate to poll'/ServiceEvent),
`src/kernel/mod.rs` (doc mention). No `peer.rs`/`spawn.rs`/`comms` change.

## STOP triggers (rejection — ship nothing, report)

1. **STOP-1:** a `SelectEvent` reference exists that is NOT a simple rename target (e.g. a
   load-bearing string compared elsewhere) — STOP, report.
2. **STOP-2:** removing `select'`'s 3-arg branch breaks a 1-arg caller (it should not — they
   are independent) — STOP, report.
3. **STOP-3:** `poll'` is not recognized as a builtin after adding the arms (resolve/reflection
   needs more than the dispatch arm) — STOP, report (the design expects the arm to suffice,
   mirroring `select'`).

## Out of scope (rejected — NOT deferred)

- The `poll'` **socket** service multiplexer (process-tier 3-arg over `process::Select`,
  consuming `ReactorClass`) = **C0b.3a-ii** (this stone only renames the existing thread-tier
  logic; the socket reactor is 3a-ii).
- `SocketListener'`→`Listener'` = **C0b.2e-ii**; `SocketAddress'`→`Address'` = **C0b.2e-iii**.

## The deadlock contract carries

Pure rename+split; no transport/lifecycle/behavior change. [[feedback_vended_primitives_never_deadlock]]
[[feedback_optional_is_a_smell]]
