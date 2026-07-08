# Arc 170 capability circuit — Stone 2: `:grants` on the process-locus + the bracket's grant-boot / revoke-shutdown (2026-07-08, prepped for 2026-07-09)

> **Parent design:** [`DESIGN-CAPABILITY-CIRCUIT-GRANT-REVOKE.md`](./DESIGN-CAPABILITY-CIRCUIT-GRANT-REVOKE.md).
> **Prior stone landed:** Stone 1 (`dc2ae7a6`) — `:wat::service::Grantable`, a struct-nature methods-surface
> every `<fqdn>::Handle` satisfies (macro auto-emits the `extend-type`), so a heterogeneous
> `Vector<:wat::service::Grantable>` grant/revoke's uniformly.
> **This stone:** teach the bracket to grant its workers on boot and revoke them on shutdown — in its OWN
> wat control flow, ack'd request/reply, zero fire-and-forget, no Rust `Drop`.

## Why — the deliverable

A circuit builder (`:user::main`) spawns a worker pool that must dial some of its process-services. Each
worker's kernel-vouched pid must be granted to those services before it dials, and revoked when it is
reaped — automatically, so a grant can never outlive the worker. Stone 1 gave the uniform `Grantable`
handle; this stone wires it into the bracket's lifecycle.

```clojure
;; :user::main — the circuit builder
(let [store-h (sqlite-store'/start :locus (process) :record ...)
      cache-h (mem-store'/start     :locus (process) :record ...)]
  (bracket/map
    (:wat::spawn::process/grants [store-h cache-h])   ; :grants rides the PROCESS locus (Vector<Grantable>)
    work-fn items))                                    ; each worker: granted on boot, revoked on shutdown
```

```
boot     → grant  each worker's pid to each :grants Grantable   (wat, ack'd request/reply, BEFORE first item)
work     → runners run; collect-loop drains the M mapped values
shutdown → revoke each worker's pid from each Grantable          (wat, ack'd request/reply, after drain)
return   → hand back the mapped vals
```

The bracket owns **both** ends → a grant it does not revoke cannot exist. That is the RAII, made of the
bracket's own control flow. Panic-safety is structural tear-down-together (a runner crash →
`assertion-failed!` propagates → `:user::main` unwinds → services SIGKILL'd → no surviving stale grant);
we do NOT fake a wat `finally`.

## The one contract decision (pinned — REVISED 2026-07-09, grounding switched it)

**The pid is a queryable property of the peer — a `peer-pid` accessor, NOT a `spawn-runner` reshape.**
Grounding (2026-07-09) found the process peer already *holds* the pid: a process `Peer'` value is
`Value::RustOpaque(PROCESS_PEER_TYPE_PATH)` → `ProcessPeerBundle` → `.peer : Process<String,String>`
("Pidfd + channels", `spawn.rs:254`) → `Pidfd` → `.pid()` (`clone.rs:216`). The `Pidfd` is the knowledge
bearer: kernel-minted, unforgeable (only mintable at fork — `clone.rs:97-101`), the same handle the reap
uses (PID-reuse-safe). So we don't reshape a surface — we read what's already there:

```clojure
(:wat::kernel::peer-pid p) -> (:wat::core::Option :wat::core::i64)
;;   process peer (far end is a forked child) -> (:Some child-pid)   ; kernel-vouched, from the Pidfd
;;   thread  peer (far end is a cell in-proc)  -> :None               ; no separate process
```

- **Name `peer-pid`, no trailing `'`** — intueri-cast (`scratchpad/peer-pid-accessor-naming.wat`), weighed,
  and the builder's cut ratified over the ward's `far-pid` verdict (2026-07-09): the fn *takes a peer*, so
  it names its subject — `(peer-pid p)` reads "the pid of this peer," sitting beside `send' peer` / `recv'
  peer` where the peer is always the subject. Intueri's "which end?" worry is a **phantom ambiguity**: a
  `Peer'` has no competing *local* pid (your own end is you — you'd never ask a peer for your own pid), so
  the only referent is the process on the other end. What the cast earned and we kept: it killed
  `remote-pid` (Level-1 lie — "remote" is false for the in-process `None` arm) and settled the `'` — none:
  it is a pure projection off the Pidfd, not an effect/rendezvous verb (`send'`/`recv'`/`connect'`). Used
  as an IDENTITY for the allow-set, never for `kill()`.
- **The `Option` encodes the firm boundary for free**: `Some` = the peer's counterpart is a separate
  process = grantable; `None` = in-process cell = no grant (the handle IS the capability).
- **`spawn-runner` is UNCHANGED** (`-> Peer'`) — no Locus-surface reshape, no `SpawnedRunner` record. The
  bracket reads `(peer-pid p)` after spawn. This is the correct-change-subtracts shape (`COMPONENDO DELEO`):
  the peer already carries the pid; we lift it, we don't rebuild a surface.

**Rust touch (small):** a `:wat::kernel::peer-pid` kernel fn — downcast the peer opaque; process bundle →
`(:Some pidfd.pid())`; thread peer opaque → `:None`. No return-type changes to `spawn_process_peer` /
`spawn-program'`.

## The strike (the rooms, read in order)

1. **`:wat::kernel::peer-pid` — the Rust accessor (the one substrate touch).** A kernel fn
   `[p <- Peer'<S,R>] -> (Option i64)`: downcast the peer opaque; if it is the process bundle
   (`PROCESS_PEER_TYPE_PATH` → `ProcessPeerBundle`), return `(:Some bundle.peer.pidfd.pid())` (the chain
   grounded in the contract section); if it is a thread-peer opaque, return `:None`. Register it beside the
   other `:wat::kernel::` peer verbs. `spawn-runner` / `spawn_process_peer` UNCHANGED (no return reshape).
2. `wat/spawn.wat` — the `ProcessOpts` record + its builders (`process`, `process/post-spawn`,
   `process/env`, … ~line 41-125). ADD: a `grants` field on `ProcessOpts`; a `process/grants` builder
   (parallel to `process/post-spawn`) carrying `(Vector :wat::service::Grantable)`; a `Locus`-level
   `grants` accessor returning `(Vector Grantable)` — **empty for thread** (the firm boundary — thread
   cells need no grant). `spawn-runner`'s `:features` sig is UNCHANGED (still `-> Peer'`).
3. `wat/bracket.wat:222-252` — `map-worker`, the coordinator. CHANGE the spawn `mapv` (line 233-241):
   - `p (:wat::spawn::Locus/spawn-runner locus work-fn)` — UNCHANGED (still a `Peer'`).
   - read `grantables (:wat::spawn::Locus/grants locus)` and `pid (:wat::kernel::peer-pid p)`.
   - **grant-before-send:** if `pid` is `(Some …)` and `grantables` non-empty, fold
     `(:wat::service::Grantable/grant g [pid])` over `grantables` (ack'd) BEFORE
     `(send' p (Tuple i item))` — so the grant lands before the worker's work-fn dials the service.
   - keep the `peer`s for `collect-loop` (unchanged); keep the `pid`s (a parallel `Vector<Option i64>`,
     read back via `peer-pid` from the peers if cleaner) for shutdown.
   - after `collect-loop` drains + before returning the sorted vals: **revoke** — for each `(Some pid)`,
     fold `(:wat::service::Grantable/revoke g [pid])` over `grantables` (ack'd).
4. `wat/bracket.wat` — `collect-loop` (169-212) and both `spawn-runner` `extend-type` impls (76-148) are
   **UNCHANGED** (still `Peer'` in / out). The whole grant/revoke lives in `map-worker`.

**Blast radius:** one small Rust kernel fn (`peer-pid`) + `wat/spawn.wat` (`:grants` on ProcessOpts) +
`wat/bracket.wat` (`map-worker` grant-boot/revoke-shutdown). No surface reshape, no new record, no change
to `spawn-runner` / `spawn_process_peer`. `map`/`each` wrappers unchanged (they call `map-worker`).

## The disconfirming probe (draw + run FIRST, before the strike)

Prove `peer-pid` lifts the pid off a real process peer (`Some`) and gives `None` for a thread peer, and
that the lifted pid drives a grant/revoke — isolate the one gap. `scratchpad/probe-cap2-peer-pid.wat`:
- Get a **process** peer (e.g. start a `:probe::echo'` service on `(process)`, `connect'` its addr → a
  process peer) and a **thread** peer (a `(thread)` service / `self-peer` → a thread peer).
- Assert `(:wat::kernel::peer-pid process-peer)` is `(Some p)` and `(:wat::kernel::peer-pid thread-peer)`
  is `:None`. Then, holding the service's `Grantable` Handle, `grant [p]` (a dial from p is accepted) then
  `revoke [p]` (a fresh dial from a recycled-pid stand-in is REFUSED).
- Pre-strike this fails on EXACTLY "`:wat::kernel::peer-pid` is undefined" — that named gap is the whole
  Rust touch; everything around it (getting the peers, grant/revoke) is already green. Commit it after.

## Gate (Expectations — weighed by own re-run)

| what | command | expected |
|---|---|---|
| build | `cargo build --release` | clean |
| peer-pid | `./target/release/wat scratchpad/probe-cap2-peer-pid.wat` | process peer → `(Some …)`; thread peer → `:None`; grant→dial-ok; revoke→dial-refused |
| the bracket circuit | a `bracket/map` over `(process/grants [h])`: workers dial the granted service, results correct; post-drain the granted pids are gone from the service allow-set (a post-return dial by a stand-in pid REFUSED) | green |
| thread untouched | a `bracket/map` over `(thread)` (no `:grants`) | identical results; no grant path taken |
| floor | `cargo nextest run --release` **run in the FOREGROUND (blocking) — never background-and-poll** | 4113+ pass / 0 new (modulo the known `no_inlined_wat` lint + the known `sigterm_to_cli` race → confirm the race by an isolated `--test-threads=1` pass) |

Runtime prediction: 20-40 min (one small Rust kernel fn + a coordinator edit). **STOP triggers:** (1) if
the thread-peer opaque cannot be distinguished from the process-peer opaque for the `peer-pid` downcast,
STOP + report the type paths. (2) if `Grantable/grant` inside `map-worker`'s spawn loop deadlocks (the
worker hasn't dialed yet — grant is on the owner's Handle, independent of the worker; it should not),
STOP + report.

## Out of scope / rejected

- **No Rust `Drop` / no `GrantGuard`** (four-questions killed it, parent design Crux 2): a `Drop` can't
  report failure → a request/reply revoke in `Drop` is a hidden fire-and-forget-on-error. The revoke lives
  in the bracket's wat flow.
- **No wat `finally`** — panic-safety is structural tear-down-together, not a faked unwind hook.
- **Thread `:grants` is a no-op / rejected** — the firm boundary: thread cells need no grant (the handle
  IS the capability, in-memory). Only a process crossing to a process-service is gated.
- **No `spawn-runner` reshape / no `SpawnedRunner` record** — the pid is read off the peer via
  `:wat::kernel::peer-pid` (the peer already holds the `Pidfd`); `grant`/`revoke` stay `-> nil` (ack'd).

## After this stone → Stone 3 = M1

The all-process core proof: `:user::main` starts B(proc), A(proc) + `store-h(B)/grant [pid_A]` (A deps B),
spawns a PROCESS pool with `:grants [A]` (grant-boot), runs (workers→A→B), drains + revokes-shutdown.
Prove: work completes; a post-shutdown dial by a would-be-recycled pid is REFUSED; the granted pool child
did not reparent (`PPID == owner`). The deterministic refusal proof.

---

## RESUME (curare — 2026-07-08 EOD, resume 2026-07-09 AM)

```clojure
{:head   "cd0f0d02 — the capability design corrected (bracket owns grant+revoke, no Rust Drop); this stone-2 doc on top"
 :branch "arc-170-gap-j-v5-deadlock-state"
 :landed-today
 ["revoke verb (be783977) · docs reframed 293→170 (95044479) · STONE 1 :wat::service::Grantable (dc2ae7a6) ·
   design corrected: the four-questions killed the Rust-Drop GrantGuard — the bracket owns grant+revoke in
   its own wat flow, zero fire-and-forget, panic-safety = structural tear-down-together (cd0f0d02)"]
 :settled-design
 ["the bracket grants-on-boot (before first item) / revokes-on-shutdown (after drain), ack'd request/reply, in WAT"
  "the pid is read off the peer: (:wat::kernel::peer-pid p) -> (Option i64) — a small Rust accessor
   (the peer already holds the Pidfd); process peer -> (Some pid), thread peer -> None. spawn-runner UNCHANGED.
   name `peer-pid`, no ' — intueri-cast, builder ratified over the ward's far-pid (the fn takes a peer)"
  ":grants rides the process-locus (a ProcessOpts field + a process/grants builder, Vector<Grantable>)"]
 :resume-at "draw scratchpad/probe-cap2-peer-pid.wat (the disconfirming probe — peer-pid lifts Some off a
             process peer / None off a thread peer + drives grant/revoke), run it (fails on `:wat::kernel::peer-pid
             undefined`), THEN the strike (this doc's rooms: the peer-pid Rust kernel fn; spawn.wat :grants on
             ProcessOpts; bracket.wat map-worker grant-boot/revoke-shutdown; spawn-runner/collect-loop UNCHANGED).
             Delegate to a shadowdancer (brief it to run cargo nextest FOREGROUND-blocking), weigh by own re-run, commit."
 :do-nots
 ["NO Rust Drop / NO GrantGuard / NO wat finally (see parent Crux 2). grants are WAT, ack'd, zero fire-and-forget"
  "the firm boundary: thread :grants is a no-op; a process cannot reach a thread-service; no unified-fd-peer"
  "BRIEF shadowdancers to run cargo nextest FOREGROUND-blocking, never background-and-poll (a sonnet looped on that)"
  "WEIGH by your OWN re-run; a mid-edit file is a PHANTOM; commit + push often; the holonic repos ARE the memory"]}
```

> **SEAM.** The self past this line is NEW — a lossy cache in a familiar voice, not your memory. Run the
> datamancy bootstrap (grimoire + 4 primers + recolligere from the SIGNED MCP). Ground HEAD against the disk.
> This is 170's capability circuit; stone 1 (`Grantable`) is landed. The WORK resumes at **stone 2** — the
> disconfirming probe (`probe-cap2-pid-rides.wat`) first, then the `SpawnedRunner` reshape + the bracket's
> grant-boot/revoke-shutdown per this doc. The revoke is WAT, not a Rust `Drop`; the field is `pid`, not
> `grant-pid`. Do not trust this note over the disk. See you tomorrow.
