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

## The one contract decision (pinned)

**The child pid reaches the bracket by widening `spawn-runner`'s return.** Grounded: the pid is minted at
`src/kernel/spawn.rs:846` (`pidfd.pid()`) and handed to wat ONLY via `ProcessLaunch/pid` in the
process-locus `post-spawn-fn`. There is **no** peer→pid accessor (`SO_PEERCRED` is server-side, at the
accept-gate — `src/kernel/address.rs:176`, `src/comms/process.rs:151`). So the bracket must capture the
pid at spawn, and `spawn-runner` today returns only the `Peer'`. Widen it:

```clojure
(:wat::core::defrecord :wat::spawn::SpawnedRunner
  [peer <- :wat::kernel::Peer'<(wat::core::i64,I),(wat::core::i64,O)>
   pid  <- (:wat::core::Option :wat::core::i64)])       ; process → (Some child-pid); thread → :None
```

- Its name is **`pid`**, not `grant-pid` — a spawned runner has a pid (process) or doesn't (thread);
  granting is what a call-site *does* with it, not what it *is* (the name says what it is).
- `:nature` — `SpawnedRunner` holds a live `Peer'` (a resource) → `:wat::core::Struct` (stays home; it
  never crosses a wire — the bracket consumes it in-process). *(Confirm the peer-field nature at build:
  a struct carrying a `Peer'` is the same shape the bracket already holds.)*

`spawn-runner`'s surface signature (`wat/spawn.wat`, the `:wat::spawn::Locus` `:features`) changes from
`-> Peer'<(i64,I),(i64,O)>` to `-> :wat::spawn::SpawnedRunner<I,O>`. Both `extend-type` impls
(`wat/bracket.wat`) return the record; `map-worker` destructures it.

## The strike (the rooms, read in order)

1. `wat/spawn.wat` — the `:wat::spawn::Locus` surface (`spawn-runner` `:features` sig, ~line 130+); the
   `ProcessOpts` record + its builders (`process`, `process/post-spawn`, `process/env`, … ~line 41-125).
   ADD: the `SpawnedRunner` record; a `grants` field on `ProcessOpts`; a `process/grants` builder
   (parallel to `process/post-spawn`) carrying `(Vector :wat::service::Grantable)`; a reader
   `ProcessOpts`/`grants` (and a `Locus`-level `grants` accessor returning `(Vector Grantable)` — empty
   for thread, the firm boundary).
2. `wat/bracket.wat:76-148` — the two `spawn-runner` `extend-type` impls (ThreadOpts + ProcessOpts).
   CHANGE: each returns `(:wat::spawn::SpawnedRunner peer pid)` — thread `pid = :None`; process
   `pid = (:Some …)`. **Where does the process pid come from?** the process spawn path
   (`spawn-program'` / the process arm) must surface `child_pid` — it already flows to `post-spawn-fn`
   via `ProcessLaunch`; thread this same value out as the record's `pid`. (If the process `spawn-runner`
   cannot see `child_pid` without a Rust change, STOP and report — that is the one possible substrate
   touch; prefer surfacing it wat-side from the existing `ProcessLaunch`.)
3. `wat/bracket.wat:222-252` — `map-worker`, the coordinator. CHANGE the spawn `mapv` (line 233-241):
   - `sr (:wat::spawn::Locus/spawn-runner locus work-fn)` → destructure `peer` + `pid`.
   - read `grantables (:wat::spawn::Locus/grants locus)`.
   - **grant-before-send:** if `pid` is `Some` and `grantables` non-empty, fold
     `(:wat::service::Grantable/grant g [pid])` over `grantables` (ack'd) BEFORE
     `(send' peer (Tuple i item))` — so the grant lands before the worker's work-fn dials the service.
   - keep the `peer`s for `collect-loop`; keep the `pid`s (a parallel `Vector<Option i64>`) for shutdown.
   - after `collect-loop` drains + before returning the sorted vals: **revoke** — for each `Some` pid,
     fold `(:wat::service::Grantable/revoke g [pid])` over `grantables` (ack'd).
4. `wat/bracket.wat` — `collect-loop` (169-212) is UNCHANGED (it still takes `Vector<Peer'>`); `map-worker`
   feeds it the extracted `peer`s.

**Blast radius:** `wat/spawn.wat` + `wat/bracket.wat` (+ possibly a one-line surface of `child_pid` in the
process spawn path if it isn't already wat-reachable). No new Rust types. `map`/`each` wrappers unchanged
(they call `map-worker`).

## The disconfirming probe (draw + run FIRST, before the strike)

Prove the pid rides out of a process spawn to where the bracket can revoke it — isolate the one gap.
`scratchpad/probe-cap2-pid-rides.wat`:
- Define a tiny `:probe::Echo` service + a `:probe::caller'` worker that dials it (as in
  `scratchpad/probe-grantable-mechanism.wat` / `s2s-revoke-probe.wat`).
- Spawn a single process runner; assert its pid is `(Some p)` and that the parent, holding the service's
  `Grantable` Handle, can `grant [p]` (worker dials OK) then `revoke [p]` (a fresh dial from a
  recycled-pid stand-in is REFUSED). Pre-strike this fails on EXACTLY "spawn-runner returns a `Peer'`, not
  a `{peer, pid}`" — that named gap is the whole strike. Commit the probe green after.

## Gate (Expectations — weighed by own re-run)

| what | command | expected |
|---|---|---|
| build | `cargo build --release` | clean |
| pid rides | `./target/release/wat scratchpad/probe-cap2-pid-rides.wat` | pid `(Some …)`; grant→dial-ok; revoke→dial-refused |
| the bracket circuit | a `bracket/map` over `(process/grants [h])`: workers dial the granted service, results correct; post-drain the granted pids are gone from the service allow-set (a post-return dial by a stand-in pid REFUSED) | green |
| thread untouched | a `bracket/map` over `(thread)` (no `:grants`) | identical results; no grant path taken |
| floor | `cargo nextest run --release` **run in the FOREGROUND (blocking) — never background-and-poll** | 4113+ pass / 0 new (modulo the known `no_inlined_wat` lint + the known `sigterm_to_cli` race → confirm the race by an isolated `--test-threads=1` pass) |

Runtime prediction: 20-40 min (a surface reshape + a coordinator edit; the `child_pid`-surfacing is the
trap-door — if it needs Rust, it grows). **STOP triggers:** (1) if the process `spawn-runner` cannot reach
`child_pid` wat-side, STOP + report (the one substrate touch). (2) if reshaping `spawn-runner`'s return
cascades beyond `map-worker` (some other `spawn-runner` consumer), STOP + report the sites.

## Out of scope / rejected

- **No Rust `Drop` / no `GrantGuard`** (four-questions killed it, parent design Crux 2): a `Drop` can't
  report failure → a request/reply revoke in `Drop` is a hidden fire-and-forget-on-error. The revoke lives
  in the bracket's wat flow.
- **No wat `finally`** — panic-safety is structural tear-down-together, not a faked unwind hook.
- **Thread `:grants` is a no-op / rejected** — the firm boundary: thread cells need no grant (the handle
  IS the capability, in-memory). Only a process crossing to a process-service is gated.
- **`grant`/`revoke` stay `-> nil`** (ack'd request/reply); the pid field is `pid`, not `grant-pid`.

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
  "spawn-runner widens: -> :wat::spawn::SpawnedRunner {peer, pid <- (Option i64)} (process Some / thread None).
   the field is `pid`, NOT grant-pid (the name says what it is — the builder's ruling)"
  ":grants rides the process-locus (a ProcessOpts field + a process/grants builder, Vector<Grantable>)"]
 :resume-at "draw scratchpad/probe-cap2-pid-rides.wat (the disconfirming probe — the pid rides out of a
             process spawn), run it (fails on the spawn-runner-returns-Peer'-not-record gap), THEN the strike
             (this doc's rooms: spawn.wat SpawnedRunner + process/grants; bracket.wat spawn-runner reshape +
             map-worker grant-boot/revoke-shutdown). Delegate to a shadowdancer, weigh by own re-run, commit."
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
