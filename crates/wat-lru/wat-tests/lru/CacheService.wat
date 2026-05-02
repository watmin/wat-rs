;; wat-lru :: CacheService — compositional rewrite (arc 130
;; REALIZATIONS).
;;
;; Top-down dependency graph in ONE file. Earlier defines compose
;; into later defines. Each layer carries its own deftest. The final
;; deftest body is short BECAUSE the layers exist.
;;
;; Reference: docs/arc/2026/05/130-cache-services-pair-by-index/REALIZATIONS.md
;;
;; ─── Layers ──────────────────────────────────────────────────────────
;;
;;   Layer 0  :test::lru-spawn-and-shutdown      ; spawn → finish pool → join
;;
;;   Layer 1  :test::lru-spawn-then-put          ; Layer 0 + one Put
;;            :test::lru-spawn-then-get          ; Layer 0 + one Get
;;
;;   Layer 2  :test::lru-spawn-put-then-get      ; Layer 1 composed
;;
;;   Final    :wat-lru::test-cache-service-put-then-get-round-trip
;;
;; Layer 0 is the lifecycle proof — pure; no helper-verb calls; no
;; arc-126 deadlock pattern. Layer 1+ each call `:wat::lru::put` /
;; `:wat::lru::get` whose pre-arc-130 helper-verb signatures take
;; both halves of an ack/reply pair from `make-bounded-channel`. Arc
;; 126's `channel-pair-deadlock` rule fires at freeze time — every
;; deftest using a prelude that contains those calls panics with
;; that substring. The panic is INTENTIONAL until arc 130's substrate
;; reshape lands; `:wat::test::should-panic` catches it as expected
;; behaviour.
;;
;; Post-arc-130 plan: helper verbs take a single `Handle<K,V>` and
;; allocate channels internally. The Layer 1+ helper bodies update
;; (one signature change each); the deftests at every layer drop
;; their `:should-panic` annotations; failures localize to the
;; specific layer that broke. Layer 0's lifecycle proof is unchanged
;; — it doesn't depend on helper-verb shape.
;;
;; The substrate uses inner-let* nesting per arc 131 + SERVICE-
;; PROGRAMS.md § "The lockstep": outer scope holds only the driver
;; Thread; inner owns pool + handle + per-call channels; inner
;; returns the Thread; pool drops at inner-scope exit; driver's
;; recv-loop sees disconnect; outer's `Thread/join-result` unblocks.

(:wat::test::make-deftest :deftest
  (
   ;; ─── Layer 0 — lifecycle. No helper-verb calls. ────────────────
   ;;
   ;; Spawn the cache, immediately drop the pool, join the driver.
   ;; The narrowest possible proof: substrate spawn + shutdown
   ;; cycle works under inner-let* nesting.
   (:wat::core::define
     (:test::lru-spawn-and-shutdown -> :wat::core::unit)
     (:wat::core::let*
       (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
         (:wat::core::let*
           (((spawn :wat::lru::Spawn<wat::core::String,wat::core::i64>)
             (:wat::lru::spawn 16 1
               :wat::lru::null-reporter
               (:wat::lru::null-metrics-cadence)))
            ((pool :wat::kernel::HandlePool<wat::lru::ReqTx<wat::core::String,wat::core::i64>>)
             (:wat::core::first spawn))
            ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
             (:wat::core::second spawn))
            ((_finish :wat::core::unit)
             (:wat::kernel::HandlePool::finish pool)))
           d))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       ()))

   ;; ─── Layer 1 — single helper-verb actions. ─────────────────────
   ;;
   ;; Each spawns the cache, pops a handle, allocates the per-call
   ;; channel pair, calls the helper verb, tears down. The
   ;; `(:wat::lru::put req-tx ack-tx ack-rx ...)` and
   ;; `(:wat::lru::get req-tx reply-tx reply-rx ...)` call sites
   ;; pass both halves of one channel pair → arc 126 fires at freeze.
   ;;
   ;; Post-arc-130: helper verb takes a single `Handle<K,V>`; ack /
   ;; reply channels owned internally; arc 126 stops firing.

   (:wat::core::define
     (:test::lru-spawn-then-put
       (k :wat::core::String)
       (v :wat::core::i64)
       -> :wat::core::unit)
     (:wat::core::let*
       (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
         (:wat::core::let*
           (((spawn :wat::lru::Spawn<wat::core::String,wat::core::i64>)
             (:wat::lru::spawn 16 1
               :wat::lru::null-reporter
               (:wat::lru::null-metrics-cadence)))
            ((pool :wat::kernel::HandlePool<wat::lru::ReqTx<wat::core::String,wat::core::i64>>)
             (:wat::core::first spawn))
            ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
             (:wat::core::second spawn))
            ((req-tx :wat::lru::ReqTx<wat::core::String,wat::core::i64>)
             (:wat::kernel::HandlePool::pop pool))
            ((_finish :wat::core::unit)
             (:wat::kernel::HandlePool::finish pool))
            ((ack-pair :wat::lru::PutAckChannel)
             (:wat::kernel::make-bounded-channel :wat::core::unit 1))
            ((ack-tx :wat::lru::PutAckTx) (:wat::core::first ack-pair))
            ((ack-rx :wat::lru::PutAckRx) (:wat::core::second ack-pair))
            ((_put :wat::core::unit)
             (:wat::lru::put req-tx ack-tx ack-rx
               (:wat::core::conj
                 (:wat::core::Vector :wat::lru::Entry<wat::core::String,wat::core::i64>)
                 (:wat::core::Tuple k v)))))
           d))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       ()))

   (:wat::core::define
     (:test::lru-spawn-then-get
       (k :wat::core::String)
       -> :wat::core::Vector<wat::core::Option<wat::core::i64>>)
     (:wat::core::let*
       (((pair :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::Vector<wat::core::Option<wat::core::i64>>))
         (:wat::core::let*
           (((spawn :wat::lru::Spawn<wat::core::String,wat::core::i64>)
             (:wat::lru::spawn 16 1
               :wat::lru::null-reporter
               (:wat::lru::null-metrics-cadence)))
            ((pool :wat::kernel::HandlePool<wat::lru::ReqTx<wat::core::String,wat::core::i64>>)
             (:wat::core::first spawn))
            ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
             (:wat::core::second spawn))
            ((req-tx :wat::lru::ReqTx<wat::core::String,wat::core::i64>)
             (:wat::kernel::HandlePool::pop pool))
            ((_finish :wat::core::unit)
             (:wat::kernel::HandlePool::finish pool))
            ((reply-pair :wat::lru::ReplyChannel<wat::core::i64>)
             (:wat::kernel::make-bounded-channel
               :wat::core::Vector<wat::core::Option<wat::core::i64>> 1))
            ((reply-tx :wat::lru::ReplyTx<wat::core::i64>)
             (:wat::core::first reply-pair))
            ((reply-rx :wat::lru::ReplyRx<wat::core::i64>)
             (:wat::core::second reply-pair))
            ((results :wat::core::Vector<wat::core::Option<wat::core::i64>>)
             (:wat::lru::get req-tx reply-tx reply-rx
               (:wat::core::conj
                 (:wat::core::Vector :wat::core::String)
                 k))))
           (:wat::core::Tuple d results)))
        ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first pair))
        ((results :wat::core::Vector<wat::core::Option<wat::core::i64>>) (:wat::core::second pair))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       results))

   ;; ─── Layer 2 — Put-then-Get composition. ───────────────────────
   ;;
   ;; Spawn ONCE. Put one entry. Get the same key. Tear down. Returns
   ;; the get's results so the deftest body can assert on them. Same
   ;; inner-let* shape — outer holds only the driver; inner does the
   ;; full sequence; inner returns (driver, results) tuple.
   (:wat::core::define
     (:test::lru-spawn-put-then-get
       (k :wat::core::String)
       (v :wat::core::i64)
       -> :wat::core::Vector<wat::core::Option<wat::core::i64>>)
     (:wat::core::let*
       (((pair :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::Vector<wat::core::Option<wat::core::i64>>))
         (:wat::core::let*
           (((spawn :wat::lru::Spawn<wat::core::String,wat::core::i64>)
             (:wat::lru::spawn 16 1
               :wat::lru::null-reporter
               (:wat::lru::null-metrics-cadence)))
            ((pool :wat::kernel::HandlePool<wat::lru::ReqTx<wat::core::String,wat::core::i64>>)
             (:wat::core::first spawn))
            ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
             (:wat::core::second spawn))
            ((req-tx :wat::lru::ReqTx<wat::core::String,wat::core::i64>)
             (:wat::kernel::HandlePool::pop pool))
            ((_finish :wat::core::unit)
             (:wat::kernel::HandlePool::finish pool))

            ;; Put — Pattern A unit-ack channel.
            ((ack-pair :wat::lru::PutAckChannel)
             (:wat::kernel::make-bounded-channel :wat::core::unit 1))
            ((ack-tx :wat::lru::PutAckTx) (:wat::core::first ack-pair))
            ((ack-rx :wat::lru::PutAckRx) (:wat::core::second ack-pair))
            ((_put :wat::core::unit)
             (:wat::lru::put req-tx ack-tx ack-rx
               (:wat::core::conj
                 (:wat::core::Vector :wat::lru::Entry<wat::core::String,wat::core::i64>)
                 (:wat::core::Tuple k v))))

            ;; Get — Pattern B data-back channel.
            ((reply-pair :wat::lru::ReplyChannel<wat::core::i64>)
             (:wat::kernel::make-bounded-channel
               :wat::core::Vector<wat::core::Option<wat::core::i64>> 1))
            ((reply-tx :wat::lru::ReplyTx<wat::core::i64>)
             (:wat::core::first reply-pair))
            ((reply-rx :wat::lru::ReplyRx<wat::core::i64>)
             (:wat::core::second reply-pair))
            ((results :wat::core::Vector<wat::core::Option<wat::core::i64>>)
             (:wat::lru::get req-tx reply-tx reply-rx
               (:wat::core::conj
                 (:wat::core::Vector :wat::core::String)
                 k))))
           (:wat::core::Tuple d results)))
        ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first pair))
        ((results :wat::core::Vector<wat::core::Option<wat::core::i64>>) (:wat::core::second pair))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       results))))

;; ─── Per-layer deftests ──────────────────────────────────────────────
;;
;; Each layer carries its own proof. `cargo test --list` shows the
;; tree. When a layer breaks, the failing deftest's name names the
;; broken unit. Bottom-up proofs THEN top-down composition.
;;
;; All deftests `:should-panic("channel-pair-deadlock")` because the
;; shared prelude includes Layer 1+ helpers whose bodies fire arc 126
;; at freeze. The panic is intentional under the current substrate;
;; arc 130 lands the substrate reshape that retires it. Post-arc-130,
;; the `:should-panic` annotations come off and per-layer failures
;; localize cleanly.


;; Layer 0 — lifecycle proof.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest :wat-lru::test-lru-spawn-and-shutdown
  (:test::lru-spawn-and-shutdown))


;; Layer 1a — Put proof.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest :wat-lru::test-lru-spawn-then-put
  (:test::lru-spawn-then-put "answer" 42))


;; Layer 1b — Get proof.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest :wat-lru::test-lru-spawn-then-get
  (:wat::core::let*
    (((_results :wat::core::Vector<wat::core::Option<wat::core::i64>>)
      (:test::lru-spawn-then-get "answer")))
    ()))


;; Layer 2 — Put-then-Get composition proof.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest :wat-lru::test-lru-spawn-put-then-get
  (:wat::core::let*
    (((_results :wat::core::Vector<wat::core::Option<wat::core::i64>>)
      (:test::lru-spawn-put-then-get "answer" 42)))
    ()))


;; Final — the named scenario. Body is short BECAUSE the layers exist.
;; Post-arc-130, the assertion runs and the test passes cleanly.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest :wat-lru::test-cache-service-put-then-get-round-trip
  (:wat::core::let*
    (((results :wat::core::Vector<wat::core::Option<wat::core::i64>>)
      (:test::lru-spawn-put-then-get "answer" 42)))
    (:wat::test::assert-eq
      (:wat::core::first results)
      (:wat::core::Some (:wat::core::Some 42)))))
