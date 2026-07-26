;; wat-tests/service-cache-hologram.wat — arc 278 cache Stone 4: the SIMILARITY cache, as a service.
;;
;; Stone 3 (`wat-tests/cache/HolographicLru.wat`) proved `:wat::cache::HolographicLru` in-process.
;; Stone 4 puts it behind the SAME `:wat::cache::Cache<K,V>` multi-client surface Stone 2's
;; `lru-svc<K,V>` wears (`wat-tests/service-cache-lru.wat`), pinned concretely at
;; `Cache<wat::holon::HolonAST,wat::holon::HolonAST>` (`:wat::cache::hologram-svc`, no `<K,V>` of
;; its own — `HolographicLru` is concrete over `HolonAST`, so the service is too).
;;
;; Four load-bearing behaviours, ONE round trip, two dialed clients A and B (mirrors Stone 2's
;; shape — steal the `dial` idiom, the separately-typed verb that pins the wire's type args):
;;
;;   1. MULTI-CLIENT     — A `put`s k1 (a Thermometer @ 50.0); B, a DIFFERENT client off the same
;;                          `Handle/addr`, `get`s the SAME key and sees A's write.
;;   2. ★ SIMILARITY ACROSS THE WIRE — B then probes with a DIFFERENT (not equal) but coincident
;;                          HolonAST (Thermometer @ 50.01) and still hits k1's value. This is the
;;                          one that matters: no in-process test can show the similarity match
;;                          surviving encode -> wire -> decode.
;;   3. MISS IS A VALUE  — B probes with something never coincident with anything present and gets
;;                          the `Miss` variant, not an error.
;;   4. EVICTION IS VISIBLE THROUGH THE SERVICE — capacity 2; a third distinct key (`k3`) overflows
;;                          and evicts the LRU entry (`k1`, since B's two prior `get`s both bumped
;;                          it to MRU and then k2's `put` filled the second slot); a later `get` of
;;                          `k1` shows the dual-eviction invariant holding *through the actor*.
;;
;; `HolographicLru::put` returns `nil` (unlike Stone 1's `Lru::put`) — the dual-eviction chain
;; removes the displaced key from the Hologram internally but never hands it back. So
;; `hologram-svc`'s `put` impl always answers `Ok :displaced None`: an honest reflection of what
;; the primitive actually exposes (wat/cache.wat's Stone 4 section), not a `Some(Entry)` lie
;; dressed up to mirror `lru-svc`. Eviction is proven the only way it CAN be proven here — a
;; later `get` miss — which is exactly gate behaviour 4 above.
;;
;; Assert on structure exactly (never a rendered-string `contains`): each response is unwrapped by
;; a small per-shape assertion helper that pattern-matches the enum and compares the carried
;; HolonAST / Option<Entry> by VALUE, dying loud (`assertion-failed!`) on a wire breach
;; (Lost/Closed) or an unexpected response variant (RequestTooLarge / RequestMalformed) — same
;; discipline as `service-cache-lru.wat`'s labels, just structural equality instead of string
;; rendering (HolonAST has no natural string form, and stringifying it to `contains`-match would
;; be exactly the anti-pattern the brief rules out).

;; ── dial — the separately-typed verb, load-bearing (pins the wire's type args) ────────────────
(:wat::core::defn :wat-tests::hologram-svc/dial
  [a <- :wat::kernel::Address'<wat::cache::Cache::Op<wat::holon::HolonAST,wat::holon::HolonAST>,wat::cache::Cache::Reply<wat::holon::HolonAST,wat::holon::HolonAST>>]
  -> :wat::kernel::Peer'<wat::cache::Cache::Op<wat::holon::HolonAST,wat::holon::HolonAST>,wat::cache::Cache::Reply<wat::holon::HolonAST,wat::holon::HolonAST>>
  (:wat::core::match (:wat::kernel::connect' a)
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed cz)
      (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cz) :wat::core::None :wat::core::None))))

;; ── assertion helpers — unwrap RecvOutcome, then assert the STRUCTURE, dying loud on a breach ──

(:wat::core::defn :wat-tests::hologram-svc/assert-hit
  [r        <- :wat::kernel::RecvOutcome<wat::cache::Cache::GetResponse<wat::holon::HolonAST>>
   expected <- :wat::holon::HolonAST]
  -> :wat::core::nil
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::cache::Cache::GetResponse::Hit v) (:wat::test::assert-eq v expected))
        ((:wat::cache::Cache::GetResponse::Miss) (:wat::test::assert-eq :expected-hit :got-miss))
        ((:wat::cache::Cache::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "hologram-svc get: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat::cache::Cache::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "hologram-svc get: unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :wat-tests::hologram-svc/assert-miss
  [r <- :wat::kernel::RecvOutcome<wat::cache::Cache::GetResponse<wat::holon::HolonAST>>]
  -> :wat::core::nil
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::cache::Cache::GetResponse::Hit v) (:wat::test::assert-eq :expected-miss :got-hit))
        ((:wat::cache::Cache::GetResponse::Miss) nil)
        ((:wat::cache::Cache::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "hologram-svc get: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat::cache::Cache::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "hologram-svc get: unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :wat-tests::hologram-svc/assert-put-ok
  [r <- :wat::kernel::RecvOutcome<wat::cache::Cache::PutResponse<wat::holon::HolonAST,wat::holon::HolonAST>>]
  -> :wat::core::nil
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat::cache::Cache::PutResponse::Ok displaced) (:wat::test::assert-eq displaced :wat::core::None))
        ((:wat::cache::Cache::PutResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "hologram-svc put: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat::cache::Cache::PutResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "hologram-svc put: unexpected RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

;; ── the gate: ONE service, TWO clients, the four behaviours in one round trip ─────────────────
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
     ;; never put; not coincident with anything the store will hold.
     probe-far (:wat::holon::leaf :nope)

     _put-k1 (:wat-tests::hologram-svc/assert-put-ok
               (:wat::cache::hologram-svc/put a (:wat::cache::Cache::PutRequest :key k1 :value v1)))
     ;; (1) MULTI-CLIENT — B, a DIFFERENT client off the same addr, sees A's exact write.
     _get-k1-by-b (:wat-tests::hologram-svc/assert-hit
                    (:wat::cache::hologram-svc/get b (:wat::cache::Cache::GetRequest :key k1)) v1)
     ;; (2) ★ SIMILARITY ACROSS THE WIRE — B probes with a coincident but DIFFERENT HolonAST and
     ;; still hits k1's value; this also bumps k1 to MRU inside the actor.
     _get-probe-near-k1-by-b (:wat-tests::hologram-svc/assert-hit
                                (:wat::cache::hologram-svc/get b
                                  (:wat::cache::Cache::GetRequest :key probe-near-k1)) v1)
     ;; fills the cache to capacity: {k1(LRU), k2(MRU)}.
     _put-k2 (:wat-tests::hologram-svc/assert-put-ok
               (:wat::cache::hologram-svc/put a (:wat::cache::Cache::PutRequest :key k2 :value v2)))
     ;; overflow — k3 pushes past capacity 2; k1 is LRU (dual-evicted from the Hologram too).
     _put-k3 (:wat-tests::hologram-svc/assert-put-ok
               (:wat::cache::hologram-svc/put a (:wat::cache::Cache::PutRequest :key k3 :value v3)))
     ;; (4) EVICTION IS VISIBLE THROUGH THE SERVICE — k1 was evicted; a later get is a Miss.
     _get-k1-evicted (:wat-tests::hologram-svc/assert-miss
                        (:wat::cache::hologram-svc/get b (:wat::cache::Cache::GetRequest :key k1)))
     ;; (3) MISS IS A VALUE — a probe never coincident with anything present.
     _get-probe-far (:wat-tests::hologram-svc/assert-miss
                       (:wat::cache::hologram-svc/get b (:wat::cache::Cache::GetRequest :key probe-far)))
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
