;; wat-tests/service-parametric.wat — arc 278: a PARAMETRIC `defservice` that CHECKS and RUNS.
;;
;; The question was formally open for a month (arc 290 SCOPE.md:64-69 — "does `defservice`
;; support a generic <K,V> service, or only monomorphic?"). It is ruled here BY BUILDING IT.
;;
;; RED before the split (wat/service.wat minted its companion names by naive concatenation onto
;; the raw fqdn):
;;   #wat.type/MalformedName {:message "malformed type name \":…::box-svc<T>::Record\":
;;     parametric name must close with '>'"}
;; The parser was innocent — `box-svc<T>::Record` genuinely IS malformed. The macro built it.
;; GREEN after: the suffix appends to the BASE and the params re-attach at the end
;; (`box-svc::Record :- [T]`), and every generated `defn` whose signature names a parametric
;; companion declares those params.
;;
;; WHAT THIS PROVES (beyond `--check`): the service is STOOD UP on the thread locus, a client
;; `connect'`s to its address, and one `put` round-trips — so the generic State/Record/Admin/
;; Status/Handle family is real machinery, not just well-formed names. The handler READS the
;; T-typed durable field (`held <- (Option :- [T])`) generically, so `T` is load-bearing in the state.
;;
;; SCOPE (v3 — arc 278, the surface-minted op alias stone): the surface's `:messages` are BARE
;; again — `PutRequest` / `PutResponse` name no `T` at all, honestly. Rust mints one alias per op
;; at the surface's REGISTRATION site (well after `expand_all`, when `:features` is actually
;; held), named `<Surface>::<op>/Request` / `/Response`, targeting each message EXACTLY as
;; `:features` declares it; `wat/service.wat` names that alias instead of guessing a message's
;; arity by concatenation. The prior `v2` forced every message to spell the surface's params even
;; vacuously (`PutRequest :- [T]`) because the macro could not do better — that forcing lock is
;; retired; spell what you use. What this file pins is unchanged: T is load-bearing in the STATE,
;; not on the wire.

;; ── the surface: parametric (arc 170 C2 — a shipped capability), messages bare ──────────────
(:wat::core::defsurface :wat-tests::Box :- [T] :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::Box::PutRequest [item <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::Box::PutResponse :wat::enum::Pure
     ;; `echo` carries the handler's answer back so the round-trip asserts a VALUE, not just
     ;; "no crash": item + 1 when the generic durable holds something, item + 0 when empty.
     :Ok              [echo <- :wat::core::i64]
     ;; ruling A — every serviceable op-Response carries the protocol-tier too-large variant.
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  ;; Stone 16.3 — `:max-request-bytes` is MANDATORY on a `:nature :Peer'` op.
  [(put [self <- (:wat-tests::Box :- [T])  req <- :wat-tests::Box::PutRequest]
     -> :wat-tests::Box::PutResponse :max-request-bytes 1024)])

;; ── the parametric service ──────────────────────────────────────────────────────────────────
;; `held <- (Option :- [T])` is the whole point: the durable record — and therefore ::State, ::Admin,
;; ::Status and ::Handle, each of which carries it — is generic in T.
(:wat::service::defservice :wat-tests::box-svc :- [T]
  :satisfies (:wat-tests::Box :- [T])
  :durable   [held <- (:wat::core::Option :- [T])]
  :ephemeral []
  :impls
  [(put [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat-tests::Box::PutResponse::Ok
         (:wat::core::i64::+
           (:wat-tests::Box::PutRequest/item req)
           ;; read the T-typed durable field generically — `v` is bound at type T
           (:wat::core::match
               (:wat-tests::box-svc::Record/held (:wat-tests::box-svc::State/durable s))
             ((:wat::core::Some v) 1)
             (:wat::core::None 0))))))])

;; ── the gate: stand it up on the thread locus and round-trip one call ───────────────────────
;; `T` is pinned to `i64` at the `/start` call site by the seed `(Some 42)` — the generic
;; service is instantiated exactly the way a caller instantiates any other generic fn.
;; Expected: item 7 + 1 (durable is `Some`) = 8.
(:wat::test::deftest :wat-tests::service::parametric-round-trip-on-thread

  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::box-svc/start :locus (:wat::spawn::thread)
           :record (:wat-tests::box-svc::Record :held (:wat::core::Some 42)))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::box-svc::Handle/addr h))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused c)
             (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected c)
             (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed c)
             (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       r (:wat-tests::box-svc/put c (:wat-tests::Box::PutRequest :item 7))]
      (:wat::core::match r
        ((:wat::kernel::RecvOutcome::Message __recv)
          (:wat::core::match __recv
            ((:wat-tests::Box::PutResponse::Ok echo) echo)
            ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
            ((:wat-tests::Box::PutResponse::RequestTooLarge bytes cap)
              (:wat::kernel::assertion-failed! "box-svc put: unexpected RequestTooLarge"
                :wat::core::None :wat::core::None))
            ((:wat-tests::Box::PutResponse::RequestMalformed mpath mexpected mgot)
              (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
        ((:wat::kernel::RecvOutcome::Lost __cause)
          (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
        (:wat::kernel::RecvOutcome::Stopped
          (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
        (:wat::kernel::RecvOutcome::Closed
          (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
    8))
