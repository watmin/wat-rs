;; wat-tests/service-cache-hologram.wat — arc 278 cache Stone 4: the SIMILARITY cache, as a
;; service, now on the BATCH `(Cache :- [K V])` surface (BRIEF-cache-batch-surface).
;;
;; Stone 3 (`wat-tests/cache/HolographicLru.wat`) proved `:wat::cache::HolographicLru` in-process.
;; Stone 4 puts it behind the SAME `(:wat::cache::Cache :- [K V])` multi-client surface Stone 2's
;; `lru-svc :- [K V]` wears (`wat-tests/service-cache-lru.wat`), pinned concretely at
;; `(Cache :- [wat::holon::HolonAST wat::holon::HolonAST])` (`:wat::cache::hologram-svc`, no `:- [K V]` of
;; its own — `HolographicLru` is concrete over `HolonAST`, so the service is too). `get`/`put` are
;; BATCH in both directions (`docs/CONVENTIONS.md:658`), same as `lru-svc`.
;;
;; Load-bearing behaviours, ONE round trip, two dialed clients A and B (mirrors Stone 2's shape —
;; steal the `dial` idiom, the separately-typed verb that pins the wire's type args):
;;
;;   ★ INDEX ALIGNMENT — ONE `get` round trip, THREE probes, DELIBERATELY JUMBLED: a similarity
;;     hit (`probe-near-k1`, coincident with but NOT EQUAL to k1) first, a miss in the middle, an
;;     exact-key hit last — `results[i]` answers `probes[i]`.
;;   ★ SIMILARITY ACROSS THE WIRE — `probe-near-k1` (Thermometer @ 50.01) is a DIFFERENT HolonAST
;;     from k1 (Thermometer @ 50.0) yet still hits k1's value. No in-process test can show the
;;     similarity match surviving encode -> wire -> decode; this is the one that matters.
;;   BATCH PUT, BATCH GET — two entries in one `put`, both readable in the jumbled `get` above.
;;   BATCH-OF-ONE — `put`s and `get`s a single entry each, proving the degenerate case is not
;;     degenerate.
;;   EMPTY PROBE VECTOR — `get []` -> `Ok` with an empty results Vector, not an error.
;;   MULTI-CLIENT — A `put`s; B, a DIFFERENT client off the SAME `Handle/addr`, does every `get`
;;     and sees A's writes throughout.
;;   EVICTION IS VISIBLE THROUGH THE SERVICE — capacity 2; a batch-of-one `put` (`k3`) overflows
;;     it and dual-evicts the LRU entry (`k1`, since the jumbled `get` bumped it to MRU and then
;;     `k2`'s `put` filled the second slot); a later `get` of `k1` shows the dual-eviction
;;     invariant holding THROUGH THE ACTOR.
;;
;; `HolographicLru::put` returns `nil` (unlike Stone 1's `Lru::put`) — the dual-eviction chain
;; removes the displaced key from the Hologram internally but never hands it back, so this was
;; ALREADY an honest `nil` per entry before batching. `PutResponse::Ok []` (file-header departure
;; note in `wat/cache.wat`) says the same thing at the whole-batch level — not a lie dressed up to
;; mirror `lru-svc`. Eviction is proven the only way it CAN be proven here — a later `get` miss.
;;
;; Assert on structure exactly (never a rendered-string `contains`): `get-results` unwraps
;; `(RecvOutcome :- [(GetResponse :- [HolonAST])])` down to the raw `(Vector :- [(GetResult :- [HolonAST])])`, dying loud
;; (`assertion-failed!`) on a wire breach (Lost/Closed) or an unexpected response variant
;; (RequestTooLarge/RequestMalformed); the gate then `assert-eq`s that Vector against a literal
;; expected Vector — index position and all — so a shifted or dropped result fails structurally,
;; not by string coincidence. HolonAST has no natural string form, and stringifying it to
;; `contains`-match would be exactly the anti-pattern the brief rules out.

;; ── dial — the separately-typed verb, load-bearing (pins the wire's type args) ────────────────
(:wat::core::defn :wat-tests::hologram-svc/dial
  [a <- (:wat::kernel::Address :- [(:wat::cache::Cache::Op :- [:wat::holon::HolonAST :wat::holon::HolonAST]) (:wat::cache::Cache::Reply :- [:wat::holon::HolonAST :wat::holon::HolonAST])])]
  -> (:wat::kernel::Peer :- [(:wat::cache::Cache::Op :- [:wat::holon::HolonAST :wat::holon::HolonAST]) (:wat::cache::Cache::Reply :- [:wat::holon::HolonAST :wat::holon::HolonAST])])
  (:wat::core::match (:wat::kernel::connect a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))))

;; ── assertion helpers — unwrap RecvOutcome, then assert the STRUCTURE, dying loud on a breach ──

;; unwraps down to the raw index-aligned results Vector — the CALLER `assert-eq`s it against a
;; literal expected Vector (structural, position-sensitive), never a per-element helper here.
(:wat::core::defn :wat-tests::hologram-svc/get-results
  [r <- (:wat::kernel::RecvOutcome :- [(:wat::cache::Cache::GetResponse :- [:wat::holon::HolonAST])])]
  -> (:wat::core::Vector :- [(:wat::cache::Cache::GetResult :- [:wat::holon::HolonAST])])
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::cache::Cache::GetResponse::Ok results) results)
        ((:wat::cache::Cache::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "hologram-svc get: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat::cache::Cache::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "hologram-svc get: unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

;; `put` answers nothing meaningful (file-header departure note in `wat/cache.wat`) — the only
;; honest assertion is that the batch was accepted at all.
(:wat::core::defn :wat-tests::hologram-svc/assert-put-ok
  [r <- (:wat::kernel::RecvOutcome :- [:wat::cache::Cache::PutResponse])]
  -> :wat::core::nil
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::cache::Cache::PutResponse::Ok) nil)
        ((:wat::cache::Cache::PutResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "hologram-svc put: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat::cache::Cache::PutResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "hologram-svc put: unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

;; ── the gate: ONE service, TWO clients, ALL SEVEN behaviours in one round trip ────────────────
(:wat::core::defn :wat-tests::hologram-svc/run [locus <- :wat::spawn::Locus] -> :wat::core::nil
  (:wat::core::let
    [h (:wat::cache::hologram-svc/start :locus locus
         :record (:wat::cache::hologram-svc::Record :capacity 2
                   :filter (:wat::cache::HologramFilterKind::Coincident)))
     a (:wat-tests::hologram-svc/dial (:wat::cache::hologram-svc::Handle/addr h))
     b (:wat-tests::hologram-svc/dial (:wat::cache::hologram-svc::Handle/addr h))
     ;; k1 — a Thermometer @ 50.0; probe-near-k1 is a DIFFERENT HolonAST, coincident by cosine.
     k1            (:wat::holon::Thermometer 50.0 0.0 100.0)
     v1            (:wat::holon::leaf :fifty)
     probe-near-k1 (:wat::holon::Thermometer 50.01 0.0 100.0)
     k2 (:wat::holon::leaf :k2)
     v2 (:wat::holon::leaf :v2)
     k3 (:wat::holon::leaf :k3)
     v3 (:wat::holon::leaf :v3)
     ;; never put; not coincident with anything the store will hold. Used inside the ★ INDEX
     ;; ALIGNMENT probe below as the jumbled batch's miss.
     probe-far (:wat::holon::leaf :nope)

     ;; BATCH PUT — two entries, ONE round trip. Fills the cache to capacity 2: {k1(LRU), k2(MRU)}.
     _put-batch (:wat-tests::hologram-svc/assert-put-ok
                  (:wat::cache::hologram-svc/put a
                    (:wat::cache::Cache::PutRequest
                      :entries (:wat::core::Vector :- [(:wat::cache::Entry :- [:wat::holon::HolonAST :wat::holon::HolonAST])]
                                 (:wat::cache::Entry :key k1 :value v1)
                                 (:wat::cache::Entry :key k2 :value v2)))))
     ;; ★ INDEX ALIGNMENT — ONE `get` round trip, THREE probes, DELIBERATELY JUMBLED: a similarity
     ;; hit (probe-near-k1, NOT k1 itself) first, a miss in the middle, an exact hit last.
     ;; Also ★ SIMILARITY ACROSS THE WIRE (probe-near-k1 hits k1's value despite being a DIFFERENT
     ;; HolonAST) and (1) MULTI-CLIENT (B reads A's batch put) and BATCH GET (reads both entries
     ;; the batch put wrote). Side effect: the similarity hit bumps k1 to MRU, then the exact hit
     ;; on k2 bumps k2 to MRU — leaving {k1(LRU), k2(MRU)}.
     _align (:wat::test::assert-eq
              (:wat-tests::hologram-svc/get-results
                (:wat::cache::hologram-svc/get b
                  (:wat::cache::Cache::GetRequest
                    :probes (:wat::core::Vector :- [:wat::holon::HolonAST] probe-near-k1 probe-far k2))))
              (:wat::core::Vector :- [(:wat::cache::Cache::GetResult :- [:wat::holon::HolonAST])]
                (:wat::cache::Cache::GetResult::Hit v1)
                (:wat::cache::Cache::GetResult::Miss)
                (:wat::cache::Cache::GetResult::Hit v2)))
     ;; BATCH-OF-ONE put — overflow: k3 pushes past capacity 2; k1 is LRU (dual-evicted from the
     ;; Hologram too). `PutResponse` carries nothing back — eviction provable only via a later get.
     _put-k3 (:wat-tests::hologram-svc/assert-put-ok
               (:wat::cache::hologram-svc/put a
                 (:wat::cache::Cache::PutRequest
                   :entries (:wat::core::Vector :- [(:wat::cache::Entry :- [:wat::holon::HolonAST :wat::holon::HolonAST])]
                              (:wat::cache::Entry :key k3 :value v3)))))
     ;; BATCH-OF-ONE get + EVICTION IS VISIBLE THROUGH THE SERVICE — k1 was evicted.
     _evicted (:wat::test::assert-eq
                (:wat-tests::hologram-svc/get-results
                  (:wat::cache::hologram-svc/get b
                    (:wat::cache::Cache::GetRequest :probes (:wat::core::Vector :- [:wat::holon::HolonAST] k1))))
                (:wat::core::Vector :- [(:wat::cache::Cache::GetResult :- [:wat::holon::HolonAST])]
                  (:wat::cache::Cache::GetResult::Miss)))
     ;; EMPTY PROBE VECTOR — `Ok` with an empty results Vector, not an error.
     _empty (:wat::test::assert-eq
              (:wat-tests::hologram-svc/get-results
                (:wat::cache::hologram-svc/get b
                  (:wat::cache::Cache::GetRequest :probes (:wat::core::Vector :- [:wat::holon::HolonAST]))))
              (:wat::core::Vector :- [(:wat::cache::Cache::GetResult :- [:wat::holon::HolonAST])]))
     _ (:wat::cache::hologram-svc/stop h)]
    nil))

;; ── thread tier ────────────────────────────────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::cache-hologram-multi-client-on-thread
  (:wat-tests::hologram-svc/run (:wat::spawn::thread)))

;; ── process tier ───────────────────────────────────────────────────────────────────────────
;; The SAME sequence — tier-generality is the requirement, not a bonus: a forked child re-registers
;; the surface from the shipped `service-forms` bundle and every HolonAST payload crosses as ENCODED
;; EDN, decoded on the way in — the wire hop `probe-near-k1` must survive for gate behaviour 2 to
;; mean anything at all.
(:wat::test::deftest :wat-tests::service::cache-hologram-multi-client-on-process
  (:wat-tests::hologram-svc/run (:wat::spawn::process)))
