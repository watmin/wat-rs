;; wat-tests/service-parametric-two-params.wat — arc 278: a TWO-parameter `defservice :- [K V]`.
;;
;; Sibling of `service-parametric.wat` (which pins the ONE-parameter case). One param was never
;; enough to prove the machinery: `box-svc<T>` mints `Locus/launch<Op,Reply,State<T>,Admin<T>,
;; Status<T>>`, and `State<T>` carries no inner comma, so a flat `split(',')` on that call-head's
;; type-arg suffix happened to produce the right five fragments.
;;
;; RED before the depth-aware split (the call-head's `<…>` suffix was split on EVERY comma, so
;; `State<K,V>` tore into `State<K` + `V>` — 8 fragments bound positionally into 5 type params,
;; shifting every one of them):
;;   #wat.check/TypeMismatch {:callee ":wat::spawn::Locus/launch<…>" :param "#2"
;;     :expected ":V>" :got ":…::pair-svc::Admin<K,V>"}
;; `:V>` — a torn shard in an EXPECTED type — is the signature of the defect. GREEN after the
;; explicit-type-arg binder in `check.rs` split at bracket-depth 0 (via
;; `types::split_type_list_top_level`, the tracker `parse_type_list` has used since arc 170 W2
;; Strike 1a).
;;
;; WHAT THIS PROVES (beyond `--check`): the two-param service is STOOD UP on the thread locus, a
;; client `connect'`s to its address, and one `put` round-trips. K and V are BOTH load-bearing and
;; pinned to DIFFERENT concrete types at the `/start` call site — K=String (`:k (Some "hi")`),
;; V=i64 (`:v (Some 42)`). A gate where K and V coincided would not distinguish a correct split
;; from a shifted one. The handler READS BOTH durable fields (presence, not payload — a K- or
;; V-typed value is opaque inside a generic body), so both params reach the state and back.
;;
;; SCOPE (arc 278, the surface-minted op alias stone): the synthesized `(Pair::Op :- [K V])` /
;; `(Pair::Reply :- [K V])` carry the surface's params (unchanged), but the surface's `:messages` are
;; BARE again — `PutRequest` / `PutResponse` here, naming neither K nor V. Rust mints an alias per
;; op at the surface's registration site (`<Surface>::<op>/Request` / `/Response`) targeting each
;; message exactly as `:features` declares it, so `wat/service.wat` names that alias instead of
;; forcing every message to spell the surface's params vacuously. See `service-parametric.wat`'s
;; header for the fuller account and `service-parametric-messages.wat` for the gate where the
;; params ARE genuinely load-bearing ON THE WIRE. What THIS file pins is unchanged: the
;; depth-aware type-arg split, with K and V load-bearing in the STATE.

;; ── the surface: TWO type params, messages bare ─────────────────────────────────────────────
(:wat::core::defsurface :wat-tests::Pair :- [K V] :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::Pair::PutRequest [item <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::Pair::PutResponse :wat::enum::Pure
     ;; `echo` carries a value the assertion reads APART: item, plus a distinct weight per
     ;; durable field, so a state that lost K (or V) yields a DIFFERENT number, not a crash.
     :Ok              [echo <- :wat::core::i64]
     ;; ruling A — every serviceable op-Response carries the protocol-tier too-large variant.
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  ;; Stone 16.3 — `:max-request-bytes` is MANDATORY on a `:nature :Peer'` op.
  [(put [self <- (:wat-tests::Pair :- [K V])  req <- :wat-tests::Pair::PutRequest]
     -> :wat-tests::Pair::PutResponse :max-request-bytes 1024)])

;; ── the two-parameter service ───────────────────────────────────────────────────────────────
;; `k <- (Option :- [K])` and `v <- (Option :- [V])` are the whole point: ::Record, ::State, ::Admin,
;; ::Status and ::Handle are each generic in BOTH K and V, which is what forces the
;; `Locus/launch :- [Op Reply (State :- [K V]) (Admin :- [K V]) (Status :- [K V])]` call-head with NESTED type-args.
(:wat::service::defservice :wat-tests::pair-svc :- [K V]
  :satisfies (:wat-tests::Pair :- [K V])
  :durable   [k <- (:wat::core::Option :- [K])  v <- (:wat::core::Option :- [V])]
  :ephemeral []
  :impls
  [(put [s ctx req]
     (:wat::service::Outcome::Reply s
       (:wat-tests::Pair::PutResponse::Ok
         (:wat::i64::+
           (:wat-tests::Pair::PutRequest/item req)
           (:wat::i64::+
             ;; read the K-typed durable field generically — `kk` is bound at type K
             (:wat::core::match
                 (:wat-tests::pair-svc::Record/k (:wat-tests::pair-svc::State/durable s))
               ((:wat::core::Some kk) 10)
               (:wat::core::None 0))
             ;; read the V-typed durable field generically — `vv` is bound at type V
             (:wat::core::match
                 (:wat-tests::pair-svc::Record/v (:wat-tests::pair-svc::State/durable s))
               ((:wat::core::Some vv) 100)
               (:wat::core::None 0)))))))])

;; ── the gate: stand it up on the thread locus and round-trip one call ───────────────────────
;; K is pinned to String and V to i64 BY THE SEED — two DIFFERENT concrete types, so a split
;; that shifted the type-args could not accidentally still unify.
;; Expected: item 7 + 10 (k is Some) + 100 (v is Some) = 117.
(:wat::test::deftest :wat-tests::service::parametric-two-params-round-trip-on-thread

  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::pair-svc/start :locus (:wat::spawn::thread)
           :record (:wat-tests::pair-svc::Record
                     :k (:wat::core::Some "hi")
                     :v (:wat::core::Some 42)))
       c (:wat::core::match (:wat::kernel::connect (:wat-tests::pair-svc::Handle/addr h))
           ((:wat::kernel::ConnectOutcome::Connected p) p)
           ((:wat::kernel::ConnectOutcome::Refused c)
             (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Rejected c)
             (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
           ((:wat::kernel::ConnectOutcome::Failed c)
             (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
       r (:wat-tests::pair-svc/put c (:wat-tests::Pair::PutRequest :item 7))]
      (:wat::core::match r
        ((:wat::kernel::RecvOutcome::Message __recv)
          (:wat::core::match __recv
            ((:wat-tests::Pair::PutResponse::Ok echo) echo)
            ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
            ((:wat-tests::Pair::PutResponse::RequestTooLarge bytes cap)
              (:wat::kernel::assertion-failed! "pair-svc put: unexpected RequestTooLarge"
                :wat::core::None :wat::core::None))
            ((:wat-tests::Pair::PutResponse::RequestMalformed mpath mexpected mgot)
              (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
        ((:wat::kernel::RecvOutcome::Lost __cause)
          (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
        (:wat::kernel::RecvOutcome::Stopped
          (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
        (:wat::kernel::RecvOutcome::Closed
          (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))
    117))
