# Slice 8 — services universe-resident: the deadlock class dies here

> The heart-kill of the forever-fix. The ambient-stdio-ProcessPeer deadlock
> lived in the stdio services' handle-passing architecture; this slice makes
> that architecture unrepresentable. Sequencing per the re-order: 5.1 done
> (channels comms-backed) → THIS → Slice 6 (typed_channel deletes, tenantless).

## The disease (grounded)

Today's stdio services (`wat/kernel/services/{stdin,stdout,stderr}.wat`, ~900
lines) are wat programs that do their own MULTIPLEXING:

- The `Add` event carries **`data-rx <- Receiver<Event>` and
  `ack-tx <- Sender<nil>` as message FIELDS** — handle-passing IS the
  registration protocol (the exact uniform-portability violation 254.1 found
  and deferred to here).
- Each service maintains a routing Vector + selects by index over N per-thread
  channels + a control channel.
- The forked-child story inherits/rebuilds this plumbing across the fork
  boundary (the fd-7 business) — the ambient-stdio ProcessPeer deadlock's home.

## The cure (the 214 DESIGN's own principle, completed)

**Layer 2 (DESIGN.md:579): a service IS a peer.** Spawn it; talk to it with
`send'`/`recv'`. Multi-user dispatch happens INSIDE the service program. We
complete that principle with the stdio trio's specific shape:

### The TaggedEvent service shape

The service program is a **pure portable-message loop** — `peer<Req, Rep>`
where every request is TAGGED with its client's ThreadId and every reply is
tagged with its recipient:

```
service loop:  (recv' self) → match the tagged event → do the owned-resource
               work (write fd 1 / read fd 0) → (send' self tagged-reply)
```

- **No handles in messages** — Req/Rep are records of scalars (ThreadId +
  String). Fully portable; the 254.1 gate passes without exception.
- **No routing table, no select-by-index, no Add/Remove** in the wat program —
  the ~900 lines of wat multiplexing collapse to three small pure loops.
- **The universe does the fan-in/fan-out** (Rust `thread_io` layer): per-thread
  client pairs (already installed per-thread by the spawn orchestrator, already
  comms-backed post-5.1) fan into the service peer's input; replies fan out by
  tag to the requesting thread's reply channel. The existing bridge
  architecture (`spawn_*_bridge`) IS this layer — it survives; only the
  wat-side handle-passing dies.
- **Mini-TCP preserved**: println's ack = the tagged reply (write-confirmed
  before the caller proceeds — panic-ordering guarantees keep holding);
  readln's reply = the line, routed by tag (the reply-routing proof case).

This is also the 256 endgame's first production instance: the CONTROL PLANE
(what the service does per event) is wat; the TRANSPORT (fan-in, tag-routing,
fd ownership) is the universe. The program never knows its transport — the
truest reading of universe-residency.

### The child-universe story (the actual deadlock kill)

A forked child does NOT inherit service plumbing. **Each universe boots its own
service peers on its own fd 0/1/2** at startup (the same way the parent did).
The fd-7 inheritance dance — the deadlock's mechanism — has no successor: there
is nothing to inherit because services are universe-LOCAL by construction. The
parent talks to the child via the child's stdio pipes (the IPC triangle,
recovery doc §13), exactly as before; the child's INTERNAL stdio routing is its
own universe's business.

## The stones

- **8.1 — StdOutService reborn** (the template): Req/Rep records minted; the
  wat service rewritten as the pure handle fn driven by the Rust service loop
  (`spawn_service_peer`, src/services/peer.rs — the 8.1w/8.2w lift made the
  loop universe-resident in Rust; the wat half is the handle alone);
  the Rust fan-in/fan-out (bridge layer) re-pointed; `println` routes through
  it end to end. Probe: println round-trip + ordering + the panic-envelope
  ordering test stays green. stderr follows as 8.1b (same shape, fd 2).
- **8.1w — LIFT the perfected forms to the warded home `src/services/`**
  (builder directive 2026-06-07: *"we should lift these perfected forms to a
  warded home before we close out"*). The 8.1 build necessarily lands its new
  forms (the service loop, the input enum, the reply registry, the boot
  spawn) inside `src/thread_io.rs` — a CONDEMNED quarry whose old-stack guts
  Slice 6 deletes. Immediately after 8.1 scores: mint `src/services/`
  (the 214 DESIGN's reserved Layer-2 home), lift the new machinery + the
  surviving caller path into it, ward it (vigilia → stamp), and leave
  thread_io.rs holding ONLY condemned material so Slice 6's deletion is a
  clean rm. **8.1b/8.2 then build IN the home** — the quarry never grows
  again.
- **8.2 — StdInService reborn**: the reply-routing proof (readln returns the
  RIGHT thread's line under concurrent readers). Probe: two threads readln
  concurrently; each gets its own line.
- **8.3 — the child-universe boot**: forked children construct their own
  service peers (no inherited plumbing); **un-ignore the ProcessPeer-deadlock
  test** — the class's tombstone; the arc-170 ignore-drawdown advances.
- **8.4 — `:wat::services::start`** (Layer 2 user surface): the generic
  service-as-peer sugar (the DESIGN's snippet) for USER services; the arc-203
  ServiceWithProvisioning pattern rebuilt on it.

## Four questions (the shape)

- **Obvious?** YES — a service is a peer; a request is a tagged record; the
  loop reads like the OTP gen_server it converges on.
- **Simple?** YES — three pure wat loops + one Rust routing layer that already
  exists (the bridges); handle-passing deleted, not patched.
- **Honest?** YES — the deadlock dies by making its mechanism (inherited
  handle plumbing) have no successor, not by guarding it; the 254.1 portability
  finding closes at its root.
- **Good UX?** YES — println/readln/eprintln keep their exact surfaces; user
  services gain the same one-canonical pattern.
