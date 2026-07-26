;; wat-tests/service-cache-lru.wat — arc 278 cache Stone 2: the MULTI-CLIENT `defservice` gate.
;;
;; Stone 1 (`a86f521c`) shipped `:wat::cache::Lru<K,V>`, thread-owned, zero mutex. Stone 2 is the
;; multi-client form: `:wat::cache::lru-svc<K,V>` (`:wat::cache::Cache<K,V>` its wire surface) —
;; a `defservice` whose actor serialization IS the mutex, so N clients share ONE cache with no
;; lock written anywhere. This gate proves the three load-bearing behaviours, one round trip per
;; locus:
;;
;;   1. MULTI-CLIENT — client A `put`s, client B `get`s off the SAME `Handle/addr` and sees A's
;;      value. One cache, N clients — the arc-130 N-client case landing natively.
;;   2. EVICTION IS OBSERVABLE — capacity 2; a third distinct key overflows, and the `put` that
;;      overflows returns `Ok[displaced = Some(Entry …)]` NAMING the evicted key.
;;   3. MISS IS A VALUE — `get` of the evicted (now-absent) key returns the `Miss` variant, not
;;      an error.
;;
;; K = String, V = i64 — two DIFFERENT concrete types (as `service-parametric-messages.wat`
;; establishes, a gate where K and V coincide cannot tell a correct instantiation from a shifted
;; one). Sequence, on ONE shared service, two dialed clients A and B:
;;   A put k1=100         -> NoDisplace                      (cache: {k1})
;;   B get k1              -> Hit:100                        (MULTI-CLIENT — B sees A's write)
;;   A put k2=200         -> NoDisplace                      (cache: {k1(LRU), k2(MRU)})
;;   B get k1              -> Hit:100                        (bumps k1 to MRU: {k2(LRU), k1(MRU)})
;;   A put k3=300         -> Displaced:k2=200                (EVICTION — k2 was LRU)
;;   B get k2              -> Miss                           (MISS IS A VALUE)
;;
;; Assert on the STRUCTURE exactly — each label function extracts the response's fields (via
;; pattern match / accessor, never a rendered-string `contains`) before composing the one
;; expected token string, exactly as `service-parametric-messages.wat` /
;; `service-request-malformed.wat` do.

;; ── dial — the separately-typed verb, load-bearing (pins K,V) per the parametric precedent ──
(:wat::core::defn :wat-tests::cache-svc/dial
  [a <- :wat::kernel::Address'<wat::cache::Cache::Op<wat::core::String,wat::core::i64>,wat::cache::Cache::Reply<wat::core::String,wat::core::i64>>]
  -> :wat::kernel::Peer'<wat::cache::Cache::Op<wat::core::String,wat::core::i64>,wat::cache::Cache::Reply<wat::core::String,wat::core::i64>>
  (:wat::core::match (:wat::kernel::connect' a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))))

;; ── labels — extract the response's fields apart, render the one honest token ────────────────
(:wat::core::defn :wat-tests::cache-svc/get-label
  [r <- :wat::kernel::RecvOutcome<wat::cache::Cache::GetResponse<wat::core::i64>>]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::cache::Cache::GetResponse::Hit v) (:wat::core::string::concat "Hit:" (:wat::core::i64::to-string v)))
        ((:wat::cache::Cache::GetResponse::Miss) "Miss")
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat::cache::Cache::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "cache-svc get: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat::cache::Cache::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "cache-svc get: unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost __cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :wat-tests::cache-svc/put-label
  [r <- :wat::kernel::RecvOutcome<wat::cache::Cache::PutResponse<wat::core::String,wat::core::i64>>]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::cache::Cache::PutResponse::Ok displaced)
          (:wat::core::match displaced
            ((:wat::core::Some e)
              (:wat::core::string::concat "Displaced:"
                (:wat::core::string::concat (:wat::cache::Entry/key e)
                  (:wat::core::string::concat "=" (:wat::core::i64::to-string (:wat::cache::Entry/value e))))))
            (:wat::core::None "NoDisplace")))
        ((:wat::cache::Cache::PutResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "cache-svc put: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat::cache::Cache::PutResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "cache-svc put: unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost __cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

;; ── the gate: ONE service, TWO clients, the three behaviours in one round trip ───────────────
(:wat::core::defn :wat-tests::cache-svc/run [locus <- :wat::spawn::Locus] -> :wat::core::String
  (:wat::core::let
    [h (:wat::cache::lru-svc/start :locus locus
         :record (:wat::cache::lru-svc::Record :capacity 2))
     a (:wat-tests::cache-svc/dial (:wat::cache::lru-svc::Handle/addr h))
     b (:wat-tests::cache-svc/dial (:wat::cache::lru-svc::Handle/addr h))
     ;; A put, capacity has room.
     put-k1 (:wat-tests::cache-svc/put-label
              (:wat::cache::lru-svc/put a (:wat::cache::Cache::PutRequest :key "k1" :value 100)))
     ;; (1) MULTI-CLIENT — B, a DIFFERENT client off the same addr, sees A's write.
     get-k1-by-b (:wat-tests::cache-svc/get-label
              (:wat::cache::lru-svc/get b (:wat::cache::Cache::GetRequest :key "k1")))
     ;; fills the cache to capacity: {k1(LRU), k2(MRU)}.
     put-k2 (:wat-tests::cache-svc/put-label
              (:wat::cache::lru-svc/put a (:wat::cache::Cache::PutRequest :key "k2" :value 200)))
     ;; bumps k1 to MRU: {k2(LRU), k1(MRU)} — so k2, not k1, is next evicted.
     get-k1-again (:wat-tests::cache-svc/get-label
              (:wat::cache::lru-svc/get b (:wat::cache::Cache::GetRequest :key "k1")))
     ;; (2) EVICTION IS OBSERVABLE — a third distinct key overflows capacity 2; k2 is LRU.
     put-k3 (:wat-tests::cache-svc/put-label
              (:wat::cache::lru-svc/put a (:wat::cache::Cache::PutRequest :key "k3" :value 300)))
     ;; (3) MISS IS A VALUE — k2 was evicted, so a subsequent get is a named Miss, not an error.
     get-k2-miss (:wat-tests::cache-svc/get-label
              (:wat::cache::lru-svc/get b (:wat::cache::Cache::GetRequest :key "k2")))
     _ (:wat::cache::lru-svc/stop h)]
    (:wat::core::string::concat put-k1
      (:wat::core::string::concat " | " (:wat::core::string::concat get-k1-by-b
        (:wat::core::string::concat " | " (:wat::core::string::concat put-k2
          (:wat::core::string::concat " | " (:wat::core::string::concat get-k1-again
            (:wat::core::string::concat " | " (:wat::core::string::concat put-k3
              (:wat::core::string::concat " | " get-k2-miss))))))))))))

;; ── thread tier ────────────────────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::cache-lru-multi-client-on-thread

  (:wat::test::assert-eq
    (:wat-tests::cache-svc/run (:wat::spawn::thread))
    "NoDisplace | Hit:100 | NoDisplace | Hit:100 | Displaced:k2=200 | Miss"))

;; ── process tier ───────────────────────────────────────────────────────────────────────────
;; The SAME expectation, one token apart — tier-generality is the requirement, not a bonus: a
;; forked child re-registers the surface from the shipped `service-forms` bundle and the payload
;; crosses as ENCODED EDN, decoded against each message's declared field types on the way in.
(:wat::test::deftest :wat-tests::service::cache-lru-multi-client-on-process

  (:wat::test::assert-eq
    (:wat-tests::cache-svc/run (:wat::spawn::process))
    "NoDisplace | Hit:100 | NoDisplace | Hit:100 | Displaced:k2=200 | Miss"))
