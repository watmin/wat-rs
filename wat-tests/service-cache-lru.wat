;; wat-tests/service-cache-lru.wat — arc 278 cache Stone 2: the MULTI-CLIENT `defservice` gate,
;; now on the BATCH `(Cache :- [K V])` surface (BRIEF-cache-batch-surface).
;;
;; Stone 1 (`a86f521c`) shipped `(:wat::cache::Lru :- [K V])`, thread-owned, zero mutex. Stone 2 is the
;; multi-client form: `:wat::cache::lru-svc :- [K V]` (`(:wat::cache::Cache :- [K V])` its wire surface) —
;; a `defservice` whose actor serialization IS the mutex, so N clients share ONE cache with no
;; lock written anywhere. `get`/`put` are BATCH in both directions (`docs/CONVENTIONS.md:658`,
;; arc 119 — every wat-rs-shipped service is batch-oriented, Console excepted; the original
;; single-key `(Cache :- [K V])` was a miss in the brief that designed it, corrected here). This gate
;; proves, one round trip per behaviour, per locus:
;;
;;   ★ INDEX ALIGNMENT — the load-bearing property a batch API can get wrong while every
;;     single-key test still passes: ONE `get` round trip, THREE probes, DELIBERATELY JUMBLED
;;     (a hit not first, a miss in the middle, another hit last, none in insertion order) —
;;     `results[i]` answers `probes[i]`.
;;   BATCH PUT, BATCH GET — several entries in one `put`, all readable in one `get` (the jumbled
;;     probe above reads BOTH entries the batch `put` wrote).
;;   BATCH-OF-ONE — the builder's own argument for batch-only ("a user wanting to read exactly
;;     one item can just produce a vec of one") proven not degenerate, on both `put` and `get`.
;;   EMPTY PROBE VECTOR — `get []` -> `Ok` with an empty results Vector, not an error.
;;   MULTI-CLIENT — client A `put`s, client B `get`s off the SAME `Handle/addr` and sees A's
;;     writes throughout. One cache, N clients — the arc-130 N-client case landing natively.
;;   EVICTION IS OBSERVABLE THROUGH THE ACTOR — capacity 2; a batch-of-one `put` overflows it;
;;     `PutResponse` itself carries nothing back (file-header departure note in `wat/cache.wat` —
;;     `Lru::put`'s displaced entry and `HolographicLru::put`'s silent `nil` cannot both be told
;;     truthfully through the same field), so eviction is proven the only way it CAN be proven
;;     here — a later `get` of the evicted key comes back `Miss`.
;;
;; K = String, V = i64 — two DIFFERENT concrete types (as `service-parametric-messages.wat`
;; establishes, a gate where K and V coincide cannot tell a correct instantiation from a shifted
;; one). Sequence, on ONE shared service (capacity 2), two dialed clients A and B:
;;   A put [k1=100, k2=200]        -> Ok                    (batch put; cache: {k1(LRU), k2(MRU)})
;;   B get [k2, "missing", k1]     -> [Hit:200, Miss, Hit:100]   (★ INDEX ALIGNMENT, jumbled;
;;                                                                bumps k1 to MRU: {k2(LRU), k1(MRU)})
;;   A put [k3=300]                -> Ok                    (batch-of-one; k2 was LRU -> evicted)
;;   B get [k2]                    -> [Miss]                (batch-of-one; EVICTION OBSERVABLE)
;;   B get []                      -> []                    (EMPTY PROBE VECTOR)
;;
;; Assert on the STRUCTURE exactly — each label function extracts the response's fields (via
;; pattern match / accessor, never a rendered-string `contains`) before composing the one
;; expected token string, exactly as `service-parametric-messages.wat` /
;; `service-request-malformed.wat` do.

;; ── dial — the separately-typed verb, load-bearing (pins K,V) per the parametric precedent ──
(:wat::core::defn :wat-tests::cache-svc/dial
  [a <- (:wat::kernel::Address :- [(:wat::cache::Cache::Op :- [:wat::core::String :wat::core::i64]) (:wat::cache::Cache::Reply :- [:wat::core::String :wat::core::i64])])]
  -> (:wat::kernel::Peer :- [(:wat::cache::Cache::Op :- [:wat::core::String :wat::core::i64]) (:wat::cache::Cache::Reply :- [:wat::core::String :wat::core::i64])])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))))

;; ── labels — extract the response's fields apart, render the one honest token ────────────────

;; one result -> one token; NEVER a rendered-string `contains`, a real pattern match per element.
(:wat::core::defn :wat-tests::cache-svc/result-label
  [r <- (:wat::cache::Cache::GetResult :- [:wat::core::i64])]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::cache::Cache::GetResult::Hit v) (:wat::string::concat "Hit:" (:wat::i64::to-string v)))
    ((:wat::cache::Cache::GetResult::Miss) "Miss")))

;; the whole batch's results, index order preserved, rendered "[tok,tok,...]" — the fold walks
;; `results` LEFT TO RIGHT and `conj` appends, so this string's token order IS `results`' order.
(:wat::core::defn :wat-tests::cache-svc/get-label
  [r <- (:wat::kernel::RecvOutcome :- [(:wat::cache::Cache::GetResponse :- [:wat::core::i64])])]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::cache::Cache::GetResponse::Ok results)
          (:wat::string::concat "["
            (:wat::string::concat
              (:wat::string::join ","
                (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String])
                                   res <- (:wat::cache::Cache::GetResult :- [:wat::core::i64])]
                    -> (:wat::core::Vector :- [:wat::core::String])
                    (:wat::core::conj acc (:wat-tests::cache-svc/result-label res)))
                  (:wat::core::Vector :- [:wat::core::String])
                  results))
              "]")))
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:wat::cache::Cache::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "cache-svc get: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat::cache::Cache::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "cache-svc get: unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost __cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))

;; `put` answers nothing meaningful (file-header departure note in `wat/cache.wat`) — the ONLY
;; honest token is whether the batch was accepted at all.
(:wat::core::defn :wat-tests::cache-svc/put-label
  [r <- (:wat::kernel::RecvOutcome :- [:wat::cache::Cache::PutResponse])]
  -> :wat::core::String
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:wat::cache::Cache::PutResponse::Ok) "Ok")
        ((:wat::cache::Cache::PutResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "cache-svc put: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat::cache::Cache::PutResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "cache-svc put: unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost __cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))

;; ── the gate: ONE service, TWO clients, ALL SIX behaviours in one round trip ──────────────────
(:wat::core::defn :wat-tests::cache-svc/run [locus <- :wat::spawn::Locus] -> :wat::core::String
  (:wat::core::let
    [h (:wat::cache::lru-svc/start :locus locus
         :record (:wat::cache::lru-svc::Record :capacity 2))
     a (:wat-tests::cache-svc/dial (:wat::cache::lru-svc::Handle/addr h))
     b (:wat-tests::cache-svc/dial (:wat::cache::lru-svc::Handle/addr h))
     ;; BATCH PUT — two entries, ONE round trip. Capacity has room for both: {k1(LRU), k2(MRU)}.
     ;; MULTI-CLIENT set-up: A writes, B (below) reads.
     put-batch (:wat-tests::cache-svc/put-label
                 (:wat::cache::lru-svc/put a
                   (:wat::cache::Cache::PutRequest
                     :entries (:wat::core::Vector :- [(:wat::cache::Entry :- [:wat::core::String :wat::core::i64])]
                                (:wat::cache::Entry :key "k1" :value 100)
                                (:wat::cache::Entry :key "k2" :value 200)))))
     ;; ★ INDEX ALIGNMENT — ONE `get` round trip, THREE probes, DELIBERATELY JUMBLED: k2 (a hit,
     ;; NOT the first-inserted key) first, an absent key in the middle, k1 (a hit) last — none in
     ;; insertion order. `results[i]` must answer `probes[i]` exactly: [Hit:200, Miss, Hit:100].
     ;; Also proves MULTI-CLIENT (B reads A's batch put) and BATCH GET reading BOTH entries the
     ;; batch put wrote, in one round trip. Side effect: hits bump k2 then k1 to MRU, leaving
     ;; {k2(LRU), k1(MRU)} — so k2, not k1, is next evicted.
     get-jumbled (:wat-tests::cache-svc/get-label
                   (:wat::cache::lru-svc/get b
                     (:wat::cache::Cache::GetRequest
                       :probes (:wat::core::Vector :- [:wat::core::String] "k2" "missing" "k1"))))
     ;; BATCH-OF-ONE put — the degenerate case, still meaningful: overflows capacity 2; k2 is LRU.
     ;; `PutResponse` carries nothing back (file-header departure note) — eviction is provable only
     ;; via a later `get` miss, which is exactly the next probe.
     put-k3 (:wat-tests::cache-svc/put-label
              (:wat::cache::lru-svc/put a
                (:wat::cache::Cache::PutRequest
                  :entries (:wat::core::Vector :- [(:wat::cache::Entry :- [:wat::core::String :wat::core::i64])]
                             (:wat::cache::Entry :key "k3" :value 300)))))
     ;; BATCH-OF-ONE get + EVICTION IS OBSERVABLE THROUGH THE ACTOR — k2 was evicted by the put
     ;; above; a batch-of-one get names it a Miss, not an error.
     get-k2-miss (:wat-tests::cache-svc/get-label
                   (:wat::cache::lru-svc/get b
                     (:wat::cache::Cache::GetRequest
                       :probes (:wat::core::Vector :- [:wat::core::String] "k2"))))
     ;; EMPTY PROBE VECTOR — `Ok` with an empty results Vector, not an error.
     get-empty (:wat-tests::cache-svc/get-label
                 (:wat::cache::lru-svc/get b
                   (:wat::cache::Cache::GetRequest :probes (:wat::core::Vector :- [:wat::core::String]))))
     _ (:wat::cache::lru-svc/stop h)]
    (:wat::string::concat put-batch
      (:wat::string::concat " | " (:wat::string::concat get-jumbled
        (:wat::string::concat " | " (:wat::string::concat put-k3
          (:wat::string::concat " | " (:wat::string::concat get-k2-miss
            (:wat::string::concat " | " get-empty))))))))))

;; ── thread tier ────────────────────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::cache-lru-multi-client-on-thread

  (:wat::test::assert-eq
    (:wat-tests::cache-svc/run (:wat::spawn::thread))
    "Ok | [Hit:200,Miss,Hit:100] | Ok | [Miss] | []"))

;; ── process tier ───────────────────────────────────────────────────────────────────────────
;; The SAME expectation, one token apart — tier-generality is the requirement, not a bonus: a
;; forked child re-registers the surface from the shipped `service-forms` bundle and the payload
;; crosses as ENCODED EDN, decoded against each message's declared field types on the way in.
(:wat::test::deftest :wat-tests::service::cache-lru-multi-client-on-process

  (:wat::test::assert-eq
    (:wat-tests::cache-svc/run (:wat::spawn::process))
    "Ok | [Hit:200,Miss,Hit:100] | Ok | [Miss] | []"))
