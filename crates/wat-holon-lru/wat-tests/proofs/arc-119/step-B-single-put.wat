;; arc-119 stepping stone B — step A + ONE put via helper verb.
;;
;; Adds: ONE call to HologramCacheService/put (batch-of-one entry).
;; Post-arc-130: the helper verb takes Handle = (ReqTx, ReplyRx);
;; channels are pre-allocated by spawn (pair-by-index). The helper
;; sends Request::Put on req-tx, the driver replies Reply::PutAck
;; on the slot's reply-tx, the helper recvs from reply-rx, returns
;; unit. NO caller-allocated ack channel — that pattern is gone.
;;
;; What this proves: the single put cycle completes — send Request::Put,
;; driver processes the batch, sends PutAck on reply-tx, caller recvs
;; PutAck. After the inner scope exits (Handle drops), the driver sees
;; disconnect and the outer Thread/join-result unblocks.
;;
;; If THIS hangs but A passed, the deadlock is in the Put+ack cycle
;; specifically — either the driver doesn't ack, or the bounded(1) reply
;; channel blocks, or shutdown after a put doesn't clean up.
;;
;; Arc 130: the channel-pair-deadlock pattern is GONE because the helper
;; verb does send-AND-recv internally on a single Handle. No
;; :should-panic annotation needed — the test passes naturally.

(:wat::test::time-limit "200ms")
(:wat::test::deftest :wat-tests::holon::lru::proofs::arc_119::step_B_single_put
  ()
  (:wat::core::let*
    ;; Outer holds the driver Thread; inner owns everything else.
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((spawn :wat::holon::lru::HologramCacheService::Spawn)
          (:wat::holon::lru::HologramCacheService/spawn 1 4
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
         ((pool :wat::kernel::HandlePool<wat::holon::lru::HologramCacheService::Handle>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::core::second spawn))
         ((handle :wat::holon::lru::HologramCacheService::Handle)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit)
          (:wat::kernel::HandlePool::finish pool))

         ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v :wat::holon::HolonAST) (:wat::holon::leaf :av))

         ;; ONE put — batch-of-one. HologramCacheService/put sends
         ;; Request::Put on the slot's req-tx and blocks on reply-rx
         ;; until the driver replies Reply::PutAck.
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put handle
            (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry
              (:wat::core::Tuple k v)))))
        ;; Inner exits — handle drops. Driver sees disconnect on its
        ;; req-rx, exits cleanly.
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))
