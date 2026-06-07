# Arc 254 — comms integration: serve Values transport-agnostically; annihilate typed_channel

**Status:** OPEN (2026-06-06). Contract PINNED (four-questions verdict below). Highest priority — displaces 252.2 coverage + 253 instance-2.

> The builder's framing: *"the thing was being able to serve values to callers — they don't care if it's a thread channel, a process pipe or a network socket."* That requirement is the governing law of this arc.

## Provenance — why this exists

- **Arc 214** (concurrency toolkit) built the `comms/` tier *primitives* (Layers 0a–0b): `comms::thread` + `comms::process` wrappers and the io_uring N-way `Select` (`comms/process.rs:865`, multi-arm `POLL_ADD`, `POLLHUP` death-attribution). But its **Slices 4–9 — the integration (kernel verbs, `Thread<I,O>`/`Process<I,O>` peer structs, the Value-shim migration, the structural wall, brackets, services) — were never built.** The io_uring `Select` is fully built, tested, and warded — it is just **UNWIRED** (no live consumer reaches it yet). **io_uring REMAINS the select substrate for every fd-backed handle (process + future socket); arc 254 promotes it to the live path — it is the replacement, never the casualty.** What gets annihilated is `typed_channel`'s *inferior* select: `crossbeam_channel::Select` (crossbeam-only, rejects PipeFd) + the racy `libc::poll(timeout=0)` snapshot — exactly the things io_uring was built to replace.
- **Arc 170 slice 1c** deferred fd-backed `select` ("no consumer demand today") — `runtime.rs:18592`. The wat `select` verb (`eval_kernel_select`, `runtime.rs:18560`) is crossbeam-only and *actively rejects* PipeFd receivers (`runtime.rs:18606`).
- Result: the live wat channel surface runs on the **older `typed_channel`** stack; the better io_uring `comms/` stack sits unwired (`comms/` referenced in `src/` only by `lib.rs:65 pub mod comms;` + a doc line in `function/mod.rs`).

## Grounded findings (scouts + orchestrator reads, this session)

1. **`comms` tiers are correctly split for this**:
   - `comms::thread::Sender<T>/Receiver<T>` — **no `HolonRepresentable` bound**, only `T: Send + 'static` (`thread.rs:106,190`). In-process, by-move via crossbeam. `Send+Sync`, clone-shares-channel.
   - `comms::process::Sender<T>/Receiver<T>` — `T: HolonRepresentable` (`process.rs:144,249`), EDN-framed over a pipe. `Sender`: `Send+Sync`, no Clone (PIPE_BUF atomicity). `Receiver`: `Send + !Sync` (owns a per-receiver `RefCell<IoUring>`); Clone = independent competing endpoint (dup'd fd, fresh ring).
   - `comms::process::Select<'a,T>` — `!Send+!Sync` thread-local combinator; monomorphic in `T`; single-tier (no heterogeneous fan-in).
2. **`HolonRepresentable`** (`comms/mod.rs:111-116`): `Send + 'static`; `to_holon_ast(&self) -> HolonAST` (**infallible**) + `from_holon_ast(&HolonAST) -> Result<Self, WireError>`.
3. **`impl HolonRepresentable for Value` does not exist** and cannot faithfully cover the full enum: ~16 opaque variants (closures `wat__core__fn`/`clauses`, `Sender`/`Receiver`/`ProgramHandle`/`HandlePool`/`ChildHandle`, `io__IOReader`/`Writer`, `RustOpaque`, `OnlineSubspace`/`Reckoner`/`Engram`/`EngramLibrary`/`Hologram`, `wat__WatAST`) render as `#wat-edn.opaque/X nil` and cannot round-trip (`edn_shim.rs:1613-1638`). `Value↔EDN` exists (`edn_shim::value_to_edn_with` :1506, `read_edn` :332) for the serializable subset.
4. **`typed_channel` blast radius**: ~84 refs / 9 files. Live consumers in `runtime.rs` (verbs + channel/peer construction), `spawn.rs`, `fork.rs`, `thread_io.rs`, `value/value.rs` (the `wat__kernel__Sender/Receiver` variants carry `SenderInner`/`ReceiverInner`). `check.rs`/`types.rs` refs are comment/string-only. Dead-from-outside: `unbounded`, `make_pipe_channel_pair`, `make_thread_peer_pair_for_test`, the `RecvError`/`SendError`/`TryRecvError` re-exports.

## The contract — PINNED (four-questions verdict)

Governing requirement: *a caller serves/receives Values and never cares about the transport; thread → pipe → socket changes nothing for the caller.*

| Option | Obvious | Simple | Honest | UX |
|---|---|---|---|---|
| **A — uniform: every channel carries `HolonRepresentable` Values, type-enforced, all transports** | YES | YES | YES | YES |
| B — per-transport capability (thread=any Value, process/socket=serializable) | NO | NO | NO | NO |
| C — uniform "any Value", runtime-fail on non-serializable transports | ~ | NO | NO | NO |

**A strictly dominates (4×YES).** B and C make the caller care about the transport — the one thing forbidden.

**The contract:** wat channel payloads are `HolonRepresentable` (serializable), **enforced by the type checker, uniformly across all transports.** Opaque Values are *resources, not messages* — not channel-able on any transport. `impl HolonRepresentable for Value` covers the serializable subset; its opaque arm is unreachable-by-typecheck with a clean-`WireError` backstop. The trait's infallible `to_holon_ast` is acceptable *because* the checker guarantees it never sees an opaque variant.

## Stones (sequence; each its own STRIKE-READY brief + FM-2-bis probe)

1. **254.1 — the bridge + the constraint (gates everything).** `impl HolonRepresentable for Value` (serializable subset, reusing `edn_shim`); type-checker constraint: channel payload types must be serializable (uniform, all transports). FM-2-bis probe: a non-serializable channel payload is rejected at check time; a serializable one round-trips.
2. **254.2 — thread tier onto `comms::thread`.** Replace `typed_channel` Crossbeam path; `wat__kernel__Sender/Receiver` carry comms thread endpoints; clean drop-in (no serialization at runtime — by-move).
3. **254.3 — process tier onto `comms::process`** (io_uring). Replace the PipeFd path. **OWNERSHIP RESOLVED by de-risk probe (`tests/probe_arc254_process_ownership.rs`, 2026-06-06):** process receivers are **single-owner-move** — a single owner drains every frame losslessly and in order (the per-receiver accumulator is drained by that same owner). **Dup-clone fan-out is DISCONFIRMED** (lossy: a clone greedily reads multiple frames into its private accumulator; dropped-with-buffer strands them). ⇒ the wat process-receiver handle is single-owner (NOT `Arc`-shared across threads — `Arc<!Sync>` is `!Send` anyway); clone-as-fan-out is not exposed; **multi-reader fan-in is served by `select` over N distinct single-owner channels** (254.4), never by cloning one receiver.
4. **254.4 — `select` onto `comms`**: the **io_uring `comms::process::Select` becomes the LIVE select for every fd-backed handle** (process + future socket); `comms::thread::Select` (crossbeam `select!`) serves in-memory thread channels (not file handles — io_uring is unneeded there). Delete the crossbeam-only PipeFd rejection (`runtime.rs:18606`). This is the remote-program fan-in the builder thought was already done — built in 214, now made live.
5. **254.5 — ANNIHILATE `typed_channel`** (hard cut; all 84 refs migrated or deleted).
6. **254.N — ward (`comms/` re-earn vigilatum incl. the new wat-facing surface) + INSCRIPTION.**

## Out of scope (affirmative cuts)

- **Passing opaque Values (closures/handles) through channels** is RETIRED, uniformly. Channels carry messages, not resources. Any existing thread-channel test that sends a non-serializable Value goes red — that is the substrate teaching us where the old non-uniform assumption leaked; fix the cascade, do not bridge it.
- **Remote tier + reactor tier**: arc 214 left empty seats; this arc wires thread + process and makes the surface socket-ready (uniform contract + fd-select), but does not mint the remote/socket transport itself. A future arc adds the `Socket` `ReceiverInner`/tier; it requires zero caller change by construction.

## Risks / traps

- **254.3 ownership reconciliation** — **RESOLVED 2026-06-06** by `tests/probe_arc254_process_ownership.rs` (3/3 green): single-owner-move is sound; dup-clone fan-out is lossy and dropped. No longer an open trap. The remaining 254.3 work is mechanical: thread the single-owner receiver through `spawn` (move, not Arc-share).
- **Cascade size** (254.5): ~84 sites. Surgical, per substrate-as-teacher; fail-count is the progress meter.
