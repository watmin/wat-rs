# Arc 209 Stone C.3 — the client face (the full-gRPC RPC surface)

> C.2 made `defservice` generate `serve` (the dispatch loop) with `Op`/`Reply` variants carrying
> INLINE fields. C.3 makes the surface a real RPC: per-op **standalone Request/Response records**,
> `Op`/`Reply` WRAP them, type-safe **methods** + **request constructors** + a **start fn**. This
> stone REFINES C.2's variant generation (variants wrap the shape records) AND adds the client face.
> Thread tier. Stone D (counter proof via `deftest'`) and C.4 (terminal op / `:Stop`) follow.

## DECIDED (this session, builder-driven)

**Shape (four-Q): full-gRPC standalone shapes.** Each op is `method(InputShape) -> OutputShape`,
both NAMED standalone records; `Op`/`Reply` are tagged unions that WRAP them; methods are per-op
type-safe. The decider was **Honest**: with standalone records, handing `increment` a `GetRequest`
is **uncompilable** — the variant-as-shape alternative (method takes the whole `Op` enum) leaves the
wrong request representable, exactly the magic affordance [[feedback_no_magic_that_lets_llm_fake_correctness]]
forbids. (`:ops` SURFACE is unchanged; only the macro's emitted types change.)

**Names (intueri cast 2026-06-14, weighed + credited):**
| thing | name | note |
|---|---|---|
| per-op request record | `<fqdn>::<Op>Request` (e.g. `GetRequest`, `IncrementRequest`) | gRPC idiom; standalone record |
| per-op response record | `<fqdn>::<Op>Response` (e.g. `GetResponse`) | **`Response` ≠ the locked `Reply` enum** — resolves the collision (different word) |
| request constructor fn | `<fqdn>/<op>-request` (lowercase) | e.g. `(:my::counter/increment-request 5)` |
| method fn | `<fqdn>/<op>` (lowercase) | e.g. `(:my::counter/increment <req>)`; lowercase disambiguates from the `Op::Increment` variant |
| start fn | `<fqdn>/start` | OTP word; `[initial-state] -> <fqdn>::Handle` |
| start return record | `<fqdn>::Handle` | **not `Service`** (tautological — the namespace IS the service); `Handle` = the RAII token |
| Handle fields | `handle <- <spawn-handle ty>`, `addr <- <addr ty>` | ⚠️ TYPES NOT YET GROUND — see below |

Locked from C.2 (unchanged): `<fqdn>::Op` / `<fqdn>::Reply` (now WRAP the records) · `<fqdn>::serve`
· `:wat::service::Outcome<S,R>` · `:state`/`:ops`.

## ⚠️ GROUND BEFORE BUILDING (intueri judged names, not types)

1. **`spawn-program' (thread)` return type** — the `Handle.handle` field type. The c0b1b driver binds
   `svc (spawn-program' (thread) (fn …))`; find `svc`'s type (likely `:wat::kernel::ProgramHandle<…>`
   or `Thread'<…>` — GROUND it, do not trust intueri's guessed `Handle'`).
2. **`listener'` addr type** — the `Handle.addr` field type. c0b1b: `addr (second (listener' (thread) Op Reply))`;
   find `addr`'s type (likely `:wat::kernel::Address'<…>` — GROUND it).
3. **Does `Handle{handle, addr}` read awkwardly** (`Handle/handle`)? Acceptable per intueri, but confirm
   no better field name (`handle` IS the spawn handle; `addr` IS the connect address).

## The generation (REFINES C.2 + adds C.3) — per op clause `(:Op [s <- :State …in] -> [..out] body)`

C.2 currently makes `Op::<Op>` carry the in-fields inline + `Reply::<Op>` the out-fields inline. C.3
splits those into standalone records and wraps:

1. **Request record:** `(:wat::core::defenum?? no — defrecord)` `<fqdn>::<Op>Request [..in-fields]`
   (the in-argvec minus the `s <- :State` triple). NOTE: a single-variant record = a `defrecord`, not
   a `defenum` — GROUND whether the per-op shapes are `defrecord` (record) and `Op` is the `defenum`
   wrapping them. (`wat/Record.wat` is the defrecord model.)
2. **Response record:** `<fqdn>::<Op>Response [..out-fields]` (the out-fieldvec).
3. **`Op` enum (REVISED):** `:Get [req <- :<fqdn>::GetRequest] | :Increment [req <- :<fqdn>::IncrementRequest]`
   — each variant wraps its Request record (was: inline fields).
4. **`Reply` enum (REVISED):** `:Get [resp <- :<fqdn>::GetResponse] | …` — wraps Response records.
5. **`serve` (REVISED arm):** match `Op::<Op> req` → bind the handler's in-param names from `req`'s
   accessors (`n` = `<Op>Request/n`) + `s` = state → run the inline `body` → `Outcome::Reply{new-state,
   <Response>}` (body constructs the Response record) → serve wraps `Reply::<Op>(response)` → `send'`
   → recur new-state. (serve owns the Op/Reply ENVELOPE wrap/unwrap; the handler speaks Request/Response.)
6. **request constructor:** `<fqdn>/<op>-request [..in] -> <fqdn>::<Op>Request` (builds the record).
7. **method:** `<fqdn>/<op> [c <- Peer'<Reply,Op> req <- <fqdn>::<Op>Request] -> <fqdn>::<Op>Response`
   = `send' c (Op::<Op> req)` → `recv' c` → `(match reply ((Reply::<Op> resp) resp))` (unwrap envelope → Response).
   ⚠️ PEER THREADING: does the method take the connected `c` explicitly, or is it partial-applied /
   ambient (the builder's `install_closures` facade)? The builder's example `(my.counter/get (get-request))`
   OMITS `c` — RESOLVE: explicit `c` arg vs. a connect-facade that closes over `c`. (Open — four-Q at draw.)
8. **start:** `<fqdn>/start [initial-state <- :State] -> <fqdn>::Handle` = mint `listener' (thread) Op Reply`,
   `spawn-program' (thread) (fn [self] (serve self l [] initial-state))`, return `Handle{spawn-handle, addr}`.

## The counter target (sketch — the BRIEF will carry the exact expansion)

```clojure
;; per-op records
(:wat::core::defrecord :my::counter::GetRequest        [])
(:wat::core::defrecord :my::counter::GetResponse       [value <- :wat::core::i64])
(:wat::core::defrecord :my::counter::IncrementRequest  [n     <- :wat::core::i64])
(:wat::core::defrecord :my::counter::IncrementResponse [value <- :wat::core::i64])
;; wire enums wrap the records
(:wat::core::defenum :my::counter::Op    :Get [req <- :my::counter::GetRequest]  :Increment [req <- :my::counter::IncrementRequest])
(:wat::core::defenum :my::counter::Reply :Get [resp <- :my::counter::GetResponse] :Increment [resp <- :my::counter::IncrementResponse])
;; serve (unwrap req → handler → wrap Reply), constructors, methods, start … (see BRIEF)
;; client face:
;;   (:my::counter/increment c (:my::counter/increment-request 5))  -> :my::counter::IncrementResponse  (.value = 5)
```

## Open at draw (four-Q each — do NOT punt to the builder; decide + note)

- **Peer threading** (#7 above): explicit `c` arg vs. connect-facade closing over `c`. (The
  `install_closures`/partial-application hint suggests a facade; weigh against the explicit-arg
  simplicity + the type story.)
- **`defrecord` vs `defenum` for the per-op shapes** (a single-shape record is a `defrecord`).
- **Single-field response unwrap**: the method returns the full `<Op>Response` record (uniform), NOT
  the bare `.value` (decided: uniform output record).

## Scope / out (C.4+, rejected here)

- The terminal op / `:Stop` / `:NoReply` `Outcome` variants → C.4.
- The process/remote tier client face → later (thread tier is the model; `serve` is tier-agnostic at
  the verb level via `poll'`).
- Stone D (the `deftest'` counter proof exercising the full client face) → after C.3 ships.

## Gate (the RED probe the draw writes)

`defservice` a counter (RPC `:ops`, unchanged surface) + drive via the GENERATED client face on a
thread: `(start 0)` → `connect' .addr` → `(increment c (increment-request 5))` = `IncrementResponse{5}`
→ `(get c (get-request))` = `GetResponse{5}` → drop the handle → `:Shutdown` → join. + C.1/C.2 probes
migrate to the wrapped-record shape + stay green + lib 915/36 + nursery 895/4 + workspace compiles.
