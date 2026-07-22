# DESIGN — self-scheduling defservices (item (c)'s substrate stone)

> **Origin:** designing `with-log-sink` (`DESIGN-service-io-budgets.md` item (c)) surfaced that the
> telemetry **span** wants to *buffer* its log writes (drain by timer OR pressure) instead of writing
> one wire frame per `log`. The buffer is invisible plumbing behind the span/ctx the user actually
> holds. Batching-by-time needs a service that can **fire its own timer between client messages** —
> which the generated serve loop cannot do today (it wakes only on client peers). This stone is that
> capability, GENERAL (Erlang/OTP `send_after` from inside a gen_server) — not a one-off flush clause.
> It is the foundation the buffered sink (next stone) transcribes; the mechanism is proven in
> `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` (green, both loci by env-grab).

## ⛔ STATUS (2026-07-21) — BLOCKED on a substrate gap; the "homogeneous selectables" premise is FALSE

The strike STOPped (correctly) and was weighed `AD ORACVLVM`. **The load-bearing premise below — "the
serve loop threads one homogeneous vec of connections + timers" — does NOT hold on the substrate:**
- The serve loop multiplexes with **`poll'`** (`wat/service.wat:848`), not `select'`, over
  `clients : Vector<Peer'<Reply,Op>>` (`:552`) — the **unified `Peer'`** accepted-connection opaque.
- `after` returns a **`Timer'<O>`** that fuses only into `Thread'`/`Process'` (`is_peer_tier_head`,
  `check.rs:15533`), **never** the unified `Peer'`; `poll'` rejects any non-`Peer'`. arc-209's
  `Peer'`-unification and arc-292's `Timer'`-fusion **never met** — that seam is the gap.
- The feasibility probe (`wat-scripts/scratch-pad/probe-self-scheduling-loop.wat`) proved `select'`
  over homogeneous `Timer'`s — an **adjacent** mechanism, not the `poll'`/`Peer'` the serve loop uses.
  (Lesson → memory `feedback_feasibility_probe_must_exercise_the_exact_mechanism`.)

**RESUME: resolve the poll'/timer fork FIRST** (the stone-before-the-stone) — then everything below
(Outcome grow, `<service>::Op` superset, `selectables`, the leading-dash marker) builds on it. The
three options (four-question them; each has a load-bearing feasibility question to PROBE on the REAL
path, not an adjacent one):
1. **a `Peer'`-tier timer** — `after` (or a sibling) returns a `PEER_TYPE_PATH` opaque wrapping a
   *sender-less* timer receiver that EOFs on fire → `Closed`; joins `poll'` unchanged (the timer just
   IS a one-shot `Peer'`). Probe: is a sender-less `Peer'` representable, both tiers (crossbeam +
   timerfd)?
2. **heterogeneous `poll'`** — teach `poll'` (both tiers) + its checker signature to multiplex a mixed
   `{Peer' | Timer'}` vec. Probe: the multiplexer branch-per-element + the fired-index→`Message`/
   `remove-at` semantics.
3. **self-peer + timer-driven send** — the timer, on fire, `send'`s the op to the service's OWN
   address (a self-connection, a real `Peer'`); `poll'` receives it normally. Probe: a self-peer + a
   per-timer waiter; reuses `poll'`/`after` unchanged.

Everything below is the SETTLED design *above the multiplexer* (types, superset, marker, dispatch,
UX) — all still valid once a timer can reach the reactor; only the "how the timer joins the set"
premise is open. Kept as-is below.

## The capability (builder-ratified)

A `defservice` can send **itself** a message on a delay. A timer is just a `Peer'<_,Op>` that delivers
one of the service's own `Op` variants; the **existing** serve-loop op-dispatch routes it to its
handler. **Many timers → many actions** (flush, heartbeat, backoff, deadline), armed / re-armed /
one-shot at the author's choice.

- **The internal op is marked by a LEADING DASH.** `(-flush-tick [s] …)` in `:impls` is a
  reactor-internal op: **not on the `:satisfies` surface** (no client can name or call it), and a
  member of the service's own `select'` set. The dash binds visibility to the identifier — an op's
  name and its client-reachability cannot drift apart (a named `:internal-ops` clause would let them).
  A `-`-arm with no surface counterpart is *intentionally* internal; a non-`-` arm with no surface
  match stays a compile error (the typo-guard). **Grounded legal:** a `-FlushTick` enum variant
  constructs + matches + type-checks; a bare `-flush-tick` symbol parses (bare `-` is not a wat
  operator). **Preserved through synthesis:** `kebab->pascal` (`string_ops.rs:336`) drops a leading
  `-` today (`-flush-tick` → `FlushTick`); this stone makes it **prepend** the dash (`-flush-tick` →
  `-FlushTick`) so the marker survives to the `Op` variant.
- **Periodic = explicit re-arm** (arc-292 doctrine: no `tick` primitive; a heartbeat re-arms itself).
  **One-shot = arm nothing** (a `Deadline` fires once). The author's choice, visible in the handler.

## The one contract decision — `Outcome<S,R,O>` (grounded, `--check` green)

A handler schedules a self-message by emitting `Alarm`s in its return. The arm's `op` must be the
service's **concrete `Op`** (to arm a `Timer'<Op>` that fits the homogeneous `select'` set), so
`Outcome` grows a **third type param `O`** (the Op type), used only by the arm-carrying variants:

```clojure
(:wat::core::defrecord :wat::service::Alarm<O> [after <- :wat::time::Duration  op <- :O])

(:wat::core::defenum :wat::service::Outcome<S,R,O> :wat::enum::Pure   ;; was <S,R>
  :Reply         [state <- :S  reply <- :R]                          ;; existing — client op, replies
  :Stop          [state <- :S  reply <- :R]                          ;; existing — reply, then stop
  :NoReply       [state <- :S]                                       ;; NEW — self/cast op, no client to reply to (OTP {noreply,S})
  :ReplyAndArm   [state <- :S  reply <- :R  arms <- :wat::core::Vector<wat::service::Alarm<O>>]  ;; NEW
  :NoReplyAndArm [state <- :S  arms <- :wat::core::Vector<wat::service::Alarm<O>>])              ;; NEW
```

- **`NoReply` back-fills OTP.** `Outcome`'s comment (service.wat:44) already claims to re-derive OTP's
  `{reply,R,S} | {noreply,S} | {stop,…}` — but `noreply` was missing. Adding it completes the mirror.
- **Migration ≈ zero.** Grep: `Outcome<` appears at **3 sites, all in `service.wat`** (the def + 2
  comments); no handler annotates it (arms build `(:Outcome::Reply …)` bare — `O` is phantom for
  `Reply`/`Stop`/`NoReply`, inferred from context). So this is NOT a corpus migration / codemod — it
  is a localized grow of the def + the macro binding `O` to the synthesized `Op` type it already owns.
- **Type-safe, no Value-erasure.** The arm's `op` is statically `Op`, not an opaque `Value` narrowed
  at runtime — the substrate way (verbosity is the shield; the checker teaches).

## The serve-loop change — `clients` → `selectables` (one vec; the decomplection)

The loop already threads a mutable peer-set (`clients` — grows on `Connection`, shrinks via
`remove-at` on `Closed`). It is generalized, **not** paralleled: `clients` becomes **`selectables`**
— one homogeneous vec (`select'` takes one anyway) holding client connections **and** armed timers.
`clients` was an honest name only while clients were all it held. (Not `handles` — `<S>::Handle` is
already the owner's lineage peer; an accepted connection is not a Handle. `selectables` = what
`select'` watches.)

Serve loop, on `select'` → `Message{idx, op}` → dispatch `op` → `Outcome<State,Reply,Op>`, three
**orthogonal** effects, each keyed to what actually decides it:

| effect | keyed on | Reply / ReplyAndArm | NoReply / NoReplyAndArm | Stop |
|---|---|---|---|---|
| **reply** | the **Outcome variant** | send `reply` to `selectables[idx]` | no send | send, then stop |
| **arm** | the **Outcome variant** (`…AndArm`) | `conj` each `(after own-kind alarm.after alarm.op)` into `selectables` | same | — |
| **remove** | the **op kind** (a fired one-shot) | keep `idx` (a client persists) | remove `idx` **iff the op was a `-`-internal op** (a fired one-shot timer; a client *cast* — a surface op returning `NoReply` — keeps its connection) | — |

- **`own-kind`** for `after` = the service's own tier (env-grab: `(:wat::program::Env/wat.peer-kind
  (:wat::program::env))`, per `timer-env-grab-parity.wat`) → both loci for free.
- **Remove is keyed on op-kind, not the Outcome** — a `-`-internal op came from a one-shot timer
  (remove it, or it leaks dead into the set); a surface op came from a persistent client (keep it,
  even on a `NoReply` cast). The macro classifies (it knows which arms are `-`-marked).
- **Dead connections** still reap via the existing `Closed{idx} → remove-at selectables idx`.
- **`:init` is unchanged** — it returns `State`, not `Outcome`, so it does **not** arm. A service
  arms its first timer in an **op handler** (which returns `Outcome`): the sink arms `-flush-tick` on
  first-push-into-empty; a `start`-op arms a heartbeat. The timer then re-arms itself. (A purely
  autonomous startup timer — armed with no client op at all — would need `:init` to return an
  `Outcome`; that is a separate, larger change and is **out of scope** here.)

## How internal ops join the homogeneous `select'` — the `<service>::Op` superset (four-questions Option 1)

`select'` is homogeneous on the received type, so a `-`-timer must deliver a variant of the *same*
`Op` the clients speak. But `-tick` is internal (not on the surface). Resolution (four-questions:
Option 1 over a wrapper — Obvious *private methods*, Simple *one synthesis + a wire gate*, Honest,
Good-UX — all YES; the wrapper fails Simple on a forever two-level dispatch):

- **The defservice synthesizes `<service>::Op` = `<Surface>::Op` variants + its internal `-`-ops** (it
  already mints `State`/`Record`/`Handle` — one more per-service synthesis). The serve loop dispatches
  **`<service>::Op`** (the superset); `Outcome`'s `O` = `<service>::Op`.
- **The wire stays `<Surface>::Op`** (the client's type — a client can only *construct* surface ops).
  The wire decode targets `<Surface>::Op` and **rejects any non-surface tag** → a client literally
  cannot send `-tick`; that decode **is** the "internals are un-callable" wall. Then embed the surface
  variant into its `<service>::Op` counterpart for dispatch. Timers deliver `<service>::Op` internal
  variants **in-process** (never serialized).
- **`<service>::Op` is INVISIBLE in user forms** (materialized + confirmed): the author declares an
  internal op *by handling it* (`-tick [s]`) and names it *by keyword* (`:op :-tick`); the client sees
  only the surface. Nobody types `<service>::Op` — it is pure internal synthesis.
- **The `Alarm`'s `:op` takes the op KEYWORD** (`:op :-tick`), macro-resolved to the `<service>::Op`
  internal variant (same kebab→pascal + dash-preservation the marker needs). NOT the constructed
  variant `(:svc::Op::-tick)` — that would leak `<service>::Op` into every arming site and kill the
  invisibility. Keyword-`:op` also reads consistently with the arm's own name (`-tick` ⇄ `:-tick`).
- **Internal arms are 1-param `[s]`** (no `req` — no client request). `serve-op-arms` (`service.wat:753`)
  currently assumes `[s req]` (`first (rest param-ch)` → empty on a 1-param arm — the RED gate's first
  failure). It must handle a `-`-marked 1-param arm distinctly: no `req-binder`, no reply-to-a-client,
  dispatch → the `NoReply` family, remove the fired one-shot.



A bare service declares a `-tick` internal op, `:init` arms it, and each `-tick` re-arms via
`NoReplyAndArm` while advancing a durable counter. Assert: after arming once, the counter reaches N
(the timer fired N times, re-armed each time, on the service's own `selectables`), and a client op on
the SAME service still replies (the reactor keeps serving between ticks) — thread ≡ process. At HEAD:
`-tick` cannot be armed (no `Alarm`/`ReplyAndArm`/`NoReplyAndArm`; the serve loop threads `clients`
not `selectables`; the leading dash is dropped by synthesis) → RED. GREEN when the stone lands.
(The mechanism itself is already proven hand-rolled in
`wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` — the exemplar the shadowdancer transcribes
into the generated loop.)

## Scope + sequencing

- **This stone:** `Outcome<S,R,O>` + `Alarm<O>` (grow the def); the macro binds `O` = the synthesized
  `Op`; `clients` → `selectables`; the arm / reply / remove dispatch; the leading-dash marker
  (parse + not-on-surface + preserved through kebab→pascal); the `-` convention documented once (the
  `service.wat` header + `docs/CONVENTIONS.md`). Localized to `service.wat` (+ the Rust the macro's
  `O`-binding / dash-preservation need — grounded during the strike).
- **Then:** the **buffered log-sink** (a self-scheduling actor: buffer + `-flush-tick` latency-flush +
  size-flush) → wire the **span** so `log` enqueues into it (invisible) + `close` flushes → the
  **`with-span'`** nesting ergonomics (fresh uuid, shared sink from lexical scope; flat, no chain).
- **OUT (this stone):** the buffered sink, the span wiring, `with-span'`, the error/unwind close-on-
  *panic* case (the everyday error path is `match` + `log span :error`, no unwind — builder-ruled);
  a `tick`/periodic sugar (arc-292 forbids it — periodic is an explicit re-arm).

## Open cruxes (tracked)

- **CRUX-A** — the exact Rust seam for the macro to bind `O` to the synthesized `Op` in the serve
  loop's `Outcome` typing, and for `kebab->pascal` to preserve a leading dash. Resolve at the strike
  (grounded, RED-gated) — the phantom-`O` construction + a `-FlushTick` variant are already proven
  legal (`--check` green), so the seam is a wiring, not a substrate gap.
