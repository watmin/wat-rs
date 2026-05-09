;; arc-119 stepping stone A — spawn HologramCacheService + shutdown.
;;
;; SMALLEST possible test at the consumer's vantage:
;;   1. call HologramCacheService/spawn (null-reporter, null-cadence)
;;   2. pop ONE Handle from the pool, call HandlePool/finish
;;   3. let inner scope exit — Handle drops → driver sees disconnect
;;   4. outer Thread/join-result unblocks
;;
;; No put. No get. Nothing but the lifecycle.
;;
;; What this proves: the post-arc-130 spawn/shutdown lifecycle is
;; intact — the driver starts, the pool mechanics work, and the
;; driver exits cleanly when its sole Handle (ReqTx, ReplyRx) drops.
;;
;; If THIS hangs, the deadlock is in spawn/shutdown itself, independent
;; of any request shape.
;;
;; Arc 130: Handle = (ReqTx, ReplyRx) — pair-by-index. The pool gives
;; out Handles, not bare ReqTxs. Driver holds matching DriverPair =
;; (ReqRx, ReplyTx) at the same index.
;;
;; NOTE: uses (:wat::test::deftest ...) directly (not make-deftest alias)
;; so the arc-121 proc-macro scanner discovers it for per-deftest
;; cargo test filtering.

(:wat::test::time-limit "200ms")
(:wat::test::deftest :wat-tests::holon::lru::proofs::arc_119::step_A_spawn_shutdown
  ()
  (:wat::core::let
    ;; Outer holds the driver Thread; inner owns everything else.
    [driver
      (:wat::core::let
        [spawn
          (:wat::holon::lru::HologramCacheService/spawn 1 4
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence))
         pool
          (:wat::core::first spawn)
         d
          (:wat::core::second spawn)
         ;; Pop Handle; finish asserts pool empty.
         _handle
          (:wat::kernel::HandlePool::pop pool)
         _finish
          (:wat::kernel::HandlePool::finish pool)]
        ;; _handle drops here — driver sees disconnect, loop exits.
        d)]
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::nil
      ((:wat::core::Ok _) :wat::core::nil)
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))
