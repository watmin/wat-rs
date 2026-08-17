# BRIEF — 293.W.2e: mint `:wat::kernel::address-wire?`

> **Executor: one leaf.** Orchestrator drew this and weighs the kill. Work ONLY in `wat-rs/`.
> Commit nothing. Leave the tree green for the orchestrator to weigh.

## What to do

Mint one PURE projection: `(:wat::kernel::address-wire? addr) -> bool`. It un-erases the fact
`Address::portable_form()` already knows (`Some` = wire, `None` = shared memory). This composes
pieces that all already exist (verified): the measurement, the Address downcast, and the
`peer-wire?` infer/eval shape. Copy those. Do not invent a type parameter.

## Read in order

1. `docs/arc/2026/06/293-struct-record-symmetry/DESIGN-STONE-293.W.2e-address-wire.md` — why,
   the contract, what is REJECTED.
2. `src/kernel/address.rs:306-317` — `portable_form`. The body of the verb is
   `portable_form().is_some()`.
3. `src/runtime.rs:25727-25753` — `eval_connect_prime` downcasts `ADDRESS_TYPE_PATH` to
   `&Address`. Reuse this match, then call `portable_form` instead of `connect_as_value`.
4. `src/runtime.rs:6764` and `31520-31567` — `peer-wire?` dispatch + eval. Same purity,
   same bool, different type-path (`ADDRESS_TYPE_PATH`, not `PEER_TYPE_PATH`).
5. `src/check.rs:4117-4124` and `10801-10841` — `infer_peer_wire`. Twin: 1 arg, unify against
   `Address<S,R>` (see `infer_connect_prime` at `src/check.rs:9514-9517` for the expected
   type), return `:wat::core::bool`. Do **not** call `project_peer_io`.
6. `src/kernel/spawn.rs:152` — `ADDRESS_TYPE_PATH`.
7. The committed probe: `tests/comms/probe_arc293_W2e_address_wire.{rs,wat}`.

## Implementation sketch

```rust
// runtime.rs dispatch, next to peer-wire?:
":wat::kernel::address-wire?" => eval_address_wire(args, list_span, env, sym),

// eval_address_wire: arity 1; eval the arg; downcast ADDRESS_TYPE_PATH like
// eval_connect_prime; Ok(Value::bool(addr.portable_form().is_some()));
// else TypeMismatch { expected: "Address<S,R>", got: snapshot }.

// check.rs infer_list arm + infer_address_wire:
// arity 1; infer arg; unify with Address<?,?>; return Path(":wat::core::bool").
```

## Discipline

- `src/runtime.rs` + `src/check.rs` ONLY (plus the already-committed probe).
- No new `Value` variant. No `Address<S,R,T>`. No `/start` change. No compiler raise on
  `connect` / Setup. No 255 registry hoist. No holon-rs.
- Build any corrective/transform script as a Rust Cargo binary under repo-local `tools/`.
  Do NOT use Python or shell mass-editors. `/tmp/` is firewall-blocked. Delete the tool
  before you stop.

## STOP triggers

1. If you want a third type parameter on Address — STOP. That is 2f.
2. If you want `peer-wire?` to accept an Address — STOP. Different type-path.
3. If `portable_form` is the wrong predicate (a process address returns `None`) — STOP and
   surface it. Do not invent a parallel tag read.
4. If `sym.types()` or a new registry looks tempting — STOP. This is a runtime-tag read,
   same as `peer-wire?`.

## FM 2-bis evidence

`tests/comms/probe_arc293_W2e_address_wire.rs` — `address_wire_is_false_on_thread_true_on_process`.
Pre-stone: startup/`UnknownFunction` on `:wat::kernel::address-wire?`. Everything around the
gap (two `listener` mints, `Bound/address`) already works (`probe_arc209_c0b1_thread_connection`,
`probe_arc272_autobind_listener`).

Negative control (KEEP as a test): a `.wat.bad` that applies `address-wire?` to an `i64` is a
`TypeMismatch` naming `Address<S,R>`. Write it. Keep it.

## SCORE doc spec

Copy `docs/arc/2026/06/293-struct-record-symmetry/SCORE-293.4d.md` shape (rows, command,
expected, honest delta). Leave it uncommitted if you write a draft; the orchestrator scores
against an independent re-run.

## Calibration

15–35 min. STOP at 50 min.
