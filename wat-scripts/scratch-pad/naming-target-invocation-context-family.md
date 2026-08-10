# NAMING TARGET — the invocation-context family: two sibling records, one spliced core, one field

> **Materialized for an intueri cast** (R17 self-prompt-injection). Third in the series after
> `naming-target-scope-and-caller-id.md` and `naming-target-call-context-record.md`.
>
> The STRUCTURE below is ratified by the builder. **Do not re-litigate it** — judge the NAMES.

---

## The structure, ratified

Every invocation of a `defservice` op now receives a context. There are **three kinds of
invocation**, and their field sets are a strict NESTING, expressed with the shipped `~@`
surface-splice (the same mechanism that puts `Scope` into `Metric`/`Log`, `wat/telemetry.wat:84`):

```clojure
;; ── the invariant core — EVERY invocation has these four, no exceptions ──────────
(:wat::core::defsurface :wat::service::InvocationCore        ;; ← NAME IN QUESTION (D)
  :nature :wat::core::Record
  :features [namespace  <- :wat::core::keyword     ;; the service's own fqdn (compile-time literal)
             operation  <- :wat::core::String      ;; the op arm's own name (compile-time literal)
             request-id <- :wat::core::Uuid        ;; minted by the SERVER, per dispatch
             start-ns   <- :wat::core::i64])       ;; clock read, per dispatch

;; ── SELF-originated: a timer fired. No connection. No caller. ────────────────────
(:wat::core::defrecord :wat::service::???                    ;; ← NAME IN QUESTION (A)
  [~@:wat::service::InvocationCore])

;; ── CONNECTION-originated, no client message: a lifecycle event ─────────────────
(:wat::core::defrecord :wat::service::???                    ;; ← NAME IN QUESTION (B)
  [~@:wat::service::InvocationCore
   conn-id <- :wat::core::i64])

;; ── a CLIENT CALL: a connection AND a caller that identified itself ─────────────
(:wat::core::defrecord :wat::service::Invocation             ;; RATIFIED — do not rename
  [~@:wat::service::InvocationCore
   conn-id   <- :wat::core::i64
   parent-id <- :wat::core::Uuid])                           ;; ← NAME IN QUESTION (C)
```

**Which op receives which** — the leading `-` marks an internal op; the shape follows the NAME,
never the parameter count:

| op form | receives | when |
|---|---|---|
| `(op-name  [s ctx req])` | `Invocation` | a client sent a request |
| `(-on-connect [s ctx])` · `(-on-disconnect [s ctx])` | **(B)** | a connection arrived / went away |
| `(-tick [s ctx])` | **(A)** | a self-scheduled alarm fired |

---

# DECISION A — the SELF-originated context (core only)

The record a `-tick`-style internal op receives. There is **no connection and no caller** — the
service scheduled this itself, via `(:wat::service::Alarm :after … :op :-tick)`. It exists so a
timer-driven action is still visible to telemetry: it has an id and a time, so it can be logged and
measured exactly like a client call.

What it must NOT imply: a caller, a connection, a request, or a client.

Candidates (not a closed set — propose better):
`SelfInvocation` · `TimerInvocation` · `AlarmInvocation` · `InternalInvocation` · `Tick` ·
`ScheduledInvocation` · `Occurrence`

# DECISION B — the CONNECTION-originated, message-less context (core + `conn-id`)

The record `-on-connect` / `-on-disconnect` receive. A connection is the SUBJECT of the event —
`conn-id` is the whole point — but **no client sent a message**, so there is no request and no
caller-supplied id.

Note the tension a name here must survive: it is *about* a connection but is *not* a call, and it
is delivered to an INTERNAL op (no caller) even though it is connection-scoped.

Candidates: `ConnectionEvent` · `ConnectionInvocation` · `LifecycleInvocation` · `PeerEvent` ·
`LinkEvent` · `ConnectionContext`

# DECISION C — the field holding the CALLER'S OWN invocation id (on `Invocation`)

**Semantics, precisely.** The server mints its own `request-id` unconditionally — that is the
identity of this invocation. Field (C) is a **different value**: the id of the invocation *on the
caller's side* that caused this one. It arrives as data in the request. It is used **only to draw a
link** between two invocations, and is **never** treated as identity, never keyed on, never trusted.
A malicious client can put anything here; the worst outcome is a false edge in a trace.

It is MANDATORY, not optional: our generated client always mints and sends one, so a request
lacking it is *malformed* (an existing named failure), not a legitimate absent case.

Prior art: W3C Trace Context calls the analogous value the `parent` span id.

Candidates: `parent-id` · `caller-invocation-id` · `origin-id` · `upstream-id` · `causing-id` ·
`predecessor-id` · `trace-parent`

# DECISION D — the spliced core surface

The four fields every invocation has. Read at each splice site (`[~@<TheName> …]`) and by any
telemetry consumer that wants to log "any invocation" without caring which kind.

Candidates: `InvocationCore` · `Invocation` (taken) · `Dispatch` · `InvocationScope` ·
`CommonInvocation` · `AnyInvocation`

---

## ANCHORS + hard constraints from prior casts in this substrate

- **`:wat::service::` currently declares exactly THREE types:** `Alarm`, `Outcome`, `Invocation`.
  All bare single nouns. (`Handle`/`State`/`Record` are per-service MACRO-GENERATED names, not
  members of this namespace — a prior cast got this wrong and argued a five-sibling register.)
- **`Invocation` is RATIFIED** for the client-call record. Builder: *"Invocation reads better than
  CallCtx."* The ward that won it judged `CallCtx` a Level-2 mumble: `Ctx` fails intueri's own
  carve-out because `Ctx` **is** the type, so the abbreviation stands in for nothing.
- **`conn-id` is RATIFIED** for the connection field. `caller-id` was REJECTED — it is not a
  principal, carries no authz, and does not survive a reconnect, so "caller" over-claims. `-idx` was
  also refused: it is not a position (positions are the transport's round-scoped seat numbers).
- **`resource-id` was REJECTED** in an earlier cast — "resource" in this substrate means a live
  handle that cannot cross a wire. All four records here are PURE data, wire-crossable.
- **`Scope` was judged Level-2** in an earlier cast for colliding with lexical scope, sandbox scope,
  and `wat_dispatch` scope. Weigh that before proposing any `*Scope` name.
- The user-facing BINDER stays `ctx` for all of these, judged separately (it earns brevity by
  SCOPE — the fixed slot in `[s ctx req]` / `[s ctx]`, like `s` and `req`). **Do not propose binder
  changes.**

## The questions

1. Name A, B, C, D. Rank alternatives; be decisive — a hedge that lists options without ranking is a
   failed cast.
2. For each: does it keep its promise, and would a reader meeting it cold know what it holds?
3. **Do A, B and `Invocation` read as a FAMILY?** They are siblings by construction (a strict
   nesting over one core). A set of names that does not read as a set is a finding.
4. Flag any Level 1 (lies) or Level 2 (mumbles) you see in the RATIFIED parts too — `Invocation`,
   `conn-id`, the field names in the core. Ratified is not immune; say so if something is wrong.
