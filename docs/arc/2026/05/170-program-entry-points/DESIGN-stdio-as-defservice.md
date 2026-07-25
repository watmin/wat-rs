# DESIGN — stdio as `defservice` (arc 170)

> **✅ STATUS: COMPLETE (2026-07-25) — all four moves landed, stdio CLOSED.** Phase 1 (`6d2fa8c9`,
> 3 primed defservices) → flip the 5 verbs (`e38db291`) → write-batched fragment + `readln` cause
> (`a66066ed`) → Phase 3 (`eae45001`, hand-rolled path annihilated −541 + `'` names reclaimed). Floor
> **4162/0**. The 3 streams are `defservice`s (fd in `:ephemeral`, born inside `:init` from a pure
> fd-number seed via whitelisted `from-fd`); the bespoke `spawn_service_peer` machinery is gone.
> Realizations: `VT SE OPPVGNET DISCIPLINAM EGREDITVR` + `EX CINERIBVS SVRGIMVS` (170 realizations).
> This detour's purpose — **unblock telemetry proper** — is achieved. Retained as the record of how.

> **Builder ruling (2026-07-24):** *"services are the holders of protected resources — std{in,out,err}
> are protected resources."* → the three stdio streams become **defservices**, not the bespoke
> hand-rolled service peer. *"the only things that matter from callers are `(readln)` / `(println d)` /
> `(pprintln d)` / `(eprintln d)` / `(epprintln d)` … we just need readln/println to swap who they call …
> build the primes … move to them … delete the hand-rolled non-prime … reclaim their names."*
>
> **Namespace: `:wat::kernel::` (builder corrected an earlier `wat.core/` slip).** The five verbs keep
> their names AND their kernel namespace — this is a **pure impl-swap**, no caller-site codemod.

This is arc-170 territory (program entry / loci); the stdio work drifted into 278 and returns home here.

## 0. The caller surface (unchanged — the whole point)

Five kernel verbs. After every phase below they are byte-identical at the call site; only *who they call*
changes. 470 `println` + 72 `readln` + 17 `eprintln` + 15 `pprintln` + 2 `epprintln` call sites stay put.

```clojure
(:wat::kernel::readln)          ;; a macro → readln' (stdin.wat:127)
(:wat::kernel::println   data)  ;; native eval fn (runtime.rs:4954 → eval_kernel_println)
(:wat::kernel::pprintln  data)  ;; native eval fn (runtime.rs:4955)
(:wat::kernel::eprintln  data)  ;; native eval fn (runtime.rs:4956) — the TERMINAL death channel
(:wat::kernel::epprintln data)  ;; native eval fn (runtime.rs:4957)
```

## 1. What is hand-rolled today (the seam the flip rewires — grounded file:line)

- **`freeze.rs`** — at startup spawns three services via `spawn_service_peer`, holds each one's `input_tx`
  as a runtime global (`stdin_ctrl`/`stdout_ctrl`/`stderr_ctrl`, freeze.rs:319-321) + three join handles
  (:123/126/129), joined at teardown (:196-208).
- **`src/services/peer.rs` `spawn_service_peer`** — a bespoke universe-resident actor loop with a
  `HashMap<ThreadId, ServiceReplySender>` **ReplyRegistry** and a Rust-internal
  `ServiceMsg::{Req, Register, Deregister}` enum (never a wat message).
- **`src/services/client.rs` `ThreadIO`** — per-thread reply receivers (`stdout_reply_rx`/
  `stderr_reply_rx`/`stdin_reply_rx`), populated by a `Register(tid, reply_tx)` at thread spawn.
- **`src/services/verbs.rs`** — the five `eval_kernel_{println,pprintln,eprintln,epprintln,readln}` fns send
  a `Req(value)` on the global `*_ctrl`; the service writes the fd and routes the ack back to the calling
  thread's reply_rx **by tid**.
- **`wat/kernel/services/{stdin,stdout,stderr}.wat`** — each collapsed (Stone 214.8.x) to ONE pure handle
  fn `(handle [req resource] -> Rep)` + hand-authored `Req`/`Rep` structs; the Rust loop drives them.

Every one of these is exactly what `defservice` now generates: the serve loop, Op/Reply wrapping, the
per-caller reply routing (via `connect'`/`listener'` fan-in), and the Request/Response records.

## 2. The primed model (each stream → one defservice)

A `defservice :satisfies <Surface>` where the surface is `:nature :wat::kernel::Peer'` (arc 293 Path B:
a dialed `Peer'` of a `:satisfies` service *is* the surface, intrinsically). Grounded against the modern
exemplar `wat/query.wat` + `tests/services/probe_arc209_c3_defservice_client_face.wat`.

- **`StdOutService'` / `StdErrService'`** — `:ephemeral [out <- :wat::io::IOWriter]`, op
  `(write-line [self req] -> WriteResponse)`. Both are already plain write-line serializers
  (`{stdout,stderr}.wat` = `IOWriter/writeln` + ack; confirmed `stderr.wat` is NOT the death channel).
- **`StdInService'`** — `:ephemeral [in <- :wat::io::IOReader]`, op `(read-line [self req] -> ReadResponse)`
  where `ReadResponse` carries an **explicit `Eof` variant** — the no-hidden-failures upgrade (today's stdin
  EOF is `assertion-failed!` → panic-kills-the-loop; as a defservice it becomes a matchable value, R55/R57).

**The fd lives in `:ephemeral`** — an impure resource, thread-owned, and by the arc-293.W wall it
**cannot** be placed in `:durable` or cross the wire (correct-by-construction; this is the exact rule the
`probe_arc254_channel_payload_portable` thread-tier exemption test guards).

### Fan-in (grounded from the client-face exemplar)

`(<svc>/start :locus (:wat::spawn::thread) …)` → a `Handle` carrying an `Address'`. **Any** caller
`(connect' (Handle/addr h))` → `ConnectOutcome::Connected p` → its own `Peer'`; the service serializes
(the actor is the mutex). N callers = N dials to one held `Address'`.

Mapped to stdio: **freeze bootstraps the three services and holds their three `Handle`/`Address'`
globally**; each thread `connect'`s its own `Peer'` to each (cached in `ThreadIO`, replacing the current
`Register`/reply_rx); the five verbs route to the current thread's stdio `Peer'` via the surface op.

## 3. `eprintln` — the terminal death channel, split

`eprintln` (R51/R55) = write a final EDN line to stderr **then terminate** (`panic_any`). The split:
`StdErrService'` does the **write** (serialized, identical to stdout); the **terminate** stays the verb's
own separate act (write via the service, then `panic_any`). The service is a write-serializer; the death
rides the verb, never the service loop.

## 4. Sequence (the 4-move de-prime)

1. **Build the primes + prove** — the three defservices + freeze bootstrap, **coexisting** with the
   hand-rolled path (both alive; nothing flipped yet). Proof test: `write-line`/`read-line` via the primed
   peer round-trips, EOF surfaces as `Eof`.
2. **Flip the five verbs** — `eval_kernel_{println,…,readln}` route to the current thread's primed stdio
   `Peer'` instead of the `*_ctrl` Req path. Names + kernel namespace unchanged.
3. **Delete the hand-rolled non-prime** — `spawn_service_peer`, `ServiceMsg`, the three old
   `Std*Service/handle` fns + Req/Rep structs, `Register`/`Deregister`, the `ThreadIO` reply_rx, the
   `*_ctrl` globals, the freeze bootstrap of the old services.
4. **Reclaim names** — the freed internal names (`StdInService` etc. → the primed service names drop `'` if
   primed; the caller verbs already keep their plain kernel names).

## 5. Four questions

- **Obvious? YES** — the builder's principle made literal: a protected resource lives behind a service.
- **Simple? YES** — three defservices *delete* the bespoke loop + the manual ReplyRegistry + `ServiceMsg` +
  hand-authored Req/Rep; the generated serve loop replaces the hand-rolled one.
- **Honest? YES** — stdin EOF becomes a matchable `Eof` value (not panic-kills-loop); the fd in `:ephemeral`
  cannot leak across the wire (293.W, uncompilable to get wrong); `eprintln`'s death is separated from the
  write-serializer.
- **Good UX? YES** — the five caller verbs are unchanged; the ambient feel is preserved; the protected-
  resource discipline lives underneath.

## 6. The gating disconfirming probe — RESOLVED ✓ (2026-07-24)

The one stdio-specific unknown was **one always-on, freeze-lifetime defservice dialed by N *concurrent*
threads, with correctly-routed serialized replies** (the exemplars are single-threaded sequential). Probe
built + run: `wat-scripts/scratch-pad/probe-stdio-concurrent-dial.wat` — one thread-tier `defservice`
counter, started once, dialed by **3 concurrently-spawned worker threads** (`spawn-program' (thread)`),
each `connect'`ing its OWN `Peer'` and issuing 4 `increment`s; a mis-routed/garbled reply fails the typed
match (raise), a lost reply is `RecvOutcome::Lost` (raise). **Result: 15/15 runs GREEN (6 by the
orchestrator's own hand), `workers-ok=12 final-counter=12`, exit 0** — no cross-talk, serialization held.
`Address'` is shareable to threads (thread tier = shared memory). **Verdict: the build is mechanical, no
gap.**

## 7. Resolved unknowns (2026-07-24 — investigation + orchestrator weigh)

- **(a) Concurrent N-client dial — PROVEN** (§6). 15/15 green, 6 the orchestrator's own re-run.
- **⚠ (b-CORRECTION, 2026-07-24 — the impure-init-arg assumption was FALSE; STOP hit + pivoted).** §2's
  original assumption "a defservice `:init` can take an impure `IOWriter`/`IOReader` arg (thread tier =
  shared memory)" is **WRONG**. The defservice macro generates the service's `Admin` enum as
  unconditionally `:wat::enum::Pure` (`service.wat:681-687`) and ships the `:init` params inside
  `Admin::Init` (must be wire-portable for the process-tier fork). `validate_aggregate_containment`
  (`check.rs:14234-14252`) statically rejects an impure field in a `Pure` enum, **tier-blind** — so NO
  defservice `:init` param may be an impure resource. **The pivot (builder-ruled): birth the fd INSIDE
  `:init` from a PURE SEED** — the fd *number* (a pure `i64`) rides `Admin::Init` fine, and a new whitelisted
  primitive materializes the handle inside the init body:
  - **`:wat::io::IOWriter/from-fd [fd <- :i64] -> :wat::io::IOWriter`** + **`IOReader/from-fd`** — both
    `#[restricted_to(":wat::kernel::")]` (building a raw-fd handle is privileged; only kernel-internal wat
    calls them). Each **`dup(fd)` first and owns the dup** (io.rs:451: `from_raw_fd` owns → Drop closes;
    dup keeps the service's handle independent so dropping it never closes the process's real 0/1/2). Thin
    wrapper over the existing `PipeWriter::from_owned_fd(OwnedFd::from_raw_fd(…))` pattern (io.rs:1314,
    process/verbs.rs:311-315).
  - Each stdio defservice `:init [record <- Record  fd <- :i64]` → body
    `(State :durable record :out (:wat::io::IOWriter/from-fd fd))`. `Admin::Init` carries only pure
    (Record + i64) → containment passes.
  - The **thread-local `AmbientStdio` caveat dissolves**: an fd *number* is process-global, so `from-fd(n)`
    on the spawned service thread hits the same fd as the bootstrap thread — no thread-local dependency.
  - **Deferred to the flip (Phase 3), flagged:** test stdout-capture installs a *redirected* `AmbientStdio`;
    `from-fd(raw 1)` bypasses it. Phase 1 coexists (verbs unflipped, tests use the old path) so it's moot;
    at the flip, freeze must seed the primes with the *ambient's* fd number (`as_raw_fd` of the redirected
    handle), not raw 1. (mirrors sift's connect'-inside-`:init`: the resource is born inside init from a
    portable seed — here the seed is the fd number instead of an `Address'`.)

- **(b) Freeze-time integration = (b3): a NEW BRIDGE, but every primitive already exists.** `<svc>/start`
  is a **kwargs-defn** (`[& [locus <- :wat::spawn::Locus ~@init-param]]`, `service.wat:1588`) → not
  directly `apply_function`-able from Rust, so start it as a FORM via the existing **`eval_in_frozen`**
  bridge (`freeze.rs:967`) at the current bootstrap slot (where `spawn_service_peer` runs today, before
  `:user::main`); capture the returned `Handle`. Hold the three `Handle`/`Address'` in a new runtime global
  mirroring the existing `RuntimeServices`/`set_runtime_services` pattern (`freeze.rs:117+`). The five
  verbs (`services/verbs.rs`) `connect'` a per-thread client `Peer'` cached in `ThreadIO` (replacing the
  `*_reply_rx`) + drive `send'`/`recv'` — the kernel connect'/send'/recv' evaluators are Rust-callable.
  **Circular-dependency check: BROKEN/safe** — 0 stdio verb refs in `kernel/{spawn,listener,peer,address}.rs`
  + `wat/{spawn,bracket}.wat`; the start→`Locus/launch`→`spawn-program'`→serve-loop `poll'` path is
  stdio-free (a defservice can be started before stdio exists).
- **(c) EOF-as-variant + blocking read-line — FEASIBLE.** The generated serve loop dispatches ops
  synchronously one-at-a-time (`service.wat:1047` `poll'`, op arm `:988-989`); a handler that blocks on the
  fd holds ONLY its own service's loop — and stdin/stdout/stderr are **three separate defservices = three
  separate loops**, so a thread blocked in `readln` never blocks any thread's `println`. stdin is one-reader,
  so a blocked `read-line` serializes readers by nature. Today's EOF (`stdin.wat:75-98` → `assertion-failed!`
  → panic-kills-the-loop, `peer.rs:76-133`) becomes a matchable `ReadResponse::Eof` variant:

  ```clojure
  (:wat::core::defenum :wat::kernel::StdIn'::ReadResponse :wat::enum::Pure
    :Line            [line <- :wat::core::String]
    :Eof                                                    ;; the matchable value replacing panic-kills-loop
    :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])
  ```

**Grounded seam (file:line read):** the modern `defservice` model (`wat/service.wat`, `wat/query.wat`); the
client-face `start → Address' → connect' → Peer' → op` (`probe_arc209_c3`); the current hand-rolled seam
(`freeze.rs`, `services/{peer,client,verbs}.rs`); the three stdio service shapes; `stderr.wat` is a
write-serializer, not the death channel.

**The one real new-work leg** (everything else is reuse): the verb-side per-thread `connect'` + `ThreadIO`
peer cache replacing the `*_reply_rx`/`Register` registry. That is the heart of the build.
