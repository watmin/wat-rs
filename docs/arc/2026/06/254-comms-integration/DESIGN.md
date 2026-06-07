# Arc 254 — comms integration: serve Values transport-agnostically; annihilate typed_channel

**Status:** OPEN (2026-06-06). Contract PINNED (four-questions verdict below). Highest priority — displaces 252.2 coverage + 253 instance-2.

## ★ REFRAME (2026-06-07) — this IS arc 214 Slice 4–9; finish what we set out to build

We forked OUT of arc 214 at the kernel layer (Slice 4 shipped only program-env 4.1–4.3; `src/kernel/` was never minted), kept forking, and circled back — *through the same door, the other way, carrying everything we went to fetch.* The forks were not wandering; they **built 214's dependency tree bottom-up.** The deps are now met:

| 214 Slice 4–8 needs | built by |
|---|---|
| dispatch on peer type (polymorphic verbs) | arc 146 multimethod + 237 defclause |
| Value home for peer variants | 251.2 (`src/value/`, "the keystone") |
| clean error/wire shapes | 243 (conformare) |
| warded wat stdlib (services are wat) | 245 |
| macro engine (brackets) | 249 |
| hardened comms `try_recv` | 253 |
| portability predicate (no handles on channels) | 254.1 |
| comms tier primitives (pair/Sender/Receiver/Select/io_uring) | 214 Slices 1–3 |

**THE CONVERGENCE:** arc 254 = 214 **Slice 5** (typed_channel→comms migration); the stdio services redesign = 214 **Slice 8** (universe-resident services, no handle-passing); the arc-170 orphan leak (253 instance-2) dies in the same move — the new peer types own their fds via comms/RAII, replacing hand-managed `into_raw_fd`. **One piece of work.**

**THE GATE:** 214 **Slice 4 kernel layer** (`src/kernel/peer.rs` + `spawn.rs`): `Thread<I,O>`/`Process<I,O>` peer types built on comms (**RAII fd ownership — the leak-killer**), the spawn dispatcher (`spawn-program' :tier`), the polymorphic verbs (`send'`/`recv'`/`select'`/`close'` — multimethod, "mostly wiring" per `214 DESIGN:541`). Readiness audit (2026-06-07): deps met; comms exposes channel construction (`pair`) but **not** spawn — the kernel layer builds spawn-on-comms, replacing typed_channel + `into_raw_fd`.

**REMAINING ORDER:** Slice 4 (kernel layer) → Slice 5 (this migration + the 254.1 portability gate) → Slice 6 (structural wall) → Slice 7 (brackets) → Slice 8 (services universe-resident; the leak dies) → Slice 9 INSCRIPTION (closes 214 at last). The "254" stones (254.1 done; 254.2–254.5) fold in as Slice 5.

**THE UNWIND (spawn-block-winding — close depth-first, inscribe each; do NOT leave an open door like 214 was):** three open arcs — **214** (never inscribed), **253** (instance 1 done, instance 2 = the leak, OPEN), **254** (opened from the 253 hunt, = 214 Slice 5) — are ONE convergent body of work. Building 214 forward unwinds the stack: 253 instance-2 (the leak) dies in Slice 4 (RAII peer fds) + Slice 8 (no handle-passing) → **INSCRIBE 253**; 254 = Slice 5, its record folds into 214; Slice 9 → **INSCRIBE 214**. Each arc gets an honest closure as it completes. The failure we are correcting — 214 left warded-but-unwired and un-inscribed, reading as "done" — must not recur; no forgotten doors.

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

### The fuller contract — universe-residency + Mini-TCP at depth 1 (ALREADY DECIDED 2026-05-19; never propagated to wat)

The serializable contract is one half. The other half was decided 2026-05-19 (`docs/arc/2026/05/214-concurrency-toolkit/DESIGN.md:100-149`, user direction) and applied to `comms` but **never reached the live wat verbs**:

- **Universe-residency:** the universe provides ONE comm channel; the program never knows its transport (thread/process/socket). Program code is `peer.send(v)` / `peer.recv()`, identical across tiers. (This IS the builder's "serve values, they don't care about transport.")
- **Mini-TCP at depth 1:** every channel is **capacity-1**. Send one, read back (ack or data) — the only supported pattern. N>1 was proven (trading-lab) to cause "massive perf hits + entire categories of problems." Lock-step by construction.
- **ONE factory:** the four-questions verdict (`DESIGN.md:123-139`) RETIRED `bounded(N)` (FAILS 4×) in favour of a single `pair()` = `bounded(1)` (PASSES 4×).

**Therefore arc 254 ALSO annihilates the stale wat channel constructors** that predate this: `:wat::kernel::make-unbounded-channel` (unbounded — condemned) and `:wat::kernel::make-bounded-channel` *with arbitrary N* (only depth-1 survives). They collapse to **ONE depth-1 wat channel factory** — the wat mirror of `comms::{thread,process}::pair()`. `wat/kernel/channel.wat` + every caller migrate. This is the same already-decided principle the io_uring stack already embodies; we are propagating it to the surface, not inventing it.

## Stones (sequence; each its own STRIKE-READY brief + FM-2-bis probe)

1. **254.1 — the bridge + the constraint (gates everything).** `impl HolonRepresentable for Value` (serializable subset, reusing `edn_shim`'s `Value↔EDN`). **The checker constraint REUSES the existing portability classifier — it does NOT invent `is_wire_serializable`.** `src/closure_extract.rs` already classifies types as portable/non-portable (the `NonPortableCapture` machinery: portable types encode to a reconstructible form — Uuid→`from-string`, Char→`char/of`, List→variadic; non-portable arms = Sender/Receiver/handles/closures, line 1729). That is the *same concept* as wire-serializability ("can this cross a universe boundary?"): closures may only **capture** portable values across universes; channels may only **carry** portable values. 254.1 unifies them. CONFIRMED (`closure_extract.rs:1476` `encode_value_to_ast`): the existing classifier is **value-level** (dispatches on `Value` variants). So 254.1 factors a **type-level sibling `is_portable_type(TypeExpr) -> bool`** from the SAME classification, and it LEVERAGES the existing record/struct type distinction rather than brute-walking values:
   - atom / Uuid / Char / portable-container<portable> → portable;
   - **Record** (`TypeDef::Record`, `types.rs:1987`) → **portable always** (holon-representable by construction — it's the holon algebra's native form, `is_holon_or_record` `check.rs:11819`);
   - **Struct** (`defstruct`) → **portable iff every field type is portable** (recurse) — this is the precise gap, since a struct field may be opaque (`HandlePool<Sender<i64>>` etc.);
   - Sender / Receiver / handle / closure / `Fn` / IOReader / WatAST → non-portable.
   One source of truth shared by the value encoder and the new checker gate. The checker constrains channel payload types with `is_portable_type`. FM-2-bis probe: a non-portable channel payload (e.g. `Sender<...>`) is rejected at check time; a portable one (i64/string/vector) round-trips. This is `solvere` — collapse two parallel notions into one.

   **Warded-home promotion (builder directive):** the value encoder lives today in the flat quarry `src/closure_extract.rs`. The unified portability concern (value encoder + the new `is_portable_type` + the one classification source-of-truth) is **lifted into a warded home `src/portability/` and warded (vigilia → L1+L2=0 → vigilatum stamp) WHEN READY** — flat quarry → warded home, the Phoenix great-migration applied here. Promotion fires when the concern is clean and unified, not forced mid-extraction; until then the classifier is shared in place.

   **STATUS — gate DONE (load-bearing PASS; lib 940/0/1; probe rejects struct-with-Sender; controls green; clippy clean; see `SCORE-STONE-254.1.md`). TWO grounded deferrals:** (A) **enum portability NOT enforced** (`TypeDef::Enum → true`) — the stdlib's own service-control enums (`wat/kernel/services/{stdout,stdin,stderr}.wat` `*Service::Event`) carry `Receiver` fields sent through `make-bounded-channel`: **a uniform-portability VIOLATION inside the stdlib** (the non-uniform leak this arc predicted). Enforcement is BLOCKED on redesigning that pattern → folded into **254.3** (process tier / control-channel ownership). (B) formal type-param `:T → true` rests on instantiation-site re-checking — **VERIFY or it's a hole** (254.x).
2. **254.2 — thread tier onto `comms::thread`.** Replace `typed_channel` Crossbeam path; `wat__kernel__Sender/Receiver` carry comms thread endpoints; clean drop-in (no serialization at runtime — by-move).
3. **254.3 — process tier onto `comms::process`** (io_uring). Replace the PipeFd path. **OWNERSHIP RESOLVED by de-risk probe (`tests/probe_arc254_process_ownership.rs`, 2026-06-06):** process receivers are **single-owner-move** — a single owner drains every frame losslessly and in order (the per-receiver accumulator is drained by that same owner). **Dup-clone fan-out is DISCONFIRMED** (lossy: a clone greedily reads multiple frames into its private accumulator; dropped-with-buffer strands them). ⇒ the wat process-receiver handle is single-owner (NOT `Arc`-shared across threads — `Arc<!Sync>` is `!Send` anyway); clone-as-fan-out is not exposed; **multi-reader fan-in is served by `select` over N distinct single-owner channels** (254.4), never by cloning one receiver.
4. **254.4 — `select` onto `comms`**: the **io_uring `comms::process::Select` becomes the LIVE select for every fd-backed handle** (process + future socket); `comms::thread::Select` (crossbeam `select!`) serves in-memory thread channels (not file handles — io_uring is unneeded there). Delete the crossbeam-only PipeFd rejection (`runtime.rs:18606`). This is the remote-program fan-in the builder thought was already done — built in 214, now made live.
5. **254.5 — ANNIHILATE `typed_channel` + the stale constructors** (hard cut). All 84 `typed_channel` refs migrated or deleted; `:wat::kernel::make-unbounded-channel` removed; `:wat::kernel::make-bounded-channel(N)` collapsed to the ONE depth-1 factory (the wat mirror of `comms::pair()`). `wat/kernel/channel.wat` + every caller migrate to the single factory. The dungeon — unbounded, bounded-N, typed_channel, crossbeam-only select — is wiped.
6. **254.N — ward (`comms/` re-earn vigilatum incl. the new wat-facing surface) + INSCRIPTION.**

## Out of scope (affirmative cuts)

- **Passing opaque Values (closures/handles) through channels** is RETIRED, uniformly. Channels carry messages, not resources. Any existing thread-channel test that sends a non-serializable Value goes red — that is the substrate teaching us where the old non-uniform assumption leaked; fix the cascade, do not bridge it.
- **Remote tier + reactor tier**: arc 214 left empty seats; this arc wires thread + process and makes the surface socket-ready (uniform contract + fd-select), but does not mint the remote/socket transport itself. A future arc adds the `Socket` `ReceiverInner`/tier; it requires zero caller change by construction.

## Risks / traps

- **254.3 ownership reconciliation** — **RESOLVED 2026-06-06** by `tests/probe_arc254_process_ownership.rs` (3/3 green): single-owner-move is sound; dup-clone fan-out is lossy and dropped. No longer an open trap. The remaining 254.3 work is mechanical: thread the single-owner receiver through `spawn` (move, not Arc-share).
- **Cascade size** (254.5): ~84 sites. Surgical, per substrate-as-teacher; fail-count is the progress meter.
