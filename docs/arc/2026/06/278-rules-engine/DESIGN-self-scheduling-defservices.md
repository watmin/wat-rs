# DESIGN — self-scheduling defservices (item (c)'s substrate stone)

> **Origin:** designing `with-log-sink` (`DESIGN-service-io-budgets.md` item (c)) surfaced that the
> telemetry **span** wants to *buffer* its log writes (drain by timer OR pressure) instead of writing
> one wire frame per `log`. The buffer is invisible plumbing behind the span/ctx the user actually
> holds. Batching-by-time needs a service that can **fire its own timer between client messages** —
> which the generated serve loop cannot do today (it wakes only on client peers). This stone is that
> capability, GENERAL (Erlang/OTP `send_after` from inside a gen_server) — not a one-off flush clause.
> It is the foundation the buffered sink (next stone) transcribes; the mechanism is proven in
> `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` (green, both loci by env-grab).

## ✅ CLOSED 2026-08-30 — THE STONE IS LANDED AND GREEN AT BOTH LOCI. The scout update below is ALSO stale; its "prime suspect" was never the cause.

`tests/services/probe_arc278_self_scheduling.rs` is un-ignored and passing, thread and process.
**Nothing in the substrate needed fixing after `1212c9ae` + the `-tick` colon fix.** Read this before
the two sections below it, both of which will send you hunting a bug that does not exist.

The scout update's diagnosis — *"a SUBTLE post-migration RUNTIME bug… Prime suspect: `poll'`'s
reactor-class/homogeneity handling of a `{client + timer}` mix, OR an idx-shift when a fired timer is
removed + a new one armed"* — was **inferred from the symptom `recv': peer closed`, never measured.**
It then hardened into the `#[ignore]` reason and stood for 38 days. Three measurements retire it:

- **`remove-at` is at `wat/service.wat:1591/1594`**, not `958/961`. Those lines drifted ~630 and now
  hold unrelated handle-name minting, so the citation pointed at innocent code.
- **The mechanism reaches `target` at BOTH loci** — `wat-scripts/scratch-pad/probe-self-sched-bisect.wat`,
  `THREAD-tick=3 PROCESS-tick=3`, with the client polling *during* the tick cadence (the fixture's own
  drive shape). `poll'` multiplexes `{client + timer}` correctly.
- **The eviction reproduces with NO timer armed at all** (`F-noTimer-tail`). Self-scheduling was a
  bystander; the timer was never involved.

What actually kept it red: **the FIXTURE released its own service.** `drive-ticker` drove from the
`let`'s BODY — tail position — which ends the scope holding the ticker's `Handle` before the call
runs, and a service dies when its owner's handle is released. The client then met a severed service
and reported `recv': peer closed`. The drive now sits in a binding; see the comment there.

★ **The lesson is not about timers.** A symptom was reasoned into a cause, written into an
`#[ignore]` reason and into this document, and thereafter read as measured. The
`ignore_reason_justified` lint cannot catch this shape: it screens for *promises* wearing a
condition's clothes, and this was a **checkable fact that happened to be false**.

That owner-drop now names itself — `LociDiedError::Severed`, gated by
`tests/services/probe_severed_reaches_the_client.rs` — so the next reader gets the cause in one line
instead of a bisect. The tail-position release itself is a separate, live language question about
handle lifetime, tracked outside this stone; it is NOT a defect in TCO, which is correct and
load-bearing exactly as it is.

## ⛔ SCOUT UPDATE (2026-07-22, post-no-hidden-failures-commit `1212c9ae`) — the `after`-migration is DONE; the death root below is STALE. RE-GROUND before striking.
The bucket-C widening (`1212c9ae`) landed the CHECK (superset-O selectables type-check). Grounding the RUNTIME death (`self_scheduling` ×2, currently `#[ignore]`'d):
- **`after` IS migrated (both tiers)** — `eval_kernel_after` (`runtime.rs:27320-27356`) now builds a UNIFIED `Peer'<nil,O>` (`Peer::from_thread(dead_tx, timer_rx)` thread; a timerfd-backed process peer + wire frame process), NOT a tier-specific `Timer'`. **So the "STATUS (2026-07-21e)" diagnosis below — "`after` builds the WRONG thing" — is STALE / already fixed.** Do NOT re-do it.
- **The serve-loop arms are CORRECT** — `service.wat:948-974` the internal `-tick` arm handles `Outcome::NoReply`/`NoReplyAndArm` (remove-at the fired timer's idx, re-arm via `arm-fn`, NEVER `send'` to the timer); `Reply`/`Stop`/`ReplyAndArm` on an internal op → located `assertion-failed!`. The arm-fold inserts `(after own-kind dur op)`.
- **The death is a SUBTLE post-migration RUNTIME bug** — the service dies mid-tick; the test only sees the CLIENT's downstream manifestation (`self_scheduling.wat:87` client `poll` → `send': channel disconnected` = the service is already dead). The service's OWN death reason is NOT surfaced (it dies in its spawned thread/process). **NEXT (the honest first move): SURFACE the service's death** — a disconfirming probe running the `poll'`-over-{a real client peer + an armed `after` timer} mechanism INLINE (so its death raises directly), adapting the PROVEN hand-rolled `wat-scripts/scratch-pad/probe-self-scheduling-loop.wat` from `select'` → `poll'` (the one delta between GREEN and the crash). Prime suspect: `poll'`'s reactor-class/homogeneity handling of a `{client + timer}` mix (`eval_poll_prime`, `runtime.rs:27500+`), OR an idx-shift when a fired timer is removed + a new one armed. GROUND it by a RUN; do NOT assert. The stone is CLOSE — most is built; this is a focused runtime-debug, not a rebuild.

## ✅ STATUS (2026-07-21e) — the poll'/timer FORK is RESOLVED → **Option ① done RIGHT: implement the timer in the CORRECT location (a unified `Peer'<nil,O>`)**

The fork STOPped correctly last run (weighed `AD ORACVLVM`); this run it was four-questioned, then the
builder cut the framing to its root: **the timer was implemented in the WRONG location.** `after`
(arc-292) produces a **tier-specific `Thread'`/`Process'` `Timer'`**, but the serve loop's `poll'` — and
the whole unified-`Peer'` end-state — runs on the **unified `Peer'`** (`PEER_TYPE_PATH`). The fix is not
a bridge or a workaround; it is to implement the timer where it belongs: **`after` produces a unified
`Peer'<nil,O>`**, so it drops into `poll'` (and `select'`) *by construction* (extirpare — fix the
location, not the case; "we use wat-fix to unfuck the farm — do not fear refactors").

### The four-questions (materialized against the grounded substrate, R17):
- **③ self-peer + timer-driven send — OUT on Honest.** The timer would `send'` its op over a self-wire,
  so the op must be a **decodable** message — but the settled wire decode-gate targets `<Surface>::Op`
  and *rejects non-surface tags*. Delivering an internal `-tick` over the wire forces it to be a
  surface op (or widens the decode-gate to admit internals) → **breaks the "internals un-callable"
  wall**. Serializing an in-process internal op is a lie about what the wall guarantees. (Also fails
  Simple: a self-connection + N per-timer waiters.)
- **② teach `poll'` to swallow the tier-specific `Timer'` — OUT (the builder cut it): the workaround.**
  It routes around a mis-placed implementation — teaching the multiplexer to accept the wrong-location
  timer — instead of fixing where the timer is built. It also entrenches the tier-specific `Timer'` the
  declared end-state (`check.rs:12157-12160` TODO — *"the unified fd-backed `Peer'` end-state makes
  `select'` take ONLY `Peer'`; the tier heads vanish"*) wants deleted.
- **① timer *is* a unified `Peer'<nil,O>` — the winner (Obvious·Simple·Honest·Good-UX all YES).** It is
  the **ratified capability line made literal** (§"The capability": *"a timer is just a `Peer'<_,Op>`
  that delivers one of the service's own `Op` variants"*) AND it *builds* the arc-109/170 end-state (a
  timer is a real unified `Peer'`; the tier-open `Timer'` + its fusion machinery become the half-measure
  it subsumes). `poll'` — the multiplexer that bit us — stays **untouched**; the change is at the
  timer's own construction.

### Grounded this run (`AD ORACVLVM`, file:lines) — the timer is in the wrong location; the correct one is buildable:
- **`poll'` demands the unified `Peer'` at BOTH layers.** Checker: `infer_poll_prime` (`check.rs:12306`)
  → peers-element must be `Peer'<I,O>` (no `Timer'` arm, unlike `select'` at `:12166`). Runtime:
  `eval_poll_prime` (`runtime.rs:27473`) downcasts every peers-element to `PEER_TYPE_PATH`
  (`:wat::kernel::Peer'`) and **errors on anything else**. A `clients` element is a unified `Peer'` —
  an accepted connection (`Listener::accept_as_value`, `listener.rs:478` → `make_rust_opaque(
  PEER_TYPE_PATH, …)`).
- **`after` builds the WRONG thing.** `eval_kernel_after` (`runtime.rs:27221-27273`) returns a
  **tier-specific** `THREAD_PEER_TYPE_PATH` (`:27238`, `kernel::peer::Thread`) / `PROCESS_PEER_TYPE_PATH`
  (`:27271`, `ProcessSelectable::Timer`), typed `Timer'<O>`. It fuses only into `Thread'`/`Process'`
  (`is_peer_tier_head`, `check.rs:15533`), never the unified `Peer'`. So a timer cannot join `poll'` —
  by construction it is the wrong peer type. (arc-209's `Peer'`-unification + arc-292's `Timer'`-fusion
  never met — the timer was built on the wrong side of the seam.)
- **The correct location is BUILDABLE — the unified `Peer'` takes a receiver.** `kernel::peer::Peer`
  (`peer.rs:206`) is `{ tx, rx }` with public constructors from a Sender/Receiver pair, both tiers:
  `Peer::from_thread(tx, rx)` (`:234`) and `Peer::from_socket(tx, rx)` (`:250`). A timer's output
  receiver (`comms::thread::timer(dur, msg)` → `Receiver<Value>`; `comms::process::timer` → the frame
  receiver) is exactly that `rx`. So `after` should build `Peer::from_thread(dead_tx, timer_rx)` /
  `Peer::from_socket(dead_tx, timer_rx)` → wrap `PEER_TYPE_PATH` → typed `Peer'<nil,O>`. The tier is
  still chosen by `after`'s peer-kind arg (env-grab `own-kind`); `poll'`'s reactor-class homogeneity
  check (`runtime.rs:27510-27545`) is satisfied because the timer is built at the service's own tier.

### `after` migration + the `Timer'` retirement (the "unfuck the farm" scope):
Only **7 `.wat` files** call `after` (all tests). Making `after` return `Peer'<nil,O>` (from `Timer'<O>`)
means those `select'`-over-timer tests get `Peer'`-typed timers — `select'` already accepts unified
`Peer'`, so they compose; assert-on-behavior tests stay green. The tier-open `Timer'` type + its
fusion arms (`check.rs:15467-15487`, the `Timer'` element arm `:12166`) become vestigial — retire (or
alias) them as part of the refactor. Decide the exact cut grounded at the strike; a wat-fix codemod
handles any corpus form-change.

### The disconfirming probe → the strike's RED-gated STOP (never assert — R20):
The disconfirming probe walks the REAL `poll'`/`Peer'` path: `after` returns a unified `Peer'<nil,O>`;
it is `conj`'d into a real `poll'` `clients` set; the op is delivered on fire, **both tiers**. **HARD
STOP:** if a unified-`Peer'` timer is not constructible (e.g. the process-tier `timer` receiver can't
be adapted to the socket peer's `Receiver<Value>` decode path) or `poll'` does not deliver its op, STOP
— surface the gap; do not improvise a Value-erasure or a `poll'` bridge.

**The stone, scoped:** move the timer to the correct location (`after` → unified `Peer'<nil,O>`, both
tiers; retire the vestigial tier-open `Timer'`) → then the SETTLED design below the multiplexer
(`Outcome<S,R,O>` grow, leading-dash marker, `clients→selectables`, the `<service>::Op` superset,
keyword-`:op`). Everything below is unchanged and now sits on a resolved foundation.

## ✅ O-SIDE RULED (2026-07-22) — the `<service>::Op` superset (Option A), not `Value` (B)

The I-side homogeneity is DONE — `:wat::core::Never` (the R7-dual bottom) + STEP-0 proven, committed
`a392fd40`. The O-side (a client delivers `<surface>::Op`, a timer delivers an internal op; the
`selectables` vec + the op-dispatch need ONE O) was four-questioned:
- **Rejected — B (`O = Value` top + open `match`):** `Value` **backfires at exhaustiveness.** It erases
  the type → the dispatch needs a WILDCARD → the substrate's FREE coverage check is lost (`service.wat:749`
  — *"COVERAGE IS FREE: a missing `:impl` → non-exhaustive match → compile error"*); a forgotten op
  silently falls through — a hidden failure, in the arc whose law is *no hidden failures*. And it is a lie
  of imprecision: the O is NOT "anything," it is exactly `<surface>::Op | <service>::Internal`; `Value`
  over-widens it. (The `Never`/`Value` symmetry is false — `Never` is the timer input's *precise* type;
  `Value` for O is an over-approximation.) Fails **Honest**.
- **Ruled — A (synthesize `<service>::Op` = surface variants + internal `-ops`; dispatch over it):** the
  PRECISE type, macro-synthesized from the one `:impls` source (not hand-duplication that rots), and it
  KEEPS the free coverage check. It is the substrate's established pattern (per-service Op/Reply synthesis,
  `Handle`→`Capability` embed). The one novel bit: the **re-tag** — a client's wire frame decodes
  self-describingly to `<surface>::Op::X` and must become `<service>::Op::X` for the superset match (the
  client only ever encodes surface ops, so the wall holds; the re-tag is service-side, keyed on the peer's
  expected `<service>::Op`). *Prove the re-tag on the real decode path before the macro.*

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
