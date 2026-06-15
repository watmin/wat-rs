# BRIEF — Stone C.3: defservice's client face (full-gRPC) — BUILD-READY

**Status: STRIKE-READY draw, bootstrapped from full context (a "real curare" — the coordinate is
captured so the next self BUILDS, not re-derives).** Executor: Shadowdancer (sonnet). Anchor:
`/home/watmin/work/holon/wat-rs/` (verify `pwd`; ONLY here; ignore `.claude/worktrees/`). Do NOT
commit — the Inquisitor weighs. `cargo test` PLAINLY (no setsid/timeout). Trust your build over stale
rust-analyzer. **The shape + names + types + forks are ALL decided below — do not re-decide; build.**

## The work in one paragraph

Refine `:wat::service::defservice` (`wat/service.wat`) from C.2's inline-variant model to the full-gRPC
client face: per op, generate a standalone **Request** record + **Response** record; `Op`/`Reply` WRAP
them (not inline fields); `serve` unwraps the request + wraps the response; and ADD the client face —
**request constructors**, type-safe **methods**, and a **start fn** returning a **Handle** record.

## DECIDED (do not re-open)

- **Shape:** full-gRPC standalone records; `Op`/`Reply` wrap them; methods per-op type-safe (the
  decider was Honest — a wrong request must be UNCOMPILABLE).
- **Names (intueri, weighed):** `<fqdn>::<Op>Request` / `<fqdn>::<Op>Response` (records) · `<fqdn>/<op>-request`
  (constructor) · `<fqdn>/<op>` (method, lowercase) · `<fqdn>/start` · `<fqdn>::Handle` (start's return).
- **Types (GROUNDED):** `Handle.handle <- :wat::kernel::ProgramHandle<wat::core::nil>` ·
  `Handle.addr <- :wat::kernel::Address'<<fqdn>::Op,<fqdn>::Reply>` · per-op records via `:wat::Record::def`
  (accessor `<RecordFQDN>/<field>`) · Op/Reply via `defenum`.
- **Forks:** method takes **explicit `c`** (`(counter/get c req)`; facade deferred); per-op shapes are
  **`Record::def`** records, `Op`/`Reply` are **`defenum`** sums.

## The `:ops` surface (UNCHANGED from C.2; the body now returns the Response inside Outcome::Reply)

```clojure
(:wat::service::defservice :my::counter
  :state :wat::core::i64
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse s)))
   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [s' (:wat::core::i64::+ s n)]
       (:wat::service::Outcome::Reply s' (:my::counter::IncrementResponse s'))))])
```
Handler contract: `(s <- :State, …in) -> :wat::service::Outcome::Reply{new-state, <fqdn>::<Op>Response}`.
The body constructs the **Response record** (forward-ref to the macro-minted name, like C.2's Reply).

## THE EXACT TARGET EXPANSION (the counter — generalize the macro to emit this)

```clojure
(:wat::core::do
  ;; per-op records (Record::def). Empty fields for GetRequest.
  (:wat::Record::def :my::counter::GetRequest        [])
  (:wat::Record::def :my::counter::GetResponse       [value <- :wat::core::i64])
  (:wat::Record::def :my::counter::IncrementRequest  [n     <- :wat::core::i64])
  (:wat::Record::def :my::counter::IncrementResponse [value <- :wat::core::i64])
  ;; wire enums WRAP the records (one field per variant: req / resp)
  (:wat::core::defenum :my::counter::Op
    :Get       [req <- :my::counter::GetRequest]
    :Increment [req <- :my::counter::IncrementRequest])
  (:wat::core::defenum :my::counter::Reply
    :Get       [resp <- :my::counter::GetResponse]
    :Increment [resp <- :my::counter::IncrementResponse])
  ;; serve — unwrap req → bind handler params from req accessors → run body → Outcome::Reply{ns,resp}
  ;;        → wrap Reply::<Op>(resp) → send' → TCO-recur ns. (serve owns the Op/Reply envelope.)
  (:wat::core::defn :my::counter::serve
    [self    <- :wat::kernel::Peer'<my::counter::Reply,my::counter::Op>
     l       <- :wat::kernel::Listener'<my::counter::Op,my::counter::Reply>
     clients <- :wat::core::Vector<wat::kernel::Peer'<my::counter::Reply,my::counter::Op>>
     state   <- :wat::core::i64]
    -> :wat::core::nil
    (:wat::core::match (:wat::kernel::poll' self l clients) -> :wat::core::nil
      (:wat::kernel::ServiceEvent::Shutdown nil)
      ((:wat::kernel::ServiceEvent::Connection peer)
        (:my::counter::serve self l (:wat::core::conj clients peer) state))
      ((:wat::kernel::ServiceEvent::Message idx op)
        (:wat::core::match op -> :wat::core::nil
          ((:my::counter::Op::Get req)                          ;; no in-args besides state
            (:wat::core::match (:wat::core::let [s state]
                                 (:wat::service::Outcome::Reply s (:my::counter::GetResponse s)))
                -> :wat::core::nil
              ((:wat::service::Outcome::Reply ns resp)
                (:wat::core::do (:wat::kernel::send' (:wat::core::nth clients idx) (:my::counter::Reply::Get resp))
                  (:my::counter::serve self l clients ns)))))
          ((:my::counter::Op::Increment req)                    ;; bind n from req's accessor
            (:wat::core::match (:wat::core::let [s state  n (:my::counter::IncrementRequest/n req)]
                                 (:wat::core::let [s' (:wat::core::i64::+ s n)]
                                   (:wat::service::Outcome::Reply s' (:my::counter::IncrementResponse s'))))
                -> :wat::core::nil
              ((:wat::service::Outcome::Reply ns resp)
                (:wat::core::do (:wat::kernel::send' (:wat::core::nth clients idx) (:my::counter::Reply::Increment resp))
                  (:my::counter::serve self l clients ns)))))))
      ((:wat::kernel::ServiceEvent::Closed idx)
        (:my::counter::serve self l (:wat::std::list::remove-at clients idx) state))
      ((:wat::kernel::ServiceEvent::Lost idx _cause)
        (:my::counter::serve self l (:wat::std::list::remove-at clients idx) state))))
  ;; request constructors (lowercase fns)
  (:wat::core::defn :my::counter::get-request [] -> :my::counter::GetRequest (:my::counter::GetRequest))
  (:wat::core::defn :my::counter::increment-request [n <- :wat::core::i64] -> :my::counter::IncrementRequest
    (:my::counter::IncrementRequest n))
  ;; methods (explicit c; send Op, recv Reply, unwrap envelope → Response)
  (:wat::core::defn :my::counter::get
    [c <- :wat::kernel::Peer'<my::counter::Reply,my::counter::Op> req <- :my::counter::GetRequest]
    -> :my::counter::GetResponse
    (:wat::core::let [_ (:wat::kernel::send' c (:my::counter::Op::Get req))
                      r (:wat::kernel::recv' c)]
      (:wat::core::match r -> :my::counter::GetResponse ((:my::counter::Reply::Get resp) resp))))
  (:wat::core::defn :my::counter::increment
    [c <- :wat::kernel::Peer'<my::counter::Reply,my::counter::Op> req <- :my::counter::IncrementRequest]
    -> :my::counter::IncrementResponse
    (:wat::core::let [_ (:wat::kernel::send' c (:my::counter::Op::Increment req))
                      r (:wat::kernel::recv' c)]
      (:wat::core::match r -> :my::counter::IncrementResponse ((:my::counter::Reply::Increment resp) resp))))
  ;; start — mint listener', spawn serve, return the Handle
  (:wat::core::defn :my::counter::start [state0 <- :wat::core::i64] -> :my::counter::Handle
    (:wat::core::let [pair (:wat::kernel::listener' (:wat::spawn::thread) :my::counter::Op :my::counter::Reply)
                      l    (:wat::core::first pair)
                      addr (:wat::core::second pair)
                      svc  (:wat::kernel::spawn-program' (:wat::spawn::thread)
                             (:wat::core::fn [self <- :wat::kernel::Peer'<my::counter::Reply,my::counter::Op>] -> :wat::core::nil
                               (:my::counter::serve self l
                                 (:wat::core::Vector :wat::kernel::Peer'<my::counter::Reply,my::counter::Op>)
                                 state0)))]
      (:my::counter::Handle svc addr)))
  ;; the Handle record (start's return)
  (:wat::Record::def :my::counter::Handle
    [handle <- :wat::kernel::ProgramHandle<wat::core::nil>
     addr   <- :wat::kernel::Address'<my::counter::Op,my::counter::Reply>]))
```

## Generation algorithm (extend service.wat's foldl over `clauses`; per op `[opkw in-argvec -> out-fieldvec body]`)

For each op clause (`ch[0]=opkw ch[1]=in-argvec ch[2]=-> ch[3]=out-fieldvec ch[4]=body`):
- `in-fields` = `(drop (ast->children in-argvec) 3)` (the `s <- :State` triple dropped); `out-fields` = `(ast->children ch[3])`.
- **Request record:** `(:wat::Record::def <fqdn>::<Op>Request <in-fields-as-vector>)`.
- **Response record:** `(:wat::Record::def <fqdn>::<Op>Response <out-fields-as-vector>)`.
- **Op variant:** `<opkw> [req <- :<fqdn>::<Op>Request]`. **Reply variant:** `<opkw> [resp <- :<fqdn>::<Op>Response]`.
- **serve arm:** `((Op::<Op> req) (match (let [s state  <bind each in-field name k from (<fqdn>::<Op>Request/k req)>] <body>) -> nil ((Outcome::Reply ns resp) (do (send' (nth clients idx) (Reply::<Op> resp)) (serve self l clients ns)))))`.
- **constructor:** `(defn <fqdn>/<op>-request [<in-fields>] -> :<fqdn>::<Op>Request (<fqdn>::<Op>Request <in-field-names>))`.
- **method:** `(defn <fqdn>/<op> [c <- Peer'<Reply,Op> req <- :<fqdn>::<Op>Request] -> :<fqdn>::<Op>Response (let [_ (send' c (Op::<Op> req)) r (recv' c)] (match r ((Reply::<Op> resp) resp))))`.
- **once (not per-op):** the `start` fn + the `Handle` record (types grounded above).
- lowercase the op for fn names (`<op>` = op keyword name lowercased); keep PascalCase for records/variants.
Program-body path (top-level `let`, nested quasiquote) as C.1/C.2. The `do` holds: records + enums splice to top level (type decls), the defns keep it non-empty.

## RED probe to write (the gate) — `tests/probe_arc209_c3_defservice_client_face.rs`

defservice the counter (RPC `:ops` as above) + drive via the GENERATED client face on a thread:
```
h   (:my::counter/start 0)
c   (:wat::kernel::connect' (:my::counter::Handle/addr h))
_   (:my::counter/increment c (:my::counter/increment-request 5))   ;; -> IncrementResponse{5}
r   (:my::counter/get c (:my::counter/get-request))                 ;; -> GetResponse{5}
;; assert (:my::counter::GetResponse/value r) == 5 ; drop h.handle → :Shutdown → join
```
Verify RED at HEAD (the macro emits C.2 inline variants; `<Op>Request`/`start`/methods unresolved).

## Migrate (single format) + STOP triggers

- The C.1 (`probe_arc209_c1_defservice_op_enum`) + C.2 (`probe_arc209_c2_defservice_dispatch`) probes
  reference the OLD inline-variant + `Reply::Get value` shape — MIGRATE them to the wrapped-record shape
  (Op::Get carries a `req`; Reply::Get carries a `resp`; the C.2 driver's hand-rolled serve + the
  `reply-value` matcher update to `(Reply::Increment resp)` → `(IncrementResponse/value resp)`).
- STOP: `&` rest in an argvec (reject); program-body contract break (value → ast->children); the `do`
  wrapper fails check (records+enums splice; defns keep non-empty); any dual-format detection (single only).
- Touch ONLY `wat/service.wat` (the macro) + the probes. NO Rust. NO change to poll'/ServiceEvent/Peer'/Listener'/Address'/Outcome/Record::def.

## Gate

```
cargo test --release -p wat --test probe_arc209_c3_defservice_client_face   # 1 passed (GetResponse.value == 5)
cargo test --release -p wat --test probe_arc209_c2_defservice_dispatch      # GREEN (migrated to wrapped shape)
cargo test --release -p wat --test probe_arc209_c1_defservice_op_enum       # GREEN (migrated)
cargo test --release -p wat --lib -- --test-threads=1                        # 915 / 36 (zero new)
cargo test --release -p wat --test nursery -- --test-threads=1               # 895 / 4 (zero new)
cargo test --release --workspace --no-run                                    # compiles
```
Model to copy: `wat/service.wat` (the C.2 foldl you extend) + `wat/Record.wat` (Record::def) +
`probe_arc209_c0b1b` (the start/listener'/spawn-program' shape) + `wat/spawn.wat:101` (parametric defenum).
