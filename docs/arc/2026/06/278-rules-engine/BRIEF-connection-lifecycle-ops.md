# BRIEF — `-on-connect` / `-on-disconnect`: the service is told when a connection begins and ends

> **Design + rulings:** `DESIGN-STONE-connection-lifecycle-ops.md`. Read its § "THE MECHANISM,
> GROUNDED", § "THE ONE CONTRACT DECISION", and § "ONE MAP, NOT TWO" first. **Do not re-derive them.**
>
> The disconnect-reason fork in that stone is **RULED: option (i)** — a fourth context type. The
> four-questions grid: (i) 4/4 · (ii) fails Honest (drops a cause the substrate holds) · (iii) fails
> Obvious+Simple+Honest (the `Option` conflation) · (iv) three separate ops fails Honest, because with
> optional ops a user declares one and silently never tears down on the other two.

## The work, one paragraph

A `defservice` cannot observe a connection beginning or ending — all four lifecycle arms pass `state`
through unchanged, so per-connection state is impossible to hold. Add two OPTIONAL internal ops,
`-on-connect` and `-on-disconnect`, dispatched by the macro at those arms. Mint one new context
record, `DisconnectInvocation`, carrying the reason. A service declaring neither op must compile and
behave **byte-for-byte** as it does today.

## The ONE contract decision, pinned

**Both ops are OPTIONAL, and the macro emits their dispatch only when the arm is declared.** 169
public arms were migrated in `037ef43e`; forcing every service to also declare lifecycle ops would be
a second corpus migration for a facility almost none of them want. Absent ⇒ the four arms keep their
current bodies unchanged.

## ★ RULED 2026-08-10 — `-on-connect` MAY REFUSE, and the plumbing already exists

> *"if the caller is being abusive and exceeding some rate limit or is flat out denied completely,
> obviously yes — that's business logic for the service to write, we just need the plumbing to
> enable it."*

`Outcome`'s five variants (`Reply`/`Stop`/`NoReply`/`ReplyAndArm`/`NoReplyAndArm`) cannot express a
refusal, so `-on-connect` gets **its own return type**, weighed 4/4 against three alternatives:
adding a `RefuseConnection` variant to the shared `Outcome` fails Obvious+Simple+Honest (a variant
legal in exactly one arm, which every other exhaustive match must then handle as an unreachable
case — and unreachable arms accumulate lies); lazy provisioning with no `-on-connect` fails Simple
(one creation site becomes an existence check in every op) and cannot gate at all.

```clojure
;; ⚠ NAME IS A PLACEHOLDER — an intueri cast is OWED. `:wat::kernel::ConnectOutcome` is TAKEN (the
;; CLIENT-side dial result: Connected/Refused/Rejected/Failed), so this cannot reuse it. Candidates:
;; Admission · AdmitOutcome · ConnectionOutcome · Gate.
;;
;; A placeholder is genuinely cheap HERE, unlike `CallCtx` — that one cost a rename because 120 arms
;; were at stake; this one has ZERO declared arms, so the cast can land before a single consumer
;; exists. Do not read this as licence to placeholder a name with live call sites.
(:wat::core::defenum :wat::service::<Admission> :wat::enum::Pure
  :Accept [state <- :S]                                    ;; keep the connection; state threads on
  :Refuse [state <- :S  cause <- :wat::kernel::Failure])   ;; tell the client, drop the peer, keep serving
```

**The refusal plumbing is the `Rejected` arm's, reused verbatim** (`wat/service.wat:1633-1640`) —
`try-send` the `Reply::Failed cause` (NON-blocking: the deadlock guard, because a client wedged
mid-send is not draining its reply side), match all four `TrySendOutcome` arms to `nil` best-effort,
then continue. It is *simpler* at connect than at reject: the peer is the `Connection peer` binding,
not `(nth selectables idx)`, because it was never added.

**⛔ `next-id` INCREMENTS ON A REFUSAL TOO.** The id was minted and handed to `-on-connect`'s ctx; a
service may well have logged the refusal against it. Recurring with `next-id` unchanged would hand
that same id to the NEXT connection and break the never-reused contract that the whole `conn-id`
design rests on. The id is consumed by the invocation, not by the connection surviving.

## The new type — mint it beside its three siblings

```clojure
;; the reason a connection ENDED. Closed is nullary — so there is no Option anywhere.
(:wat::core::defenum :wat::service::DisconnectReason :wat::enum::Pure
  :Closed   []
  :Lost     [cause <- :wat::kernel::Failure]
  :Rejected [cause <- :wat::kernel::Failure])

;; the ctx `-on-disconnect` receives. `-on-connect` keeps LifecycleInvocation (a connect has no
;; reason — that is WHY this is a separate type and not an Option field).
(:wat::core::defrecord :wat::service::DisconnectInvocation
  [~@:wat::service::InvocationCore
   conn-id <- :wat::core::i64
   reason  <- :wat::service::DisconnectReason])
```

Declare it in `wat/service.wat` immediately after `LifecycleInvocation` (~`:140`), splice-first per
the arc-293 house rule (`wat/telemetry.wat:82`).

## ★ THE MECHANISM — this is NOT the alarm path, and that is the whole strike

A fired **alarm** reaches its internal op as `ServiceEvent::Message idx op`, because the `Alarm`
**carries an `Op` variant** (`:op :-tick`) and returns through the ordinary message path where
`serve-dispatch-op` matches it. **Lifecycle events carry no op** — `Connection [peer]` and
`Closed`/`Lost`/`Rejected [idx …]` (`wat/spawn.wat:195-203`) have no `Op` field. So the macro must
**synthesize** the dispatch at each arm.

**⛔ Dispatch BEFORE the eviction.** The disconnect arms read the `conn-id` via
`(:wat::core::first (:wat::core::nth selectables idx))`; after `remove-at` it is gone.

## Read in order

1. **`wat/service.wat:1194`** — `self-ctx-ctor-expr`, the `SelfInvocation` ctor built at macro-expand
   time. **Copy this shape** for the two new ctors: `~fqdn-kw` / `~op-str` splice as literals; the
   `Uuid/v4` and clock read are bare identifiers evaluated at RUNTIME in the serve loop.
2. **`wat/service.wat:1199`** — `pub-ctx-ctor-expr`, the `Invocation` ctor. Shows how `conn-id` is
   read from the tuple at dispatch — the disconnect ctor needs the same read, at a different arm.
3. **`wat/service.wat:1210-1217`** — `binding-items` / `let-bindings`. The internal-arm binder
   vector. Your two new ops are internal ops, so their arms bind through exactly this path.
4. **`wat/service.wat:1066` + `:1098`** — `internal-op-kw-strs` and `has-internal-ops?`. **This is how
   the macro already knows which internal ops a service declared** — the "only when declared"
   contract hangs off exactly this list. You are adding two well-known names to a mechanism that
   already enumerates them.
5. **`wat/service.wat:1504`** — the `Connection` arm. Mints the id, `conj`s the tuple, recurs.
   `-on-connect` dispatches here, AFTER the mint (it needs the id) and the state it returns threads on.
6. **`wat/service.wat:1593` / `:1603` / `:1633`** — `Closed` / `Lost` / `Rejected`. All three call
   `-on-disconnect` with the matching `DisconnectReason`. `:1603` and `:1633` already hold a `cause`
   in scope; `:1593` has none, which is why `Closed` is nullary.
7. **`tests/services/probe_arc278_self_scheduling.wat:54`** — a live internal op, for arm shape.
8. **`tests/services/probe_arc278_call_context.wat:88`** — the live `-mark [s ctx]` internal arm
   reading its ctx through the binder. Your gate copies this observation trick.

## Implementation sketch

```
STEP 1  Mint DisconnectReason + DisconnectInvocation beside the family (~service.wat:140).

STEP 2  Two ctor exprs beside self-ctx-ctor-expr — one LifecycleInvocation (for -on-connect),
        one DisconnectInvocation (for -on-disconnect, reason spliced per arm).

STEP 3  At each of the four arms, emit the dispatch ONLY IF the op is declared (consult the
        internal-op list from :1066). Absent ⇒ emit today's body unchanged.
        Present ⇒ bind ctx, run the arm body, match its Outcome, thread the new state.
        Disconnect arms: read conn-id BEFORE remove-at.

STEP 4  The exemplar service + the gate (below).
```

An internal op returns `Outcome::NoReply new-state` (it has no client) — the same shape `-tick`
already uses; a `Reply` from one is already a located assertion.

## Blast radius

`wat/service.wat` + the new exemplar test + its fixture. **No `src/` change is expected** — this is
macro emission. **No existing `.wat` should change**, which the byte-for-byte gate below proves. If
you find yourself editing Rust or migrating the corpus, STOP and report why.

## ⛔ STOP triggers

1. **STOP-1 — a service declaring NEITHER op must be unchanged.** If any existing `.wat` needs an
   edit, the "optional" contract is broken. STOP.
2. **STOP-2 — dispatch BEFORE `remove-at`.** If the handler receives a `conn-id` read after the
   eviction, it is reading a shifted position. This is the whole positional-identity class.
3. **STOP-3 — ONE `-on-disconnect`, never three ops.** Ruled by the four questions: three optional
   ops let a user wire one and silently leak on the other two.
4. **STOP-4 — no `Option` anywhere in the new types.** `Closed` is a nullary variant precisely so the
   reason is total. An `Option<Failure>` rebuilds the conflation this design exists to avoid.
5. **STOP-5 — `-on-connect` gets `LifecycleInvocation`, `-on-disconnect` gets
   `DisconnectInvocation`.** Do not give either a `SelfInvocation` (no `conn-id`) or an `Invocation`
   (that is a client call).
8. **STOP-8 — `next-id` increments on a REFUSAL too.** See the ruling above: the id is consumed by
   the invocation, not by the connection surviving. Reusing it breaks `conn-id`'s never-reused
   contract, which everything keyed on it depends on. The stability gate will not catch this — add
   the assertion.
9. **STOP-9 — the refusal send is `try-send`, never `send`.** A blocking send to a client that is
   not draining wedges the whole serve loop. Copy `wat/service.wat:1635` exactly, all four
   `TrySendOutcome` arms faced.
10. **STOP-10 — `-on-connect` returns the new admission type; `-on-disconnect` returns `Outcome`
   (`NoReply`).** A disconnect cannot be refused — the connection is already gone — so do not give
   it an admission type it could only answer one way.
6. **STOP-6 — do NOT build the rete service.** Its states, its chunked `install-rules`, its query
   surface are the NEXT stone. The exemplar here has zero rete in it on purpose.
7. **STOP-7 — if the floor moves for any reason other than your new tests, STOP** and report the
   failing test's whole block verbatim plus the exact arm. Do not re-run first.

## The acceptance gate

New: `tests/services/probe_arc278_connection_lifecycle.{rs,wat}`.

The exemplar is the rete ratchet **with zero rete** — same monotonic law (each step forward closes
every prior operation), so it proves the mechanism the rete service will use:

```
-on-connect        → Provisioned          entry created
set-limit          → Ready(n)             ONCE; a second → LimitAlreadySet
bump        × N    → Ready(n')
seal               → Sealed(n')
read        × N    → Sealed               before seal → NotYetSealed
-on-disconnect     → entry removed
```

State lives in **ONE** `:ephemeral` map — `PersistentMap<i64, <UserState>>` — whose value is the
user's own enum, its constructor carrying the phase. **Not two maps, and not a phase field.**

1. **`-on-connect` fires with a populated ctx** — `conn-id` equals the id that client's later public
   ops observe; `namespace`/`operation` are the compile-time literals.
2. **★ THE LEAK GATE — all THREE eviction paths remove the entry, each with the right reason.**
   Connect three clients; evict one by clean `Closed`, one by `Lost`, one by `Rejected` (an
   over-budget frame). Assert the map is empty AND that each `-on-disconnect` saw the matching
   `DisconnectReason` variant. **`Rejected` is the path the seam had lost and the one a hand-written
   teardown forgets** — a gate testing only `Closed` proves nothing about the other two.
3. **★ ISOLATION + STABILITY.** Connect three, advance each to a **different** state, disconnect the
   MIDDLE one, assert the two survivors' states are untouched and still their own. This is the
   cross-tenant leak the design exists to prevent and the defect most likely to ship green.
4. **A service declaring neither op is byte-for-byte unchanged** — STOP-1 made falsifiable. Assert an
   existing untouched service (e.g. `probe_arc278_self_scheduling`) still passes.
5. **★ REFUSAL — the client is told, and the peer does not survive.** A service whose `-on-connect`
   returns `Refuse` for (say) the third connection: assert the client observes the failure (it
   receives the `Reply::Failed` cause, or its next op fails on a closed peer), that the refused peer
   is NOT in `selectables` (a subsequent op from it cannot be served), and that the service KEEPS
   SERVING the two it accepted.
6. **★ A REFUSED CONNECTION STILL BURNS ITS ID (STOP-8).** Refuse connection #2, then connect #3 and
   assert #3's `conn-id` is **not** #2's. Nothing else in the gate can catch an id rollback, and a
   reused id silently hands one tenant another's world the moment anything is keyed on it.

## Weigh

`cargo build --release` → `cargo nextest run --release -E 'test(connection_lifecycle)'` →
**`./scripts/floor.sh`, read the Summary line** → `cargo clippy --release --all-targets`.
Expect **4386 + your new tests**.

⚠ `--check` on `wat/service.wat` is NOT an arbiter (re-registers a loaded macro → `DuplicateMacro`),
and `service.wat` is baked in at build time. Rebuild, then the floor.

⚠ **Your targeted filter cannot see the whole floor.** The previous strike's rider ran
`-E 'test(call_context)'`, reported no STOPs, and the floor was red in two places it could not have
seen. Run the floor yourself before claiming any STOP verdict.
