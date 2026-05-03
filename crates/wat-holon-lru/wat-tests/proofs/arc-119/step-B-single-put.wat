;; arc-119 stepping stone B — step A + ONE put via helper verb.
;;
;; Adds: ONE call to HologramCacheService/put (batch-of-one entry).
;; The caller allocates a (PutAckTx, PutAckRx) pair once; the helper
;; sends Request::Put carrying ack-tx, driver acks on ack-tx, helper
;; recvs from ack-rx, returns unit.
;;
;; What this proves: the single put cycle completes — send Request::Put,
;; driver processes the batch, sends ack on ack-tx, caller recvs ack.
;; After the inner scope exits (ack-tx + req-tx drop), the driver sees
;; disconnect and the outer Thread/join-result unblocks.
;;
;; If THIS hangs but A passed, the deadlock is in the Put+ack cycle
;; specifically — either the driver doesn't ack, or the bounded(1) ack
;; channel blocks, or shutdown after a put doesn't clean up.

;; Arc 126 — this stepping stone IS the minimal reproduction of the
;; arc 126 channel-pair-deadlock pattern: req-tx + ack-tx both
;; bound in the inner scope. Arc 126's check fires at inner freeze
;; and panics with the substring `channel-pair-deadlock`. The test
;; is EXPECTED to panic with that substring; 200ms time-limit is
;; the defense-in-depth safety net.
(:wat::test::should-panic "channel-pair-deadlock")
(:wat::test::time-limit "200ms")
(:wat::test::deftest :wat-tests::holon::lru::proofs::arc_119::step_B_single_put
  ()
  ;; COMPLECTENS EXEMPT: outer let* has 1 binding (driver) + final match. The nested
  ;; inner let* IS the proof's content — each allocation is a deliberate stepping-stone
  ;; assertion; collapsing it further would destroy the proof structure this file exists
  ;; to document. Visual line count is the proof's inherent complexity, not accidental.
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
         ((req-tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit)
          (:wat::kernel::HandlePool::finish pool))

         ;; Allocate ack channel once; reused for all puts.
         ((ack-pair :wat::holon::lru::HologramCacheService::PutAckChannel)
          (:wat::kernel::make-bounded-channel :wat::core::unit 1))
         ((ack-tx :wat::holon::lru::HologramCacheService::PutAckTx)
          (:wat::core::first ack-pair))
         ((ack-rx :wat::holon::lru::HologramCacheService::PutAckRx)
          (:wat::core::second ack-pair))

         ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v :wat::holon::HolonAST) (:wat::holon::leaf :av))

         ;; ONE put — batch-of-one. HologramCacheService/put sends
         ;; Request::Put carrying ack-tx, blocks on ack-rx.
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k v)))))
        ;; Inner exits — req-tx + ack-tx + ack-rx all drop.
        ;; Driver sees disconnect on its req-rx, exits cleanly.
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))
