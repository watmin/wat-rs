# Arc 209 Stone C.2 — defservice generates the dispatch loop (`serve`)

> C.1 made `defservice` emit the op-enum. C.2 makes it ALSO emit `serve` — the `poll'`/
> `ServiceEvent` dispatch loop that owns the live `:state`, decodes each op, calls the
> INLINE handler body (state-as-self), replies, and TCO-recurs. Thread tier. C.3 adds the
> client wrappers + start fn; this stone is the loop.

## ⚠️ MODEL RESHAPE (2026-06-14, builder — RPC (InShape, OutShape)) — supersedes the uniform-`O` framing below

**A service operation IS `(InRecord, OutRecord)`** — `get_object(GetObjectReq) → GetObjectResp`,
the gRPC method model. Decided (four-Q): ops are `(InShape, OutShape)`, both **records**, declared
**inline-and-minted** (referenced record types rejected: fail Obvious + Simple + break C.1's one-form
model). Consequences that REVISE the design below:

- **Input** = a record, ALREADY true in C.1: each `<fqdn>::Op` variant IS the input record (its
  fields = the InShape). No change to the request enum.
- **Output** = a record, per-op, NEW: the macro mints a SECOND enum `<fqdn>::Reply` — a tagged sum,
  one variant per op carrying that op's OutShape fields. The wire is `Peer'<Op, Reply>` (request sum
  in, response sum out), both tagged unions of records.
- **Handler** = `(s <- :State, <in-fields>) -> [<out-fields>] body`; body returns
  `(:Tuple new-state <out>)` where `<out>` is the op's `Reply::<Op>` record. `serve` matches
  `Op::X(in)` → body → `(new-s, out)` → `send'` `out` (already a `Reply` variant) → recur new-s.
- **NO `& rest`** in op argvecs (fixed-arity records; want variadic → a `Vector<T>` field). **NO
  uniform scalar `O`** — that's the pre-reshape sketch; the real `O` is the `<fqdn>::Reply` sum.

The "exact target output" + "uniform-`O`" sections below are the PRE-RESHAPE draft, kept for the
ServiceEvent-loop skeleton (which is unchanged); the op-dispatch + reply must be rebuilt to the
two-enum RPC model above. (This stone grew into the full RPC codegen — best drawn from fresh context.)

### NAMING — LOCKED (intueri cast 2026-06-14, weighed against the disk + accepted)

The `intueri` ward was cast on the candidate set and weighed (the two load-bearing calls are grounded
in disk facts: `ServiceEvent::Message` exists → a `Message` request enum would collide; wat has
`Result<T,E>` → a `Result` response enum would lie). **Locked surface:**

- **request enum: `<fqdn>::Op`** — kept. Mild mumble, but every alternative is worse: `Message`
  collides with `ServiceEvent::Message` (transport layer); `Request` imports HTTP; `Result` is taken;
  `Command` would lie (excludes queries). The per-variant names (`Op::Get`, `Op::Increment`) do the
  speaking; the match binding is the short `op`. Consistent with shipped C.1.
- **response enum: `<fqdn>::Reply`** — clear; the honest counterpart to `Op` (send an `Op`, get a
  `Reply`); zero disk collision; avoids the `Result<T,E>` lie.
- **dispatch loop fn: `<fqdn>::serve`** — clear; already the fn name in `probe_arc209_c0b1b` +
  `probe_arc209_c0b3aii`; names the role (serves requests).
- **call-form markers: `:state` / `:ops`** — kept (`:handlers` lies — the handler is the body, not the
  clause; `:methods` imports OO vocab wat rejects; `:ops` is a mild mumble with no better alternative).
- **per-op In/Out: variant-IS-the-record — NO `::In`/`::Out` sub-names.** `Op::<Op>` variant fields ARE
  the input record; `Reply::<Op>` variant fields ARE the output record. Matches `Record.wat` (a record
  IS its FQDN; no sub-names) and the ADT identity.

## What C.2 delivers

`defservice` expansion goes from a bare `defenum` (C.1) to a **`(do (defenum …) (defn …serve…))`**.
The `do`-wrapper now holds (the `defenum` still splices to top level via `splice_type_decls`; the
`defn serve` is the non-type-decl sibling that makes the `do` non-empty — the C.1 note's resolution).

## The handler contract (settled, surface A)

Each op clause is a List: `(:Head [s <- :State arg1 <- :T1 …] -> (:wat::core::Tuple :State <O>) body)`.
- `s <- :State` — the state-self (FIRST arg; the dispatch loop owns it, threads it per call).
- `arg1…` — the client args (became the op-enum variant's fields in C.1).
- `body` — a PURE transform returning `(:wat::core::Tuple new-state reply)` : `(:Tuple :State <O>)`.
- `<O>` — the reply type. **UNIFORM across all ops** (clients receive one `O`); validate (STOP-1).

## The exact target output (the counter — `tests/probe_arc209_c1_defservice_op_enum.rs` surface)

`defservice :my::counter :state :i64 :ops [(:Get …) (:Increment …)]` must expand to:

```clojure
(:wat::core::do
  ;; C.1 (unchanged) — the op enum (splices to top level)
  (:wat::core::defenum :my::counter::Op
    :Get
    :Increment [n <- :wat::core::i64])

  ;; C.2 (new) — the dispatch loop. Thread tier. Models probe_arc209_c0b1b's serve verbatim
  ;; for the ServiceEvent skeleton; the inner `(match op …)` is generated from :ops.
  (:wat::core::defn :my::counter::serve
    [self    <- :wat::kernel::Peer'<wat::core::i64,my::counter::Op>
     l       <- :wat::kernel::Listener'<my::counter::Op,wat::core::i64>
     clients <- :wat::core::Vector<wat::kernel::Peer'<wat::core::i64,my::counter::Op>>
     state   <- :wat::core::i64]                         ;; <- :State resolved to the declared :state <T>
    -> :wat::core::nil
    (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
      (:wat::kernel::ServiceEvent::Shutdown nil)
      ((:wat::kernel::ServiceEvent::Connection peer)
        (:my::counter::serve self l (:wat::core::conj clients peer) state))
      ((:wat::kernel::ServiceEvent::Message idx op)
        ;; the GENERATED op dispatch — one arm per op clause:
        (:wat::core::match op -> :wat::core::nil
          (:my::counter::Op::Get
            (:wat::core::let [s         state
                              result    (:wat::core::Tuple s s)              ;; <Get body>, s=state
                              new-state (:wat::core::nth result 0)
                              reply     (:wat::core::nth result 1)
                              _         (:wat::kernel::send' (:wat::core::nth clients idx) reply)]
              (:my::counter::serve self l clients new-state)))
          ((:my::counter::Op::Increment n)
            (:wat::core::let [s         state
                              result    (:wat::core::let [s' (:wat::core::i64::+ s n)]
                                          (:wat::core::Tuple s' s'))         ;; <Increment body>, s=state n=n
                              new-state (:wat::core::nth result 0)
                              reply     (:wat::core::nth result 1)
                              _         (:wat::kernel::send' (:wat::core::nth clients idx) reply)]
              (:my::counter::serve self l clients new-state)))))
      ((:wat::kernel::ServiceEvent::Closed idx)
        (:my::counter::serve self l (:wat::std::list::remove-at clients idx) state))
      ((:wat::kernel::ServiceEvent::Lost idx _cause)
        (:my::counter::serve self l (:wat::std::list::remove-at clients idx) state)))))
```

## The generation algorithm (extends `wat/service.wat`, program-body path)

C.1 already folds `:ops` into enum variants. C.2 ADDS, from the same `clauses`:

1. **`O` (reply type)** = from the first op clause's ret `(:Tuple :State <O>)` — the 2nd type arg
   (`ast->children` of the ret List → drop the `Tuple` head + `:State` → first remaining). Validate
   every op's ret has the SAME `<O>` (STOP-1).
2. **`state-ty`** is already a macro param (`:State` in handler argvecs resolves to it).
3. **per op clause** `ch = ast->children`: `opkw=ch[0]`, `argvec=ch[1]`, `body=ch[4]` (ch[2]=`->`,
   ch[3]=ret). Client-arg NAMES = every-3rd symbol of `(drop (ast->children argvec) 3)` (drop the
   `s <- :State` triple; C.1 already isolates the field triples — reuse, take the name symbol of each).
   - **pattern** = bare `~opkw-as-variant` if no client args, else `` `(~opkw-as-variant ~@arg-names) ``
     where the variant keyword = `<fqdn>::Op::<Head>`.
   - **arm body** = the `let` shown above: `s` bound to `state`, `result` = the spliced `body`,
     destructure `(nth result 0/1)`, `send'` the reply to `(nth clients idx)`, TCO-recur `serve` with
     `new-state` + unchanged `self`/`l`/`clients`.
4. **assemble** `serve` (the ServiceEvent skeleton, copied from c0b1b's shape, with the generated
   inner `(match op …)` spliced into the `:Message` arm) + wrap `(do <defenum> <serve>)`.

## The one contract decision

**`serve` is named `<fqdn>::serve` and emitted inside `(do (defenum) (defn serve))`.** The `do`
holds (non-empty now); `defenum` still top-levels via `splice_type_decls`. C.2 does NOT generate the
start fn or client wrappers (C.3) — the RED probe drives the generated `serve` with a hand-written
spawn/connect/send/recv driver (the c0b1b shape) + a literal initial state.

## Out of scope (rejected here, not deferred-silently)

- **Client wrappers + start fn** → C.3 (the probe hand-drives `serve`).
- **Process tier** → the thread-tier `poll'` is C0b.1b; the process `poll'` (C0b.3a-ii) already
  exists, so `serve` is tier-agnostic at the verb level, but C.2's probe + types are thread-tier.
- **Non-uniform reply types / multi-value replies beyond `(:Tuple :State <O>)`** → STOP-1; not this
  stone.
- **A terminal/Stop op** (foreground Run) → C.4 (banked).

## STOP triggers

1. **STOP-1:** the ops do not share ONE reply type `<O>` (heterogeneous `(:Tuple :State …)` 2nd args)
   → report; the uniform-`O` client contract is the precondition, not something to paper over.
2. **STOP-2:** the program-body contract breaks (a value reaches `ast->children` instead of a node —
   the C.1 trap); the macro top-level must stay a regular `let`, output via nested quasiquote.
3. **STOP-3:** `(nth <tuple> <literal>)` does not type-check as a tuple projection (it should —
   check.rs:9610 "polymorphic over Vec<T> and tuple, index-addressed"); if not, find the real tuple
   accessor before improvising.

## Gate

The RED probe `tests/probe_arc209_c2_defservice_dispatch.rs` — `defservice` a counter, hand-drive the
generated `serve` on a thread: `connect'` → `send' (Increment 5)` → `recv'` = 5 → `send' Get` →
`recv'` = 5 → owner drops the handle → `:Shutdown` → join completes. RED at HEAD (the macro emits only
the enum; `<fqdn>::serve` is unbound). GREEN once C.2 ships. + the C.1 probe still GREEN (enum
unchanged) + lib 915/36 + nursery 895/4 + workspace compiles.
