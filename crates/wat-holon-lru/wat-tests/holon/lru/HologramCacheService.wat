;; wat-tests for arc 078 — :wat::holon::lru::HologramCacheService.
;;
;; Six-step progression building up the queue-addressed cache wrapper.
;; Post-arc-114: every step uses :wat::kernel::spawn-thread; the body
;; lambda fits :Fn(:Receiver<I>, :Sender<O>) -> :() so values flow
;; through the substrate's typed pipes (mini-TCP discipline,
;; docs/ZERO-MUTEX.md) instead of via a retired R-via-join contract.
;;
;;   1. spawn-thread + Thread/join-result, no caller channels —
;;      worker creates a HologramCache, sends its len on `out`
;;      so the parent observes "the worker ran without dying"
;;   2. counted recv via the substrate input pipe — parent sends
;;      i64s on Thread/input, worker counts, sends count on `out`
;;   3. Service/loop drives the Request enum (Put-only); final len
;;      delivered on the substrate's `out` Sender
;;   4. Put + Get round-trip via reply-tx (per-request mini-TCP
;;      embedded in the payload)
;;   5. Service/spawn constructor + HandlePool fan-in (multi-client)
;;   6. LRU eviction visible through Service Get/Put round-trips
;;
;; Helpers are spliced into each test via make-deftest because
;; deftest's sandbox does not carry top-level defines from the outer
;; file (per arc 075's closure note).

(:wat::test::make-deftest :deftest-hermetic
  (
   ;; ─── Step 1 helper ──────────────────────────────────────────
   ;; Worker creates a HologramCache (thread-owned — never crosses
   ;; the boundary), then sends its len on the substrate's output
   ;; pipe. The non-zero-on-success len is the "I ran without dying"
   ;; signal; an empty cache reads len 0, which is fine — the test
   ;; only verifies receipt.
   (:wat::core::define
     (:wat-tests::holon::lru::HologramCacheService::trivial-worker
       (_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
       (out :rust::crossbeam_channel::Sender<wat::core::i64>)
       -> :wat::core::unit)
     (:wat::core::let*
       (((cache :wat::holon::lru::HologramCache)
         (:wat::holon::lru::HologramCache/make
           (:wat::holon::filter-coincident)
           16))
        ((len :wat::core::i64) (:wat::holon::lru::HologramCache/len cache)))
       (:wat::core::Result/expect -> :wat::core::unit
         (:wat::kernel::send out len)
         "trivial-worker: out disconnected — parent dropped Thread/output?")))

   ;; ─── Step 2 helpers — counted recv loop ─────────────────────
   ;; Worker recv-loops over a CALLER-ALLOCATED channel (rx from
   ;; make-bounded-queue), counting each Some. When the caller's
   ;; inner scope drops, all Sender clones go with it; the worker
   ;; recvs Ok(:None) and sends the total count on the substrate's
   ;; output pipe.
   ;;
   ;; Why caller-allocated and not substrate-allocated input pipe:
   ;; the substrate's Thread struct holds a clone of the substrate-
   ;; allocated Sender alongside the caller's clone, so dropping
   ;; the caller's Thread/input handle doesn't disconnect the
   ;; channel — Thread struct's clone keeps it alive. Streaming /
   ;; loop-until-EOF workers need every Sender to drop together;
   ;; that only happens cleanly with caller-allocated queues whose
   ;; sole owner is the caller's inner let* scope. See
   ;; SERVICE-PROGRAMS.md § "The lockstep".
   (:wat::core::define
     (:wat-tests::holon::lru::HologramCacheService::count-recv
       (rx :wat::kernel::QueueReceiver<wat::core::i64>)
       (acc :wat::core::i64)
       -> :wat::core::i64)
     (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::i64
       ((:wat::core::Ok (:wat::core::Some _v))
         (:wat-tests::holon::lru::HologramCacheService::count-recv
           rx (:wat::core::i64::+ acc 1)))
       ((:wat::core::Ok :wat::core::None) acc)
       ((:wat::core::Err _died) acc)))

   (:wat::core::define
     (:wat-tests::holon::lru::HologramCacheService::counter-worker
       (rx :wat::kernel::QueueReceiver<wat::core::i64>)
       (out :rust::crossbeam_channel::Sender<wat::core::i64>)
       -> :wat::core::unit)
     (:wat::core::let*
       (((count :wat::core::i64)
         (:wat-tests::holon::lru::HologramCacheService::count-recv rx 0)))
       (:wat::core::Result/expect -> :wat::core::unit
         (:wat::kernel::send out count)
         "counter-worker: out disconnected — parent dropped Thread/output?")))

   ;; ─── Step 3 helper — drive Service/loop, send final len on `out` ──
   ;; HologramCache is thread-owned; we cannot return the cache itself
   ;; across the join boundary. Compute len inside the worker; only
   ;; the i64 crosses via the substrate's output pipe. Pass null-
   ;; reporter + null-metrics-cadence — these tests don't care about
   ;; reporting.
   (:wat::core::define
     (:wat-tests::holon::lru::HologramCacheService::loop-then-len-worker
       (req-rxs :wat::core::Vector<wat::holon::lru::HologramCacheService::ReqRx>)
       (cap :wat::core::i64)
       (out :rust::crossbeam_channel::Sender<wat::core::i64>)
       -> :wat::core::unit)
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
           (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
        ((len :wat::core::i64)
         (:wat::holon::lru::HologramCache/len
           (:wat::holon::lru::HologramCacheService::State/cache final))))
       (:wat::core::Result/expect -> :wat::core::unit
         (:wat::kernel::send out len)
         "loop-then-len-worker: out disconnected — parent dropped Thread/output?")))))

;; ─── Step 1 — spawn-thread + Thread/join-result, no caller channels ──

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step1-spawn-join
  (:wat::core::let*
    (((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
      (:wat::kernel::spawn-thread
        :wat-tests::holon::lru::HologramCacheService::trivial-worker))
     ((rx :rust::crossbeam_channel::Receiver<wat::core::i64>)
      (:wat::kernel::Thread/output thr))
     ((_len :wat::core::i64)
      (:wat::core::Option/expect -> :wat::core::i64
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
          (:wat::kernel::recv rx)
          "step1: thread died before sending len")
        "step1: thread output closed without sending len")))
    (:wat::core::match (:wat::kernel::Thread/join-result thr) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "spawn-died" "")))))

;; ─── Step 2 — counted recv via a caller-allocated channel ──────

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step2-counted-recv
  (:wat::core::let*
    ;; Outer scope holds the Thread. Inner scope owns the queue pair
    ;; + every Sender clone; inner returns the Thread; pair drops at
    ;; inner exit. SERVICE-PROGRAMS.md § "The lockstep". Arc 117
    ;; would refuse to compile a sibling-pair-with-spawn-thread shape
    ;; alongside Thread/join-result in the same let*.
    (((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
      (:wat::core::let*
        (((pair :wat::kernel::QueuePair<wat::core::i64>)
          (:wat::kernel::make-bounded-queue :wat::core::i64 1))
         ((tx :wat::kernel::QueueSender<wat::core::i64>) (:wat::core::first pair))
         ((rx :wat::kernel::QueueReceiver<wat::core::i64>) (:wat::core::second pair))
         ((h :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
          (:wat::kernel::spawn-thread
            (:wat::core::lambda
              ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
               (out :rust::crossbeam_channel::Sender<wat::core::i64>)
               -> :wat::core::unit)
              (:wat-tests::holon::lru::HologramCacheService::counter-worker rx out))))
         ((_s1 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx 10)
            "step2 send 10: peer disconnected"))
         ((_s2 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx 20)
            "step2 send 20: peer disconnected"))
         ((_s3 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx 30)
            "step2 send 30: peer disconnected")))
        h))
     ((count-rx :rust::crossbeam_channel::Receiver<wat::core::i64>)
      (:wat::kernel::Thread/output thr))
     ((count :wat::core::i64)
      (:wat::core::Option/expect -> :wat::core::i64
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
          (:wat::kernel::recv count-rx)
          "step2: thread died before sending count")
        "step2: thread output closed without sending count")))
    (:wat::core::match (:wat::kernel::Thread/join-result thr) -> :wat::core::unit
      ((:wat::core::Ok _)
        (:wat::core::if (:wat::core::= count 3) -> :wat::core::unit
          ()
          (:wat::test::assert-eq "wrong-count" "")))
      ((:wat::core::Err _) (:wat::test::assert-eq "worker-died" "")))))

;; ─── Step 3 — Service/loop drives the real Request enum (Put only) ──

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step3-put-only
  (:wat::core::let*
    ;; Outer scope holds the Thread. Inner scope owns the queue pair
    ;; + Sender clones + the rxs vec; inner returns the Thread.
    ;; Note: arc 117's checker doesn't currently trace `rxs` (Vec<rx>)
    ;; back to its pair anchor — false negative. Discipline still
    ;; matters: nest the bindings even when the checker can't enforce.
    (((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
      (:wat::core::let*
        (((pair :wat::kernel::QueuePair<wat::holon::lru::HologramCacheService::Request>)
          (:wat::kernel::make-bounded-queue
            :wat::holon::lru::HologramCacheService::Request 1))
         ((tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::core::first pair))
         ((rx :wat::holon::lru::HologramCacheService::ReqRx)
          (:wat::core::second pair))
         ((rxs :wat::core::Vector<wat::holon::lru::HologramCacheService::ReqRx>)
          (:wat::core::conj
            (:wat::core::Vector :wat::holon::lru::HologramCacheService::ReqRx)
            rx))
         ((h :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
          (:wat::kernel::spawn-thread
            (:wat::core::lambda
              ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
               (out :rust::crossbeam_channel::Sender<wat::core::i64>)
               -> :wat::core::unit)
              (:wat-tests::holon::lru::HologramCacheService::loop-then-len-worker
                rxs 16 out))))
         ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :av))
         ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :beta))
         ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :bv))
         ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :gamma))
         ((v3 :wat::holon::HolonAST) (:wat::holon::leaf :gv))
         ((_p1 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx
              (:wat::holon::lru::HologramCacheService::Request::Put k1 v1))
            "step3 send Put k1: peer disconnected"))
         ((_p2 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx
              (:wat::holon::lru::HologramCacheService::Request::Put k2 v2))
            "step3 send Put k2: peer disconnected"))
         ((_p3 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx
              (:wat::holon::lru::HologramCacheService::Request::Put k3 v3))
            "step3 send Put k3: peer disconnected")))
        h))
     ((len-rx :rust::crossbeam_channel::Receiver<wat::core::i64>)
      (:wat::kernel::Thread/output thr))
     ((len :wat::core::i64)
      (:wat::core::Option/expect -> :wat::core::i64
        (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
          (:wat::kernel::recv len-rx)
          "step3: thread died before sending len")
        "step3: thread output closed without sending len")))
    (:wat::core::match (:wat::kernel::Thread/join-result thr) -> :wat::core::unit
      ((:wat::core::Ok _)
        (:wat::core::if (:wat::core::= len 3) -> :wat::core::unit
          ()
          (:wat::test::assert-eq "wrong-len" "")))
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))

;; ─── Step 4 — Put then Get round-trip via reply-tx ──────────────
;;
;; The Service driver's mini-TCP is at the per-REQUEST level (reply-
;; tx embedded in payload). Substrate-allocated `_in`/`_out` for
;; spawn-thread are unused — the service has its own request +
;; reply channels. spawn-thread provides a clean exit-signal via
;; Thread/join-result; that's enough for steps 4-6.

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step4-put-get-roundtrip
  (:wat::core::let*
    ;; Outer holds the Thread. Inner owns the request queue + Sender
    ;; clones + reply channel; inner returns the Thread; pair drops at
    ;; inner exit. SERVICE-PROGRAMS.md § "The lockstep". Arc 117's
    ;; check ensures the structural shape — sibling Sender bindings
    ;; alongside Thread/join-result is a compile error.
    (((thr :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((req-pair :wat::kernel::QueuePair<wat::holon::lru::HologramCacheService::Request>)
          (:wat::kernel::make-bounded-queue
            :wat::holon::lru::HologramCacheService::Request 1))
         ((req-tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::core::first req-pair))
         ((req-rx :wat::holon::lru::HologramCacheService::ReqRx)
          (:wat::core::second req-pair))
         ((rxs :wat::core::Vector<wat::holon::lru::HologramCacheService::ReqRx>)
          (:wat::core::conj
            (:wat::core::Vector :wat::holon::lru::HologramCacheService::ReqRx)
            req-rx))
         ((h :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::kernel::spawn-thread
            (:wat::core::lambda
              ((_in :rust::crossbeam_channel::Receiver<wat::core::unit>)
               (_out :rust::crossbeam_channel::Sender<wat::core::unit>)
               -> :wat::core::unit)
              (:wat::holon::lru::HologramCacheService/run rxs 16
                :wat::holon::lru::HologramCacheService/null-reporter
                (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))))
         ((reply-pair :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-queue :wat::core::Option<wat::holon::HolonAST> 1))
         ((reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair))
         ((reply-rx :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair))
         ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v :wat::holon::HolonAST) (:wat::holon::leaf :av))
         ((_p :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send req-tx
              (:wat::holon::lru::HologramCacheService::Request::Put k v))
            "step4 send Put: peer disconnected"))
         ((_g :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send req-tx
              (:wat::holon::lru::HologramCacheService::Request::Get k reply-tx))
            "step4 send Get: peer disconnected"))
         ((_check :wat::core::unit)
          (:wat::core::match (:wat::kernel::recv reply-rx) -> :wat::core::unit
            ((:wat::core::Ok (:wat::core::Some inner))
              (:wat::core::match inner -> :wat::core::unit
                ((:wat::core::Some _val) ())
                (:wat::core::None (:wat::test::assert-eq "cache-miss" ""))))
            ((:wat::core::Ok :wat::core::None) (:wat::test::assert-eq "no-reply" ""))
            ((:wat::core::Err _died) (:wat::test::assert-eq "no-reply" "")))))
        h)))
    (:wat::core::match (:wat::kernel::Thread/join-result thr) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))

;; ─── Step 5 — full Service constructor + HandlePool fan-in ──────

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step5-multi-client-via-constructor
  (:wat::core::let*
    ;; Outer holds only the driver Thread. Inner owns the spawn-tuple
    ;; (pool + driver), pops Senders, drives the protocol, drops
    ;; everything but the driver Thread which inner returns. Pool
    ;; holds N Sender clones; arc 117 catches sibling-pool-with-driver
    ;; alongside Thread/join-result.
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((spawn :wat::holon::lru::HologramCacheService::Spawn)
          (:wat::holon::lru::HologramCacheService/spawn 2 16
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
         ((pool :wat::holon::lru::HologramCacheService::ReqTxPool)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::core::second spawn))
         ((tx-a :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((tx-b :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))

         ((reply-pair-a :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-queue :wat::core::Option<wat::holon::HolonAST> 1))
         ((reply-tx-a :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair-a))
         ((reply-rx-a :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair-a))

         ((reply-pair-b :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-queue :wat::core::Option<wat::holon::HolonAST> 1))
         ((reply-tx-b :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair-b))
         ((reply-rx-b :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair-b))

         ((k-a :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v-a :wat::holon::HolonAST) (:wat::holon::leaf :av))
         ((k-b :wat::holon::HolonAST) (:wat::holon::leaf :beta))
         ((v-b :wat::holon::HolonAST) (:wat::holon::leaf :bv))

         ;; Client A: Put + Get on alpha
         ((_pa :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx-a
              (:wat::holon::lru::HologramCacheService::Request::Put k-a v-a))
            "step5 client-a send Put: peer disconnected"))
         ((_ga :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx-a
              (:wat::holon::lru::HologramCacheService::Request::Get k-a reply-tx-a))
            "step5 client-a send Get: peer disconnected"))
         ((_check-a :wat::core::unit)
          (:wat::core::match (:wat::kernel::recv reply-rx-a) -> :wat::core::unit
            ((:wat::core::Ok (:wat::core::Some inner))
              (:wat::core::match inner -> :wat::core::unit
                ((:wat::core::Some _val) ())
                (:wat::core::None (:wat::test::assert-eq "client-a-miss" ""))))
            ((:wat::core::Ok :wat::core::None) (:wat::test::assert-eq "client-a-no-reply" ""))
            ((:wat::core::Err _died) (:wat::test::assert-eq "client-a-no-reply" ""))))

         ;; Client B: Put + Get on beta
         ((_pb :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx-b
              (:wat::holon::lru::HologramCacheService::Request::Put k-b v-b))
            "step5 client-b send Put: peer disconnected"))
         ((_gb :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx-b
              (:wat::holon::lru::HologramCacheService::Request::Get k-b reply-tx-b))
            "step5 client-b send Get: peer disconnected"))
         ((_check-b :wat::core::unit)
          (:wat::core::match (:wat::kernel::recv reply-rx-b) -> :wat::core::unit
            ((:wat::core::Ok (:wat::core::Some inner))
              (:wat::core::match inner -> :wat::core::unit
                ((:wat::core::Some _val) ())
                (:wat::core::None (:wat::test::assert-eq "client-b-miss" ""))))
            ((:wat::core::Ok :wat::core::None) (:wat::test::assert-eq "client-b-no-reply" ""))
            ((:wat::core::Err _died) (:wat::test::assert-eq "client-b-no-reply" "")))))
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))

;; ─── Step 6 — LRU eviction visible through Service Get/Put round-trips ──
;;
;; cap=2 cache; Put k1, Put k2, Put k3 — k1 should be evicted.
;; Subsequent Get(k1) returns None; Get(k2) returns Some. This proves
;; the queue-addressed wrapper preserves the HologramCache eviction
;; semantics — eviction visible from the client's view, not just at
;; the substrate.

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step6-lru-eviction-via-service
  (:wat::core::let*
    ;; Outer holds only the driver Thread. Inner owns spawn-tuple,
    ;; pops Sender, drives the protocol, drops everything but driver.
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((spawn :wat::holon::lru::HologramCacheService::Spawn)
          (:wat::holon::lru::HologramCacheService/spawn 1 2
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
         ((pool :wat::holon::lru::HologramCacheService::ReqTxPool)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::core::second spawn))
         ((tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))

         ((reply-pair :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-queue :wat::core::Option<wat::holon::HolonAST> 1))
         ((reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair))
         ((reply-rx :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair))

         ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
         ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
         ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :third))
         ((v :wat::holon::HolonAST) (:wat::holon::leaf :payload))

         ;; Three puts at cap=2; k1 gets evicted by k3.
         ((_p1 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx
              (:wat::holon::lru::HologramCacheService::Request::Put k1 v))
            "step6 send Put k1: peer disconnected"))
         ((_p2 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx
              (:wat::holon::lru::HologramCacheService::Request::Put k2 v))
            "step6 send Put k2: peer disconnected"))
         ((_p3 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx
              (:wat::holon::lru::HologramCacheService::Request::Put k3 v))
            "step6 send Put k3: peer disconnected"))

         ;; Get k1 — evicted, expect None.
         ((_g1 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx
              (:wat::holon::lru::HologramCacheService::Request::Get k1 reply-tx))
            "step6 send Get k1: peer disconnected"))
         ((_check-1 :wat::core::unit)
          (:wat::core::match (:wat::kernel::recv reply-rx) -> :wat::core::unit
            ((:wat::core::Ok (:wat::core::Some inner))
              (:wat::core::match inner -> :wat::core::unit
                ((:wat::core::Some _) (:wat::test::assert-eq "k1-not-evicted" ""))
                (:wat::core::None ())))
            ((:wat::core::Ok :wat::core::None) (:wat::test::assert-eq "no-reply-1" ""))
            ((:wat::core::Err _died) (:wat::test::assert-eq "no-reply-1" ""))))

         ;; Get k2 — survived, expect Some.
         ((_g2 :wat::core::unit)
          (:wat::core::Result/expect -> :wat::core::unit
            (:wat::kernel::send tx
              (:wat::holon::lru::HologramCacheService::Request::Get k2 reply-tx))
            "step6 send Get k2: peer disconnected"))
         ((_check-2 :wat::core::unit)
          (:wat::core::match (:wat::kernel::recv reply-rx) -> :wat::core::unit
            ((:wat::core::Ok (:wat::core::Some inner))
              (:wat::core::match inner -> :wat::core::unit
                ((:wat::core::Some _) ())
                (:wat::core::None (:wat::test::assert-eq "k2-evicted" ""))))
            ((:wat::core::Ok :wat::core::None) (:wat::test::assert-eq "no-reply-2" ""))
            ((:wat::core::Err _died) (:wat::test::assert-eq "no-reply-2" "")))))
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))
