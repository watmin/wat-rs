# BRIEF — the CALL CONTEXT: a handler is told who is calling, and every op arm keeps working

> **Design + rulings:** `DESIGN-STONE-the-call-context.md`. Read its § "SCOPE CUT / OVERRULED" and
> § "THE SHAPE, RULED" first. **Do not re-derive them.**

## The work, one paragraph

A `defservice` op handler receives `(state, request)` and nothing else (`wat/service.wat:1031-1035`
binds `let-bindings` = `[s-binder state]`). It cannot tell which client is calling, so a multi-tenant
service cannot select a caller's state even when that state exists. This strike adds a **third,
OPT-IN arm parameter** — `[s ctx req]` — carrying a five-field context, and mints the **stable
caller id** the context needs inside the generated serve loop. **Existing `[s req]` and `[s]` arms
must keep working untouched.**

## The ONE contract decision, pinned

**The caller id is a monotonic `i64`, minted in the serve loop, NEVER reused within the service's
life.** Not a `Uuid` (heavier, and unnecessary — ctx already carries `namespace`, so
`(namespace, conn-id)` is globally unique). Not a position: `idx` is a position into `selectables`
and every eviction is `remove-at`, so positions shift and a position-keyed lookup silently hands one
tenant another's state. **The counter is threaded through the serve loop as pure state** — no clock,
no entropy, no global.

## The ctx floor — five fields, all pure scalars

| field | source | how |
|---|---|---|
| caller id | the serve loop's connection table | the monotonic `i64` above |
| namespace | the service fqdn | **compile-time literal** — the macro has `fqdn-kw` (`service.wat:85-93`) |
| operation | the op arm's name | **compile-time literal** — the macro has `op-str` (`:988`) |
| request id | minted per call | one `Uuid/v4` in the serve loop |
| start-ns | stamped per call | one clock read in the serve loop |

`Uuid` is in `is_pure_type`'s pure-scalar list, so **the record is pure** — wire-crossable,
`:durable`-legal. It is *produced* at an impure boundary (the serve loop) and *consumed* by a pure
handler, which handler purity already enforces.

## Read in order

1. `wat/service.wat:981-1035` — `serve-op-arms`, the per-clause fold. `op-str` (`:988`), `is-internal`
   (`:989`), `param-vec` (`:990`), and `let-bindings` = `[s-binder state]` (`:1031-1035`). **This is
   where the third binder is added.**
2. `wat/service.wat:1048` — `(:wat::core::if is-internal …)`. **THE PRECEDENT: the macro already
   branches arm shape.** You are adding a second axis (param COUNT) to a dispatch that exists.
3. `wat/service.wat:1249` — the generated `serve` fn's parameter list (`selectables <- …`). The
   counter is threaded here.
4. `wat/service.wat:1263-1266` — the `Connection` arm: `(conj selectables peer)`. **Mint the id here.**
5. `wat/service.wat:1351-1364` — `Closed` / `Lost`: `(remove-at selectables idx)`. The id leaves with
   its peer.
6. `wat/service.wat:1118`, `:1135`, `:1221` — the reply sends via `(nth selectables idx)`. These must
   still reach the PEER after the shape change.
7. `wat-tests/service-init-parity.wat:34-37` — a live 2-param arm that MUST remain untouched and green.

## Implementation sketch

**Carry the id WITH the peer, never beside it.** A parallel `conn-ids` vector desynchronises the
moment a timer is removed from one and not the other (the internal-op arms remove fired alarm timers
from the same `selectables`; grep `remove-at selectables`).

> **★ FRAMING CORRECTED 2026-08-09 — "the positional-identity DISEASE" oversells it.** This brief
> called `idx` a disease the design exists to kill. That is wrong about the existing code, and the
> correction matters because it changes what the counter is *for*.
>
> **Within one round, `idx` is a complete and correct identity.** Select fires, you get a seat
> number, and you can reply to that peer (`nth selectables idx`) and evict it (`remove-at`) with no
> ambiguity. **Every existing use in the serve loop consumes the idx inside the same arm that
> received it** — all the reply sites, all five `remove-at` sites. All correct.
>
> It is not an identity **across** rounds, because the vector mutates between them. And it never was
> a design shortcut: `poll` registers self-peer at 0, listener at 1, clients at 2..N+1 and returns
> `index.0 - 2`, so `idx` is **crossbeam's registration position** flowing up through our own
> `ReceiverIndex(usize)`. A set of anonymous channels has nothing else to offer — the positional
> vocabulary was inherited from the transport, not chosen.
>
> So the counter does not cure a bug. **It adds a second, longer-lived name for the first consumer
> whose lifetime exceeds a round** — the connection-scoped world, written in round N and read in
> round N+500. Both names are legitimate and they compose: `idx` finds the connection *this* round;
> `conn-id` is how anything durable remembers it. The eviction path uses both, in that order —
> `(first (nth selectables idx))`.
>
> The stability gate still earns its place, for the sharper reason: not "positions are bad," but
> "**this** consumer outlives the position."

```clojure
;; selectables element becomes a pair-like carrying (id, peer) — id travels with its peer,
;; so remove-at drops both together and no index arithmetic can separate them.
;; The serve loop threads `next-id` as pure state alongside `selectables`.
Connection peer  ->  conj selectables (pair next-id peer),  recur with next-id+1
Closed/Lost idx  ->  remove-at selectables idx              (the id goes with it)
Message idx msg  ->  the id is (fst (nth selectables idx))  — available at dispatch
```

**Arity dispatch in `serve-op-arms`:** `param-vec`'s length selects the arm shape.

```
1 param  [s]           internal op        — unchanged
2 params [s req]       public, no ctx     — unchanged (let-bindings as today)
3 params [s ctx req]   public, wants ctx  — bind ctx to a ctor call built HERE at expand time,
                                            splicing fqdn-kw + op-str as LITERALS and reading the
                                            caller id / request-id / start-ns from the loop
```

## Blast radius

`wat/service.wat` + one new type declaration + the acceptance gate. **ZERO existing arms change** —
that is the whole point of arity dispatch and it is STOP-1. Expect the floor at **4380 + your new
tests**.

## ⛔ STOP triggers

1. **STOP-1 — NO EXISTING ARM MAY CHANGE.** 120 arms match `[s req]` across 65 defservice files. If
   your change requires editing even one of them, the dispatch is wrong — **STOP**. The opt-in third
   param is the entire reason this design was chosen over the alternative.
2. **STOP-2 — the id travels WITH the peer.** If you find yourself adding a second vector keyed by
   position, STOP. See the sketch.
3. **STOP-3 — internal (`-`) ops get NO ctx.** They have no caller; their 1-param arm already says so
   and replying from one is already a located assertion (`:1063-1073`). Do not give them a ctx with an
   empty identity — that is the "none means skip" conflation ruled out 2026-08-08.
4. **STOP-4 — ctx MUST STAY PURE. Do not put a `Peer` in it.** The tempting "improvement" is handing
   the handler its caller's peer so it can reply directly. That forfeits wire-crossing and
   `:durable`, and a registered opaque in a pure record is now a load-time error — but design it out,
   do not lean on the wall.
5. **STOP-5 — the ctx TYPE NAME is a placeholder.** An intueri cast is OWED. Use a clear working name,
   say so in your report, and do not treat it as ratified.
6. **STOP-6 — do NOT build lifecycle hooks.** Threading USER state on `Connection`/`Closed`/`Lost` is
   the NEXT stone (the connection-scoped world) and its hook signature is not yet designed. This
   strike mints and carries the id; it does not hand the user an event.
7. **STOP-7 — if the floor moves for any reason other than your new tests, STOP** and report the
   failing test's whole block verbatim plus the exact assertion.

## The acceptance gate — build it, and make it prove the OPT-IN

`tests/services/probe_arc278_call_context.{rs,wat}` — modelled on
`tests/types/probe_arc278_opaque_purity_wall.*` (landed this session; copy its shape).

Three things, and the third is the one that matters most:

1. **A 3-param arm receives a populated ctx** — assert `namespace` equals the service's fqdn,
   `operation` equals the op's name, and the caller id is present.
2. **A 2-param arm in the SAME service still works** — proving opt-in, not migration.
3. **★ THE STABILITY GATE — the id survives an eviction.** Connect three clients, disconnect the
   MIDDLE one, then have a survivor call an op and assert **it still sees its ORIGINAL id**. A
   position-keyed implementation passes every other test and fails only this one. **This is the
   defect most likely to ship green.**

## Weigh

`cargo build --release` → `cargo nextest run --release -E 'test(call_context)'` →
`./scripts/floor.sh` (read the **Summary line**) → `cargo clippy --release --all-targets`.
