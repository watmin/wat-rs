# NAMING TARGET — the per-call context record handed to a `defservice` op handler

> **Materialized for an intueri cast** (R17 self-prompt-injection: give the ward a real artifact,
> not a description of one). Sibling of `naming-target-scope-and-caller-id.md`, which settled the
> FIELD name `conn-id` and the correlation surface's namespace. **This target is the RECORD's own
> type name**, which that cast did not decide.
>
> `CallCtx` is a **PLACEHOLDER**, declared as such in the source header
> (`wat/service.wat:67`) and carried as an owed debt in the arc's SEAM. It shipped in `c8fcfe0d`
> because the strike needed *a* name; it was never weighed.

---

## What the thing IS (grounded — `wat/service.wat:87-92`, live on disk)

```clojure
(:wat::core::defrecord :wat::service::CallCtx
  [caller-id  <- :wat::core::i64
   namespace  <- :wat::core::keyword
   operation  <- :wat::core::String
   request-id <- :wat::core::Uuid
   start-ns   <- :wat::core::i64])
```

A **pure record**, minted fresh by the generated serve loop **once per dispatched request**, and
handed to a handler that opted in. It is not a handle, not a resource, not a session. Every field is
a pure scalar, so the record is wire-crossable and `:durable`-legal (293.W).

Field provenance, which constrains what the name may honestly claim:

| field | where it comes from | scope |
|---|---|---|
| `caller-id` | a monotonic `i64` minted in the serve loop's `Connection` arm | **per CONNECTION** — stable across every call that client makes |
| `namespace` | the service's own fqdn — a compile-time literal the macro already held | per SERVICE, constant |
| `operation` | the op arm's name — a compile-time literal | per OP, constant |
| `request-id` | one `Uuid/v4` | **per CALL** |
| `start-ns` | one clock read | **per CALL** |

★ **Note the mixed lifetime, because it is the naming problem.** Three fields are per-call, one is
per-connection, one is per-service. A name that says "request" undersells the connection identity a
multi-tenant handler needs; a name that says "connection" undersells the per-call id and clock.

## Why it exists (the requirement, not the implementation)

A handler received `(state, request)` and nothing else, so it **could not tell which client was
calling** — a multi-tenant service could not select a caller's own state even when that state
existed. The next stone (the connection-scoped world) keys a per-tenant map on `caller-id`.

## Where the name is READ

1. **In a user's op-arm signature** — the third, opt-in binder: `[s ctx req]`. This is the dominant
   read site; a user writes it in every handler that wants identity.
2. **As accessor heads in a handler body** — `(:wat::service::<Name>/caller-id ctx)`,
   `/request-id`, `/operation`.
3. **In the type's own declaration**, as above.
4. Potentially as a **field type** in a user record that stores the context.

## What it must NOT be confused with (live siblings in the same namespace)

- `:wat::service::Outcome` — a handler's RETURN (`Reply`/`NoReply`/`Stop`/`…`).
- `:wat::service::Alarm` — a self-scheduled timer.
- `<svc>::State` — the service's own threaded state, the FIRST binder.
- `<svc>::Record` — the `:durable` half of state.
- `:wat::telemetry::Scope` — the correlation surface (`namespace`/`uuid`/`tags`/`time-ns`) that
  `Metric` and `Log` splice. A LATER refinement may make this record splice a shared correlation
  surface; the name should not fight that.
- `:wat::program::Env` — process-level environment. Not per-call.

## Candidates, and the case against each (weigh; do not treat as a closed set)

| candidate | the case for | the case against |
|---|---|---|
| `CallCtx` (incumbent) | short; `ctx` binder reads naturally | **`Ctx` is an abbreviation carrying real weight** — intueri's own rule says `ctx` is acceptable only when *the type* speaks, and here `Ctx` IS the type. Names the thing by its role in a convention, not by what it holds. |
| `CallContext` | unabbreviated `CallCtx` | "context" is the classic mumble noun — it says *"stuff about the surroundings"*, not what is in it |
| `Request` | matches the dominant read (per-call) | **collides hard** — the third binder is `req`, the actual request payload. Two things named Request. |
| `RequestScope` | ties to the `Scope` correlation vocabulary | `Scope` was judged Level-2 in a prior cast for colliding with lexical/sandbox/`wat_dispatch` scope |
| `Caller` | names the load-bearing field (`caller-id`) and the REASON it was built | undersells `request-id`/`start-ns`, which are per-call not per-caller |
| `Invocation` | one call of an operation — covers op + time + id honestly | does it carry the CONNECTION identity? |
| `CallInfo` | plain | `Info` is a mumble noun (intueri flags `data`/`info`/`stuff`) |

## The question for the ward

**Which name keeps its promise** for a pure, per-dispatch record that carries a *stable caller
identity* alongside *per-call identity and timing* — read primarily as the middle binder of
`[s ctx req]` and as the head of accessor calls in user handler bodies?

Judge the incumbent `CallCtx` on its merits and say plainly whether it is Level 1 (lies), Level 2
(mumbles), or clear. Propose the name that would speak. The binder spelling (`ctx`) may be judged
separately from the TYPE name; say so if they should differ.

## ANCHORS — the live `:wat::service::` type set, for register consistency

```
Outcome · Alarm · CallCtx(placeholder) · Handle · State · Record · Request/Response pairs per op
```
