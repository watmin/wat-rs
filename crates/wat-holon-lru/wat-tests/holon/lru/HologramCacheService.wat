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
;;   3. Service/spawn constructor + helper verbs (Put N items);
;;      verify all N are in cache via Get (consumer vantage)
;;   4. Put + Get round-trip via HologramCacheService/put + /get
;;   5. Service/spawn constructor + HandlePool fan-in (multi-client),
;;      each client uses helper verbs
;;   6. LRU eviction visible through HologramCacheService/put + /get
;;
;; Arc 119: steps 3-6 use the consumer-surface helper verbs
;; (HologramCacheService/get, HologramCacheService/put) rather than
;; raw Request enum construction + kernel::send. Consumer-crate
;; wat-tests stand at the consumer's vantage per CONVENTIONS.md
;; § "Caller-perspective verification".
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
   ;; make-bounded-channel), counting each Some. When the caller's
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
       (rx :wat::kernel::Receiver<wat::core::i64>)
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
       (rx :wat::kernel::Receiver<wat::core::i64>)
       (out :rust::crossbeam_channel::Sender<wat::core::i64>)
       -> :wat::core::unit)
     (:wat::core::let*
       (((count :wat::core::i64)
         (:wat-tests::holon::lru::HologramCacheService::count-recv rx 0)))
       (:wat::core::Result/expect -> :wat::core::unit
         (:wat::kernel::send out count)
         "counter-worker: out disconnected — parent dropped Thread/output?")))))

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
        (((pair :wat::kernel::Channel<wat::core::i64>)
          (:wat::kernel::make-bounded-channel :wat::core::i64 1))
         ((tx :wat::kernel::Sender<wat::core::i64>) (:wat::core::first pair))
         ((rx :wat::kernel::Receiver<wat::core::i64>) (:wat::core::second pair))
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

;; ─── Step 3 — HologramCacheService/spawn + helper verbs (Put N items) ──
;;
;; Arc 119 discipline correction: consumer vantage. The scenario is
;; "put N items into the cache and observe they're present." Previously
;; this drove Service/loop directly and checked final len — implementer
;; vantage. At the consumer's vantage: spawn, put 3 items via the
;; /put helper verb, verify all 3 are present via /get. Scenarios are
;; preserved; wire-protocol mechanics are not the test's concern.

;; Arc 126 — the arc 126 check at inner freeze sees the Put-ack
;; helper-verb cycle (Pattern B: ack-tx held in inner scope while
;; req-tx is also held) and panics with the substring
;; `channel-pair-deadlock`. This test is EXPECTED to panic with
;; that substring; cargo libtest matches by substring and reports
;; the test as PASSING. `:time-limit "200ms"` stays as a defense-
;; in-depth safety net in case the expected panic does not fire.
(:wat::test::should-panic "channel-pair-deadlock")
(:wat::test::time-limit "200ms")
(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step3-put-only
  (:wat::core::let*
    ;; Outer holds only the driver Thread. Inner owns the spawn-tuple
    ;; (pool + driver), pops the req-tx, allocates per-call channels,
    ;; drives the protocol via helper verbs, drops everything but the
    ;; driver Thread which inner returns.
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((spawn :wat::holon::lru::HologramCacheService::Spawn)
          (:wat::holon::lru::HologramCacheService/spawn 1 16
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
         ((pool :wat::holon::lru::HologramCacheService::ReqTxPool)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::core::second spawn))
         ((req-tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))

         ;; Allocate ack channel once; reused across all puts.
         ((ack-pair :wat::holon::lru::HologramCacheService::PutAckChannel)
          (:wat::kernel::make-bounded-channel :wat::core::unit 1))
         ((ack-tx :wat::holon::lru::HologramCacheService::PutAckTx)
          (:wat::core::first ack-pair))
         ((ack-rx :wat::holon::lru::HologramCacheService::PutAckRx)
          (:wat::core::second ack-pair))

         ;; Allocate reply channel once; reused across all gets.
         ((reply-pair :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-channel
            :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>> 1))
         ((reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair))
         ((reply-rx :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair))

         ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v1 :wat::holon::HolonAST) (:wat::holon::leaf :av))
         ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :beta))
         ((v2 :wat::holon::HolonAST) (:wat::holon::leaf :bv))
         ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :gamma))
         ((v3 :wat::holon::HolonAST) (:wat::holon::leaf :gv))

         ;; Put 3 items — each is a batch-of-one.
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k1 v1))))
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k2 v2))))
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k3 v3))))

         ;; Verify: all 3 keys are present (cap=16, no eviction).
         ;; first on Vector<Option<T>> returns Option<Option<T>> — double-unwrap.
         ((r1 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
          (:wat::holon::lru::HologramCacheService/get req-tx reply-tx reply-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::HolonAST) k1)))
         ;; Two-level match: outer on first's Option wrapper; inner on cache hit/miss.
         ((_ :wat::core::unit)
          (:wat::core::match (:wat::core::first r1) -> :wat::core::unit
            ((:wat::core::Some inner1)
              (:wat::core::match inner1 -> :wat::core::unit
                ((:wat::core::Some _) ())
                (:wat::core::None     (:wat::test::assert-eq "k1-missing" ""))))
            (:wat::core::None (:wat::test::assert-eq "k1-missing" ""))))

         ((r2 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
          (:wat::holon::lru::HologramCacheService/get req-tx reply-tx reply-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::HolonAST) k2)))
         ((_ :wat::core::unit)
          (:wat::core::match (:wat::core::first r2) -> :wat::core::unit
            ((:wat::core::Some inner2)
              (:wat::core::match inner2 -> :wat::core::unit
                ((:wat::core::Some _) ())
                (:wat::core::None     (:wat::test::assert-eq "k2-missing" ""))))
            (:wat::core::None (:wat::test::assert-eq "k2-missing" ""))))

         ((r3 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
          (:wat::holon::lru::HologramCacheService/get req-tx reply-tx reply-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::HolonAST) k3)))
         ((_ :wat::core::unit)
          (:wat::core::match (:wat::core::first r3) -> :wat::core::unit
            ((:wat::core::Some inner3)
              (:wat::core::match inner3 -> :wat::core::unit
                ((:wat::core::Some _) ())
                (:wat::core::None     (:wat::test::assert-eq "k3-missing" ""))))
            (:wat::core::None (:wat::test::assert-eq "k3-missing" "")))))
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))

;; ─── Step 4 — Put then Get round-trip via helper verbs ──────────
;;
;; Arc 119 discipline correction: consumer vantage. Previously this
;; hand-built Request::Put and Request::Get and called kernel::send
;; directly. At the consumer's vantage: use HologramCacheService/put
;; and HologramCacheService/get. Same scenario: put one item, get it
;; back, assert the value is present.

;; Arc 126 — the Put-ack helper-verb cycle (req-tx + ack-tx both held
;; in the inner scope) trips arc 126's check at inner freeze; the
;; expected panic substring is `channel-pair-deadlock`. The 200ms
;; time-limit stays as defense-in-depth.
(:wat::test::should-panic "channel-pair-deadlock")
(:wat::test::time-limit "200ms")
(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step4-put-get-roundtrip
  (:wat::core::let*
    ;; Outer holds the driver Thread. Inner owns spawn-tuple, pops
    ;; req-tx, allocates channels, drives via helper verbs, returns Thread.
    (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::let*
        (((spawn :wat::holon::lru::HologramCacheService::Spawn)
          (:wat::holon::lru::HologramCacheService/spawn 1 16
            :wat::holon::lru::HologramCacheService/null-reporter
            (:wat::holon::lru::HologramCacheService/null-metrics-cadence)))
         ((pool :wat::holon::lru::HologramCacheService::ReqTxPool)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
          (:wat::core::second spawn))
         ((req-tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))

         ((ack-pair :wat::holon::lru::HologramCacheService::PutAckChannel)
          (:wat::kernel::make-bounded-channel :wat::core::unit 1))
         ((ack-tx :wat::holon::lru::HologramCacheService::PutAckTx)
          (:wat::core::first ack-pair))
         ((ack-rx :wat::holon::lru::HologramCacheService::PutAckRx)
          (:wat::core::second ack-pair))

         ((reply-pair :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-channel
            :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>> 1))
         ((reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair))
         ((reply-rx :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair))

         ((k :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v :wat::holon::HolonAST) (:wat::holon::leaf :av))

         ;; Put one entry, then get it back.
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k v))))

         ((results :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
          (:wat::holon::lru::HologramCacheService/get req-tx reply-tx reply-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::HolonAST) k)))

         ;; Two-level match: outer on first's Option wrapper; inner on cache hit/miss.
         ((_ :wat::core::unit)
          (:wat::core::match (:wat::core::first results) -> :wat::core::unit
            ((:wat::core::Some inner)
              (:wat::core::match inner -> :wat::core::unit
                ((:wat::core::Some _val) ())
                (:wat::core::None        (:wat::test::assert-eq "cache-miss" ""))))
            (:wat::core::None (:wat::test::assert-eq "cache-miss" "")))))
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))

;; ─── Step 5 — full Service constructor + HandlePool fan-in ──────
;;
;; Arc 119 discipline correction: consumer vantage. Previously this
;; hand-built Request::Put/Get per client. At the consumer's vantage:
;; each client pops its own req-tx from the HandlePool and uses the
;; helper verbs. Two clients, each does put-then-get on their own key;
;; each client sees its own data.

;; Arc 126 — multi-client also holds req-tx + ack-tx pairs in inner
;; scope; the arc 126 check fires at inner freeze. Expected panic
;; substring `channel-pair-deadlock`. 200ms safety net preserved.
(:wat::test::should-panic "channel-pair-deadlock")
(:wat::test::time-limit "200ms")
(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step5-multi-client-via-constructor
  (:wat::core::let*
    ;; Outer holds only the driver Thread. Inner owns the spawn-tuple
    ;; (pool + driver), pops Senders, drives the protocol via helper
    ;; verbs, drops everything but the driver Thread which inner returns.
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

         ;; Client A channels.
         ((ack-pair-a :wat::holon::lru::HologramCacheService::PutAckChannel)
          (:wat::kernel::make-bounded-channel :wat::core::unit 1))
         ((ack-tx-a :wat::holon::lru::HologramCacheService::PutAckTx)
          (:wat::core::first ack-pair-a))
         ((ack-rx-a :wat::holon::lru::HologramCacheService::PutAckRx)
          (:wat::core::second ack-pair-a))
         ((reply-pair-a :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-channel
            :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>> 1))
         ((reply-tx-a :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair-a))
         ((reply-rx-a :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair-a))

         ;; Client B channels.
         ((ack-pair-b :wat::holon::lru::HologramCacheService::PutAckChannel)
          (:wat::kernel::make-bounded-channel :wat::core::unit 1))
         ((ack-tx-b :wat::holon::lru::HologramCacheService::PutAckTx)
          (:wat::core::first ack-pair-b))
         ((ack-rx-b :wat::holon::lru::HologramCacheService::PutAckRx)
          (:wat::core::second ack-pair-b))
         ((reply-pair-b :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-channel
            :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>> 1))
         ((reply-tx-b :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair-b))
         ((reply-rx-b :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair-b))

         ((k-a :wat::holon::HolonAST) (:wat::holon::leaf :alpha))
         ((v-a :wat::holon::HolonAST) (:wat::holon::leaf :av))
         ((k-b :wat::holon::HolonAST) (:wat::holon::leaf :beta))
         ((v-b :wat::holon::HolonAST) (:wat::holon::leaf :bv))

         ;; Client A: Put + Get on alpha.
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put tx-a ack-tx-a ack-rx-a
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k-a v-a))))
         ((results-a :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
          (:wat::holon::lru::HologramCacheService/get tx-a reply-tx-a reply-rx-a
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::HolonAST) k-a)))
         ;; Two-level match: outer on first's Option wrapper; inner on cache hit/miss.
         ((_ :wat::core::unit)
          (:wat::core::match (:wat::core::first results-a) -> :wat::core::unit
            ((:wat::core::Some inner-a)
              (:wat::core::match inner-a -> :wat::core::unit
                ((:wat::core::Some _val) ())
                (:wat::core::None        (:wat::test::assert-eq "client-a-miss" ""))))
            (:wat::core::None (:wat::test::assert-eq "client-a-miss" ""))))

         ;; Client B: Put + Get on beta.
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put tx-b ack-tx-b ack-rx-b
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k-b v-b))))
         ((results-b :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
          (:wat::holon::lru::HologramCacheService/get tx-b reply-tx-b reply-rx-b
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::HolonAST) k-b)))
         ;; Two-level match: outer on first's Option wrapper; inner on cache hit/miss.
         ((_ :wat::core::unit)
          (:wat::core::match (:wat::core::first results-b) -> :wat::core::unit
            ((:wat::core::Some inner-b)
              (:wat::core::match inner-b -> :wat::core::unit
                ((:wat::core::Some _val) ())
                (:wat::core::None        (:wat::test::assert-eq "client-b-miss" ""))))
            (:wat::core::None (:wat::test::assert-eq "client-b-miss" "")))))
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))

;; ─── Step 6 — LRU eviction visible through Service Get/Put round-trips ──
;;
;; Arc 119 discipline correction: consumer vantage. Previously this
;; hand-built Request::Put/Get with raw send/recv. At the consumer's
;; vantage: use helper verbs throughout. Same scenario: cap=2 cache,
;; put k1/k2/k3 — k1 evicts. Subsequent get(k1) returns None; get(k2)
;; returns Some. Proves the queue-addressed wrapper preserves eviction
;; semantics from the client's view.

;; Arc 126 — same Pattern B cycle as steps 3-5; arc 126's check
;; fires at inner freeze and panics with `channel-pair-deadlock`.
;; Test EXPECTED to panic with that substring; 200ms safety net
;; preserved as defense-in-depth.
(:wat::test::should-panic "channel-pair-deadlock")
(:wat::test::time-limit "200ms")
(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step6-lru-eviction-via-service
  (:wat::core::let*
    ;; Outer holds only the driver Thread. Inner owns spawn-tuple,
    ;; pops Sender, drives the protocol via helper verbs, returns Thread.
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
         ((req-tx :wat::holon::lru::HologramCacheService::ReqTx)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))

         ((ack-pair :wat::holon::lru::HologramCacheService::PutAckChannel)
          (:wat::kernel::make-bounded-channel :wat::core::unit 1))
         ((ack-tx :wat::holon::lru::HologramCacheService::PutAckTx)
          (:wat::core::first ack-pair))
         ((ack-rx :wat::holon::lru::HologramCacheService::PutAckRx)
          (:wat::core::second ack-pair))

         ((reply-pair :wat::holon::lru::HologramCacheService::GetReplyPair)
          (:wat::kernel::make-bounded-channel
            :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>> 1))
         ((reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx)
          (:wat::core::first reply-pair))
         ((reply-rx :wat::holon::lru::HologramCacheService::GetReplyRx)
          (:wat::core::second reply-pair))

         ((k1 :wat::holon::HolonAST) (:wat::holon::leaf :first))
         ((k2 :wat::holon::HolonAST) (:wat::holon::leaf :second))
         ((k3 :wat::holon::HolonAST) (:wat::holon::leaf :third))
         ((v :wat::holon::HolonAST) (:wat::holon::leaf :payload))

         ;; Three puts at cap=2; k1 gets evicted by k3.
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k1 v))))
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k2 v))))
         ((_ :wat::core::unit)
          (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
              (:wat::core::Tuple k3 v))))

         ;; Get k1 — evicted, expect None.
         ;; Two-level match: outer on first's Option wrapper; inner on cache hit/miss.
         ((r1 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
          (:wat::holon::lru::HologramCacheService/get req-tx reply-tx reply-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::HolonAST) k1)))
         ((_ :wat::core::unit)
          (:wat::core::match (:wat::core::first r1) -> :wat::core::unit
            ((:wat::core::Some inner1)
              (:wat::core::match inner1 -> :wat::core::unit
                ((:wat::core::Some _) (:wat::test::assert-eq "k1-not-evicted" ""))
                (:wat::core::None     ())))   ;; evicted — expected
            (:wat::core::None ())))

         ;; Get k2 — survived, expect Some.
         ((r2 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
          (:wat::holon::lru::HologramCacheService/get req-tx reply-tx reply-rx
            (:wat::core::conj
              (:wat::core::Vector :wat::holon::HolonAST) k2)))
         ((_ :wat::core::unit)
          (:wat::core::match (:wat::core::first r2) -> :wat::core::unit
            ((:wat::core::Some inner2)
              (:wat::core::match inner2 -> :wat::core::unit
                ((:wat::core::Some _) ())
                (:wat::core::None     (:wat::test::assert-eq "k2-evicted" ""))))
            (:wat::core::None (:wat::test::assert-eq "k2-evicted" "")))))
        d)))
    (:wat::core::match (:wat::kernel::Thread/join-result driver) -> :wat::core::unit
      ((:wat::core::Ok _) ())
      ((:wat::core::Err _) (:wat::test::assert-eq "service-died" "")))))
