;; wat-tests for arc 078 — :wat::holon::lru::HologramCacheService.
;;
;; Six-step progression building up the queue-addressed cache wrapper:
;;   1. spawn + join, no channels
;;   2. counted recv via nested let* (channel mechanics only)
;;   3. Service/loop drives the Request enum (Put-only)
;;   4. Put + Get round-trip via reply-tx
;;   5. Service/spawn constructor + HandlePool fan-in (multi-client)
;;   6. LRU eviction visible through Service Get/Put round-trips
;;
;; Helpers are spliced into each test via make-deftest because
;; deftest's sandbox does not carry top-level defines from the outer
;; file (per arc 075's closure note).

(:wat::test::make-deftest :deftest-hermetic
  (
   ;; ─── Step 1 helper ──────────────────────────────────────────
   ;; A trivial worker — make a HologramCache and return it. Verifies
   ;; spawn + join with HologramCache as the return type, without any
   ;; channel complexity.
   (:wat::core::define
     (:wat-tests::holon::lru::HologramCacheService::trivial-worker
       -> :wat::holon::lru::HologramCache)
     (:wat::holon::lru::HologramCache/make
       (:wat::holon::filter-coincident)
       16))

   ;; ─── Step 2 helpers — counted recv loop ─────────────────────
   ;; Worker recv-loops over an i64 channel, counting each Some.
   ;; Returns count when all senders have dropped (channel closed).
   (:wat::core::define
     (:wat-tests::holon::lru::HologramCacheService::count-recv
       (rx :wat::kernel::QueueReceiver<i64>)
       (acc :wat::core::i64)
       -> :wat::core::i64)
     (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::i64
       ((Some _v)
         (:wat-tests::holon::lru::HologramCacheService::count-recv
           rx (:wat::core::i64::+ acc 1)))
       (:None acc)))

   (:wat::core::define
     (:wat-tests::holon::lru::HologramCacheService::run-counter
       (rx :wat::kernel::QueueReceiver<i64>) -> :wat::core::i64)
     (:wat-tests::holon::lru::HologramCacheService::count-recv rx 0))

   ;; ─── Step 3 helper — drive Service/loop, return final len ──
   ;; HologramCache is thread-owned; we cannot return the cache itself
   ;; across the join boundary. Compute len inside the worker; only
   ;; the i64 crosses. Pass null-reporter + null-metrics-cadence —
   ;; these tests don't care about reporting.
   (:wat::core::define
     (:wat-tests::holon::lru::HologramCacheService::run-loop-then-len
       (req-rxs :Vec<wat::holon::lru::HologramCacheService::ReqRx>)
       (cap :wat::core::i64)
       -> :wat::core::i64)
     (:wat::core::let*
       (((cache :wat::holon::lru::HologramCache)
         (:wat::holon::lru::HologramCache/make
           (:wat::holon::filter-coincident)
           cap))
        ((initial :wat::holon::lru::HologramCacheService::State)
         (:wat::holon::lru::HologramCacheService::State/new
           cache (:wat::holon::lru::HologramCacheService::Stats/zero)))
        ((final :wat::holon::lru::HologramCacheService::State)
         (:wat::holon::lru::HologramCacheService/loop
           req-rxs initial
           :wat::holon::lru::HologramCacheService/null-reporter
           (:wat::holon::lru::HologramCacheService/null-metrics-cadence))))
       (:wat::holon::lru::HologramCache/len
         (:wat::holon::lru::HologramCacheService::State/cache final))))))

;; ─── Step 1 — spawn + join, no channels ─────────────────────────

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step1-spawn-join
  (:wat::core::let*
    (((handle :wat::kernel::ProgramHandle<wat::holon::lru::HologramCache>)
      (:wat::kernel::spawn
        :wat-tests::holon::lru::HologramCacheService::trivial-worker)))
    (:wat::core::match (:wat::kernel::join-result handle) -> :()
      ((Ok _cache) ())
      ((Err _) (:wat::test::assert-eq "spawn-died" "")))))

;; ─── Step 2 — counted recv via nested let* ──────────────────────

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step2-counted-recv
  (:wat::core::let*
    (((handle :wat::kernel::ProgramHandle<i64>)
      (:wat::core::let*
        (((pair :wat::kernel::QueuePair<i64>)
          (:wat::kernel::make-bounded-queue :wat::core::i64 1))
         ((tx :wat::kernel::QueueSender<i64>) (:wat::core::first pair))
         ((rx :wat::kernel::QueueReceiver<i64>) (:wat::core::second pair))
         ((h :wat::kernel::ProgramHandle<i64>)
          (:wat::kernel::spawn
            :wat-tests::holon::lru::HologramCacheService::run-counter rx))
         ((_s1 :wat::kernel::Sent) (:wat::kernel::send tx 10))
         ((_s2 :wat::kernel::Sent) (:wat::kernel::send tx 20))
         ((_s3 :wat::kernel::Sent) (:wat::kernel::send tx 30)))
        h)))
    (:wat::core::match (:wat::kernel::join-result handle) -> :()
      ((Ok 3) ())
      ((Ok _) (:wat::test::assert-eq "wrong-count" ""))
      ((Err _) (:wat::test::assert-eq "worker-died" "")))))

;; ─── Step 3 — Service/loop drives the real Request enum (Put only) ──

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step3-put-only
  (:wat::core::let*
    (((handle :wat::kernel::ProgramHandle<i64>)
      (:wat::core::let*
        (((pair :wat::kernel::QueuePair<wat::holon::lru::HologramCacheService::Request>)
          (:wat::kernel::make-bounded-queue
            :wat::holon::lru::HologramCacheService::Request 1))
         ((tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::core::first pair))
         ((rx :wat::holon::lru::HologramCacheService::ReqRx)
          (:wat::core::second pair))
         ((rxs :Vec<wat::holon::lru::HologramCacheService::ReqRx>)
          (:wat::core::conj
            (:wat::core::vec :wat::holon::lru::HologramCacheService::ReqRx)
            rx))
         ((h :wat::kernel::ProgramHandle<i64>)
          (:wat::kernel::spawn
            :wat-tests::holon::lru::HologramCacheService::run-loop-then-len rxs 16))
         ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :av))
         ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :beta))
         ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :bv))
         ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :gamma))
         ((v3 :wat::holon::HolonAST) (:wat::holon::leaf :gv))
         ((_p1 :wat::kernel::Sent)
          (:wat::kernel::send tx
            (:wat::holon::lru::HologramCacheService::Request::Put k1 v1)))
         ((_p2 :wat::kernel::Sent)
          (:wat::kernel::send tx
            (:wat::holon::lru::HologramCacheService::Request::Put k2 v2)))
         ((_p3 :wat::kernel::Sent)
          (:wat::kernel::send tx
            (:wat::holon::lru::HologramCacheService::Request::Put k3 v3))))
        h)))
    (:wat::core::match (:wat::kernel::join-result handle) -> :()
      ((Ok 3) ())
      ((Ok _) (:wat::test::assert-eq "wrong-len" ""))
      ((Err _) (:wat::test::assert-eq "service-died" "")))))

;; ─── Step 4 — Put then Get round-trip via reply-tx ──────────────

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step4-put-get-roundtrip
  (:wat::core::let*
    (((handle :wat::kernel::ProgramHandle<()>)
      (:wat::core::let*
        (((req-pair :wat::kernel::QueuePair<wat::holon::lru::HologramCacheService::Request>)
          (:wat::kernel::make-bounded-queue
            :wat::holon::lru::HologramCacheService::Request 1))
         ((req-tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::core::first req-pair))
         ((req-rx :wat::holon::lru::HologramCacheService::ReqRx)
          (:wat::core::second req-pair))
         ((rxs :Vec<wat::holon::lru::HologramCacheService::ReqRx>)
          (:wat::core::conj
            (:wat::core::vec :wat::holon::lru::HologramCacheService::ReqRx)
            req-rx))
         ((h :wat::kernel::ProgramHandle<()>)
          (:wat::kernel::spawn
            :wat::holon::lru::HologramCacheService/run rxs 16
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))

         ((reply-pair :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-queue :Option<wat::holon::HolonAST> 1))
         ((reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair))
         ((reply-rx :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair))

         ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v :wat::holon::HolonAST) (:wat::holon::leaf :av))

         ((_p :wat::kernel::Sent)
          (:wat::kernel::send req-tx
            (:wat::holon::lru::HologramCacheService::Request::Put k v)))
         ((_g :wat::kernel::Sent)
          (:wat::kernel::send req-tx
            (:wat::holon::lru::HologramCacheService::Request::Get k reply-tx)))
         ((maybe-reply :Option<Option<wat::holon::HolonAST>>)
          (:wat::kernel::recv reply-rx))
         ((_check :())
          (:wat::core::match maybe-reply -> :()
            ((Some inner)
              (:wat::core::match inner -> :()
                ((Some _val) ())
                (:None (:wat::test::assert-eq "cache-miss" ""))))
            (:None (:wat::test::assert-eq "no-reply" "")))))
        h)))
    (:wat::core::match (:wat::kernel::join-result handle) -> :()
      ((Ok _) ())
      ((Err _) (:wat::test::assert-eq "service-died" "")))))

;; ─── Step 5 — full Service constructor + HandlePool fan-in ──────

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step5-multi-client-via-constructor
  (:wat::core::let*
    (((handle :wat::kernel::ProgramHandle<()>)
      (:wat::core::let*
        (((spawn :wat::holon::lru::HologramCacheService::Spawn)
          (:wat::holon::lru::HologramCacheService/spawn 2 16
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
         ((pool :wat::holon::lru::HologramCacheService::ReqTxPool)
          (:wat::core::first spawn))
         ((driver :wat::kernel::ProgramHandle<()>)
          (:wat::core::second spawn))

         ((tx-a :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((tx-b :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool))

         ((reply-pair-a :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-queue :Option<wat::holon::HolonAST> 1))
         ((reply-tx-a :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair-a))
         ((reply-rx-a :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair-a))

         ((reply-pair-b :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-queue :Option<wat::holon::HolonAST> 1))
         ((reply-tx-b :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair-b))
         ((reply-rx-b :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair-b))

         ((k-a :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v-a :wat::holon::HolonAST) (:wat::holon::leaf :av))
         ((k-b :wat::holon::HolonAST) (:wat::holon::leaf :beta))
         ((v-b :wat::holon::HolonAST) (:wat::holon::leaf :bv))

         ;; Client A: Put + Get on alpha
         ((_pa :wat::kernel::Sent)
          (:wat::kernel::send tx-a
            (:wat::holon::lru::HologramCacheService::Request::Put k-a v-a)))
         ((_ga :wat::kernel::Sent)
          (:wat::kernel::send tx-a
            (:wat::holon::lru::HologramCacheService::Request::Get k-a reply-tx-a)))
         ((reply-a :Option<Option<wat::holon::HolonAST>>)
          (:wat::kernel::recv reply-rx-a))
         ((_check-a :())
          (:wat::core::match reply-a -> :()
            ((Some inner)
              (:wat::core::match inner -> :()
                ((Some _val) ())
                (:None (:wat::test::assert-eq "client-a-miss" ""))))
            (:None (:wat::test::assert-eq "client-a-no-reply" ""))))

         ;; Client B: Put + Get on beta
         ((_pb :wat::kernel::Sent)
          (:wat::kernel::send tx-b
            (:wat::holon::lru::HologramCacheService::Request::Put k-b v-b)))
         ((_gb :wat::kernel::Sent)
          (:wat::kernel::send tx-b
            (:wat::holon::lru::HologramCacheService::Request::Get k-b reply-tx-b)))
         ((reply-b :Option<Option<wat::holon::HolonAST>>)
          (:wat::kernel::recv reply-rx-b))
         ((_check-b :())
          (:wat::core::match reply-b -> :()
            ((Some inner)
              (:wat::core::match inner -> :()
                ((Some _val) ())
                (:None (:wat::test::assert-eq "client-b-miss" ""))))
            (:None (:wat::test::assert-eq "client-b-no-reply" "")))))
        driver)))
    (:wat::core::match (:wat::kernel::join-result handle) -> :()
      ((Ok _) ())
      ((Err _) (:wat::test::assert-eq "service-died" "")))))

;; ─── Step 6 — LRU eviction visible through Service Get/Put round-trips ──
;;
;; cap=2 cache; Put k1, Put k2, Put k3 — k1 should be evicted.
;; Subsequent Get(k1) returns None; Get(k2) returns Some. This proves
;; the queue-addressed wrapper preserves the HologramCache eviction
;; semantics — eviction visible from the client's view, not just at
;; the substrate.

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step6-lru-eviction-via-service
  (:wat::core::let*
    (((handle :wat::kernel::ProgramHandle<()>)
      (:wat::core::let*
        (((spawn :wat::holon::lru::HologramCacheService::Spawn)
          (:wat::holon::lru::HologramCacheService/spawn 1 2
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
         ((pool :wat::holon::lru::HologramCacheService::ReqTxPool)
          (:wat::core::first spawn))
         ((driver :wat::kernel::ProgramHandle<()>)
          (:wat::core::second spawn))
         ((tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool))

         ((reply-pair :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-queue :Option<wat::holon::HolonAST> 1))
         ((reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair))
         ((reply-rx :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair))

         ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
         ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
         ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :third))
         ((v :wat::holon::HolonAST) (:wat::holon::leaf :payload))

         ;; Three puts at cap=2; k1 gets evicted by k3.
         ((_p1 :wat::kernel::Sent)
          (:wat::kernel::send tx
            (:wat::holon::lru::HologramCacheService::Request::Put k1 v)))
         ((_p2 :wat::kernel::Sent)
          (:wat::kernel::send tx
            (:wat::holon::lru::HologramCacheService::Request::Put k2 v)))
         ((_p3 :wat::kernel::Sent)
          (:wat::kernel::send tx
            (:wat::holon::lru::HologramCacheService::Request::Put k3 v)))

         ;; Get k1 — evicted, expect None.
         ((_g1 :wat::kernel::Sent)
          (:wat::kernel::send tx
            (:wat::holon::lru::HologramCacheService::Request::Get k1 reply-tx)))
         ((reply-1 :Option<Option<wat::holon::HolonAST>>)
          (:wat::kernel::recv reply-rx))
         ((_check-1 :())
          (:wat::core::match reply-1 -> :()
            ((Some inner)
              (:wat::core::match inner -> :()
                ((Some _) (:wat::test::assert-eq "k1-not-evicted" ""))
                (:None ())))
            (:None (:wat::test::assert-eq "no-reply-1" ""))))

         ;; Get k2 — survived, expect Some.
         ((_g2 :wat::kernel::Sent)
          (:wat::kernel::send tx
            (:wat::holon::lru::HologramCacheService::Request::Get k2 reply-tx)))
         ((reply-2 :Option<Option<wat::holon::HolonAST>>)
          (:wat::kernel::recv reply-rx))
         ((_check-2 :())
          (:wat::core::match reply-2 -> :()
            ((Some inner)
              (:wat::core::match inner -> :()
                ((Some _) ())
                (:None (:wat::test::assert-eq "k2-evicted" ""))))
            (:None (:wat::test::assert-eq "no-reply-2" "")))))
        driver)))
    (:wat::core::match (:wat::kernel::join-result handle) -> :()
      ((Ok _) ())
      ((Err _) (:wat::test::assert-eq "service-died" "")))))
