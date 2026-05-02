;; arc-119 stepping stone A — spawn HologramCacheService + shutdown.
;;
;; SMALLEST possible test at the consumer's vantage:
;;   1. call HologramCacheService/spawn (null-reporter, null-cadence)
;;   2. pop ONE req-tx from the pool, call HandlePool/finish
;;   3. let inner scope exit — req-tx drops → driver sees disconnect
;;   4. outer Thread/join-result unblocks
;;
;; No put. No get. Nothing but the lifecycle.
;;
;; What this proves: the post-arc-119 spawn/shutdown lifecycle is
;; intact — the driver starts, the pool mechanics work, and the
;; driver exits cleanly when its sole req-tx sender drops.
;;
;; If THIS hangs, the deadlock is in spawn/shutdown itself, independent
;; of any request shape.
;;
;; NOTE: uses (:wat::test::deftest ...) directly (not make-deftest alias)
;; so the arc-121 proc-macro scanner discovers it for per-deftest
;; cargo test filtering.

(:wat::test::time-limit "200ms")
(:wat::test::deftest :wat-tests::holon::lru::proofs::arc_119::step_A_spawn_shutdown
  ()
  (:wat::core::let*
    ;; Outer holds the driver Thread; inner owns everything else.
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((spawn :wat::holon::lru::HologramCacheService::Spawn)
          (:wat::holon::lru::HologramCacheService/spawn 1 4
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
         ((pool :wat::holon::lru::HologramCacheService::ReqTxPool)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::core::second spawn))
         ;; Pop req-tx; finish asserts pool empty.
         ((_req-tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit)
          (:wat::kernel::HandlePool::finish pool)))
        ;; _req-tx drops here — driver sees disconnect, loop exits.
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))
