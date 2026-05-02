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
;;
;; Arc 130 — complectēns rewrite. Top-down dependency graphs in TWO
;; named preludes:
;;
;; ─── Prelude :deftest-hermetic (no arc-126 trigger) ──────────────
;;
;;   Existing workers:
;;     trivial-worker, count-recv, counter-worker
;;
;;   Layer 0  :test::hcs-trivial-spawn-recv-join    ; step1 full scenario
;;            :test::hcs-spawn-and-shutdown          ; spawn HCS, finish, join
;;            :test::hcs-spawn-send-3-count          ; spawn counter-worker, send 3 values
;;
;;   Layer 1  :test::hcs-recv-count-and-join        ; recv count from Thread/output, join
;;
;; ─── Prelude :deftest-service (triggers arc-126) ─────────────────
;;
;;   Layer 1  :test::hcs-assert-hit                 ; assert Some(Some(_)) on results vec
;;            :test::hcs-assert-miss                ; assert eviction (None or Some(None))
;;            :test::hcs-put-one-entry              ; single-entry put via helper verb
;;            :test::hcs-get-one-key               ; single-key get via helper verb
;;
;;   Layer 2  :test::hcs-spawn-put-get              ; spawn HCS, put k/v, get k, tear down
;;            :test::hcs-spawn-put-3-verify         ; spawn HCS cap=16, put 3, verify all hit
;;            :test::hcs-spawn-2clients-put-get-verify ; 2-client put+get+assert
;;            :test::hcs-spawn-put-3-eviction       ; cap=2, put 3, assert k1 evicted + k2 hit
;;
;; All :deftest-service deftests :should-panic("channel-pair-deadlock"):
;; prelude 2 contains helpers with make-bounded-channel + helper-verb
;; call sites in the same let* scope; arc 126 fires at freeze.
;; :deftest-hermetic steps 1-2 and their per-helper proofs pass cleanly.

;; ─────────────────────────────────────────────────────────────────────────
;; Prelude 1 — :deftest-hermetic. No arc-126 triggers.
;; Steps 1-2 and per-helper proofs for the pure helpers.
;; ─────────────────────────────────────────────────────────────────────────

(:wat::test::make-deftest :deftest-hermetic
  (
   ;; ─── Step 1 worker ──────────────────────────────────────────────
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

   ;; ─── Step 2 counted-recv helpers ────────────────────────────────
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
         "counter-worker: out disconnected — parent dropped Thread/output?")))

   ;; ─── Layer 0 — step1 full scenario ──────────────────────────────
   ;;
   ;; :test::hcs-trivial-spawn-recv-join — spawn trivial-worker, recv the
   ;; len it sends on Thread/output (double-unwrap), join the Thread.
   ;; The "I ran without dying" proof: if the worker dies before sending,
   ;; the double-unwrap panics with a named message.
   (:wat::core::define
     (:test::hcs-trivial-spawn-recv-join -> :wat::core::unit)
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
             "hcs-trivial-spawn-recv-join: thread died before sending len")
           "hcs-trivial-spawn-recv-join: thread output closed without sending len")))
       (:wat::core::match (:wat::kernel::Thread/join-result thr) -> :wat::core::unit
         ((:wat::core::Ok _) ())
         ((:wat::core::Err _) (:wat::test::assert-eq "spawn-died" "")))))

   ;; ─── Layer 0 — HCS lifecycle ─────────────────────────────────────
   ;;
   ;; :test::hcs-spawn-and-shutdown — spawn HologramCacheService (1 client,
   ;; cap=16), pop the req-tx so the pool has no orphaned handles, finish
   ;; the pool, drop the req-tx at inner scope exit, join the driver.
   ;; No helper-verb calls; the narrowest possible HCS lifecycle proof.
   (:wat::core::define
     (:test::hcs-spawn-and-shutdown -> :wat::core::unit)
     (:wat::core::let*
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
            ((_req-tx :wat::holon::lru::HologramCacheService::ReqTx)
             (:wat::kernel::HandlePool::pop pool))
            ((_finish :wat::core::unit)
             (:wat::kernel::HandlePool::finish pool)))
           d))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       ()))

   ;; ─── Layer 0 — counter-worker scenario ───────────────────────────
   ;;
   ;; :test::hcs-spawn-send-3-count — make a caller-allocated channel pair,
   ;; spawn counter-worker (closes over rx via lambda), send 3 values in
   ;; the inner scope (so the tx drops at inner exit → worker sees EOF),
   ;; return the Thread for the caller to recv count from + join.
   (:wat::core::define
     (:test::hcs-spawn-send-3-count
       (a :wat::core::i64)
       (b :wat::core::i64)
       (c :wat::core::i64)
       -> :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
     (:wat::core::let*
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
               (:wat::kernel::send tx a)
               "hcs-spawn-send-3-count: send a: peer disconnected"))
            ((_s2 :wat::core::unit)
             (:wat::core::Result/expect -> :wat::core::unit
               (:wat::kernel::send tx b)
               "hcs-spawn-send-3-count: send b: peer disconnected"))
            ((_s3 :wat::core::unit)
             (:wat::core::Result/expect -> :wat::core::unit
               (:wat::kernel::send tx c)
               "hcs-spawn-send-3-count: send c: peer disconnected")))
           h)))
       thr))

   ;; ─── Layer 1 — count receipt and join ────────────────────────────
   ;;
   ;; :test::hcs-recv-count-and-join — recv the count from a Thread's
   ;; output channel (double-unwrap), join the Thread, assert the count
   ;; equals the expected value. Composes with :test::hcs-spawn-send-3-count.
   (:wat::core::define
     (:test::hcs-recv-count-and-join
       (thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
       (expected :wat::core::i64)
       -> :wat::core::unit)
     (:wat::core::let*
       (((count-rx :rust::crossbeam_channel::Receiver<wat::core::i64>)
         (:wat::kernel::Thread/output thr))
        ((count :wat::core::i64)
         (:wat::core::Option/expect -> :wat::core::i64
           (:wat::core::Result/expect -> :wat::core::Option<wat::core::i64>
             (:wat::kernel::recv count-rx)
             "hcs-recv-count-and-join: thread died before sending count")
           "hcs-recv-count-and-join: thread output closed without sending count")))
       (:wat::core::match (:wat::kernel::Thread/join-result thr) -> :wat::core::unit
         ((:wat::core::Ok _)
           (:wat::core::if (:wat::core::= count expected) -> :wat::core::unit
             ()
             (:wat::test::assert-eq "wrong-count" "")))
         ((:wat::core::Err _) (:wat::test::assert-eq "worker-died" "")))))
   ))

;; ─── Per-layer deftests (prelude 1) ──────────────────────────────────────
;; All use :deftest-hermetic. No arc-126 trigger in prelude 1 → no
;; :should-panic needed. These are the bottom-up proofs before top-down
;; composition.

;; Layer 0 — step1 scenario proof.
(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-hcs-trivial-spawn-recv-join
  (:test::hcs-trivial-spawn-recv-join))


;; Layer 0 — HCS lifecycle proof.
(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-hcs-spawn-and-shutdown
  (:test::hcs-spawn-and-shutdown))


;; Layer 0 — counter-worker spawn+send proof.
;; Directly exercises :test::hcs-spawn-send-3-count by sending 3 values
;; and verifying the Thread is returned (caller can recv + join).
(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-hcs-spawn-send-3-count
  (:wat::core::let*
    (((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
      (:test::hcs-spawn-send-3-count 10 20 30)))
    (:test::hcs-recv-count-and-join thr 3)))


;; Layer 1 — count-recv-join proof.
;; Composed from Layer 0: hcs-spawn-send-3-count provides the Thread;
;; hcs-recv-count-and-join is the unit under test.
(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-hcs-recv-count-and-join
  (:wat::core::let*
    (((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
      (:test::hcs-spawn-send-3-count 100 200 300)))
    (:test::hcs-recv-count-and-join thr 3)))


;; ─────────────────────────────────────────────────────────────────────────
;; Steps 1-2 — uses :deftest-hermetic. No arc-126 trigger → no :should-panic.
;; ─────────────────────────────────────────────────────────────────────────

;; ─── Step 1 — spawn-thread + Thread/join-result, no caller channels ──────

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step1-spawn-join
  (:test::hcs-trivial-spawn-recv-join))

;; ─── Step 2 — counted recv via a caller-allocated channel ────────────────

(:deftest-hermetic :wat-tests::holon::lru::HologramCacheService::test-step2-counted-recv
  (:wat::core::let*
    (((thr :wat::kernel::Thread<wat::core::unit,wat::core::i64>)
      (:test::hcs-spawn-send-3-count 10 20 30)))
    (:test::hcs-recv-count-and-join thr 3)))

;; ─────────────────────────────────────────────────────────────────────────
;; Prelude 2 — :deftest-service. Triggers arc-126 via scenario helpers
;; that contain make-bounded-channel + tx/rx + helper-verb calls in the
;; same let* scope. Steps 3-6 and their per-helper proofs all
;; :should-panic("channel-pair-deadlock").
;; ─────────────────────────────────────────────────────────────────────────

(:wat::test::make-deftest :deftest-service
  (
   ;; ─── Layer 1 — assertion helpers ─────────────────────────────────
   ;;
   ;; :test::hcs-assert-hit — assert that the first element of a results
   ;; vector is Some(Some(_)), i.e. a cache hit. Fails with label on miss.
   (:wat::core::define
     (:test::hcs-assert-hit
       (results :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
       (label :wat::core::String)
       -> :wat::core::unit)
     (:wat::core::match (:wat::core::first results) -> :wat::core::unit
       ((:wat::core::Some inner)
         (:wat::core::match inner -> :wat::core::unit
           ((:wat::core::Some _) ())
           (:wat::core::None     (:wat::test::assert-eq label ""))))
       (:wat::core::None (:wat::test::assert-eq label ""))))

   ;; :test::hcs-assert-miss — assert that the first element of a results
   ;; vector is None or Some(None), i.e. a cache miss (eviction expected).
   ;; Fails with label if the key is a hit.
   (:wat::core::define
     (:test::hcs-assert-miss
       (results :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
       (label :wat::core::String)
       -> :wat::core::unit)
     (:wat::core::match (:wat::core::first results) -> :wat::core::unit
       ((:wat::core::Some inner)
         (:wat::core::match inner -> :wat::core::unit
           ((:wat::core::Some _) (:wat::test::assert-eq label ""))
           (:wat::core::None     ())))   ;; evicted — expected
       (:wat::core::None ())))           ;; evicted — expected

   ;; ─── Layer 1 — single-verb operation helpers ──────────────────────
   ;;
   ;; :test::hcs-put-one-entry — send one key/value entry via the helper verb.
   ;; Constructs the batch-of-one entry vector and calls /put. Takes all
   ;; channel handles as parameters (no local make-bounded-channel here;
   ;; channels are allocated by the calling scenario helper).
   (:wat::core::define
     (:test::hcs-put-one-entry
       (req-tx :wat::holon::lru::HologramCacheService::ReqTx)
       (ack-tx :wat::holon::lru::HologramCacheService::PutAckTx)
       (ack-rx :wat::holon::lru::HologramCacheService::PutAckRx)
       (k :wat::holon::HolonAST)
       (v :wat::holon::HolonAST)
       -> :wat::core::unit)
     (:wat::holon::lru::HologramCacheService/put req-tx ack-tx ack-rx
       (:wat::core::conj
         (:wat::core::Vector :wat::holon::lru::HologramCacheService::Entry)
         (:wat::core::Tuple k v))))

   ;; :test::hcs-get-one-key — fetch one key via the helper verb.
   ;; Returns the full results vector for the caller to inspect.
   (:wat::core::define
     (:test::hcs-get-one-key
       (req-tx :wat::holon::lru::HologramCacheService::ReqTx)
       (reply-tx :wat::holon::lru::HologramCacheService::GetReplyTx)
       (reply-rx :wat::holon::lru::HologramCacheService::GetReplyRx)
       (k :wat::holon::HolonAST)
       -> :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
     (:wat::holon::lru::HologramCacheService/get req-tx reply-tx reply-rx
       (:wat::core::conj
         (:wat::core::Vector :wat::holon::HolonAST) k)))

   ;; ─── Layer 2 — full-scenario helpers ─────────────────────────────
   ;;
   ;; Each helper internalizes: spawn HCS + pop req-tx + make channel pairs
   ;; + helper-verb calls + driver join. Inner-let* lockstep (arc 131 +
   ;; SERVICE-PROGRAMS.md): outer holds only the driver Thread; inner owns
   ;; spawn-tuple + channels + work; inner returns Thread (or (Thread, results)
   ;; tuple where results must be captured).
   ;;
   ;; The make-bounded-channel allocations + first/second splits +
   ;; helper-verb call sites all land in the same inner let* scope — arc 126
   ;; sees the channel-pair-deadlock pattern and fires at freeze.

   ;; :test::hcs-spawn-put-get — single-client put+get round-trip.
   ;; Spawns HCS (1 client, given cap), puts k/v, gets k, tears down.
   ;; Returns the results vector so the deftest body can assert on it.
   (:wat::core::define
     (:test::hcs-spawn-put-get
       (cap :wat::core::i64)
       (k :wat::holon::HolonAST)
       (v :wat::holon::HolonAST)
       -> :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
     (:wat::core::let*
       (((pair :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>))
         (:wat::core::let*
           (((spawn :wat::holon::lru::HologramCacheService::Spawn)
             (:wat::holon::lru::HologramCacheService/spawn 1 cap
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
            ((_put :wat::core::unit)
             (:test::hcs-put-one-entry req-tx ack-tx ack-rx k v))
            ((results :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
             (:test::hcs-get-one-key req-tx reply-tx reply-rx k)))
           (:wat::core::Tuple d results)))
        ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
         (:wat::core::first pair))
        ((results :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
         (:wat::core::second pair))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       results))

   ;; :test::hcs-spawn-put-3-verify — spawn HCS (1 client, cap=16), put k1/k2/k3,
   ;; get each key and assert all are hits. Internalizes spawn + channel
   ;; allocation + three put+get+assert cycles + driver join. Returns unit.
   (:wat::core::define
     (:test::hcs-spawn-put-3-verify
       (k1 :wat::holon::HolonAST)
       (v1 :wat::holon::HolonAST)
       (k2 :wat::holon::HolonAST)
       (v2 :wat::holon::HolonAST)
       (k3 :wat::holon::HolonAST)
       (v3 :wat::holon::HolonAST)
       -> :wat::core::unit)
     (:wat::core::let*
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
            ((_finish :wat::core::unit)
             (:wat::kernel::HandlePool::finish pool))
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
            ((_ :wat::core::unit)
             (:test::hcs-put-one-entry req-tx ack-tx ack-rx k1 v1))
            ((_ :wat::core::unit)
             (:test::hcs-put-one-entry req-tx ack-tx ack-rx k2 v2))
            ((_ :wat::core::unit)
             (:test::hcs-put-one-entry req-tx ack-tx ack-rx k3 v3))
            ((r1 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
             (:test::hcs-get-one-key req-tx reply-tx reply-rx k1))
            ((_ :wat::core::unit) (:test::hcs-assert-hit r1 "k1-missing"))
            ((r2 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
             (:test::hcs-get-one-key req-tx reply-tx reply-rx k2))
            ((_ :wat::core::unit) (:test::hcs-assert-hit r2 "k2-missing"))
            ((r3 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
             (:test::hcs-get-one-key req-tx reply-tx reply-rx k3))
            ((_ :wat::core::unit) (:test::hcs-assert-hit r3 "k3-missing")))
           d))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       ()))

   ;; :test::hcs-spawn-2clients-put-get-verify — spawn HCS (2 clients, cap=16).
   ;; Each client pops its own req-tx, allocates its own channel pair, puts
   ;; and gets its own key, asserts a hit. Internalizes all channel management
   ;; and driver join. Returns unit.
   (:wat::core::define
     (:test::hcs-spawn-2clients-put-get-verify
       (k-a :wat::holon::HolonAST)
       (v-a :wat::holon::HolonAST)
       (k-b :wat::holon::HolonAST)
       (v-b :wat::holon::HolonAST)
       -> :wat::core::unit)
     (:wat::core::let*
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
            ((_finish :wat::core::unit)
             (:wat::kernel::HandlePool::finish pool))
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
            ;; Client A: put + get + assert hit.
            ((_ :wat::core::unit)
             (:test::hcs-put-one-entry tx-a ack-tx-a ack-rx-a k-a v-a))
            ((results-a :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
             (:test::hcs-get-one-key tx-a reply-tx-a reply-rx-a k-a))
            ((_ :wat::core::unit) (:test::hcs-assert-hit results-a "client-a-miss"))
            ;; Client B: put + get + assert hit.
            ((_ :wat::core::unit)
             (:test::hcs-put-one-entry tx-b ack-tx-b ack-rx-b k-b v-b))
            ((results-b :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
             (:test::hcs-get-one-key tx-b reply-tx-b reply-rx-b k-b))
            ((_ :wat::core::unit) (:test::hcs-assert-hit results-b "client-b-miss")))
           d))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       ()))

   ;; :test::hcs-spawn-put-3-eviction — spawn HCS (1 client, cap=2), put k1/k2/k3
   ;; (k1 evicted by k3), assert k1 is a miss and k2 is a hit. Returns unit.
   (:wat::core::define
     (:test::hcs-spawn-put-3-eviction
       (k1 :wat::holon::HolonAST)
       (k2 :wat::holon::HolonAST)
       (k3 :wat::holon::HolonAST)
       (v :wat::holon::HolonAST)
       -> :wat::core::unit)
     (:wat::core::let*
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
            ((_finish :wat::core::unit)
             (:wat::kernel::HandlePool::finish pool))
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
            ;; Three puts at cap=2; k1 gets evicted by k3.
            ((_ :wat::core::unit)
             (:test::hcs-put-one-entry req-tx ack-tx ack-rx k1 v))
            ((_ :wat::core::unit)
             (:test::hcs-put-one-entry req-tx ack-tx ack-rx k2 v))
            ((_ :wat::core::unit)
             (:test::hcs-put-one-entry req-tx ack-tx ack-rx k3 v))
            ;; k1 evicted — expect miss.
            ((r1 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
             (:test::hcs-get-one-key req-tx reply-tx reply-rx k1))
            ((_ :wat::core::unit) (:test::hcs-assert-miss r1 "k1-not-evicted"))
            ;; k2 survived — expect hit.
            ((r2 :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
             (:test::hcs-get-one-key req-tx reply-tx reply-rx k2))
            ((_ :wat::core::unit) (:test::hcs-assert-hit r2 "k2-evicted")))
           d))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       ()))))

;; ─── Per-layer deftests (prelude 2) ──────────────────────────────────────
;; All use :deftest-service. Prelude 2 contains scenario helpers with
;; make-bounded-channel + helper-verb call sites in the same inner let* scope;
;; arc 126 fires at freeze → all :should-panic("channel-pair-deadlock").

;; Layer 2 — put+get round-trip proof.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest-service :wat-tests::holon::lru::HologramCacheService::test-hcs-spawn-put-get
  (:wat::core::let*
    (((results :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
      (:test::hcs-spawn-put-get 16
        (:wat::holon::leaf :alpha)
        (:wat::holon::leaf :av))))
    (:test::hcs-assert-hit results "hcs-spawn-put-get-miss")))


;; Layer 2 — 3-item put+verify proof.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest-service :wat-tests::holon::lru::HologramCacheService::test-hcs-spawn-put-3-verify
  (:test::hcs-spawn-put-3-verify
    (:wat::holon::leaf :alpha)  (:wat::holon::leaf :av)
    (:wat::holon::leaf :beta)   (:wat::holon::leaf :bv)
    (:wat::holon::leaf :gamma)  (:wat::holon::leaf :gv)))


;; Layer 2 — 2-client fan-in proof.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest-service :wat-tests::holon::lru::HologramCacheService::test-hcs-spawn-2clients-put-get-verify
  (:test::hcs-spawn-2clients-put-get-verify
    (:wat::holon::leaf :alpha)  (:wat::holon::leaf :av)
    (:wat::holon::leaf :beta)   (:wat::holon::leaf :bv)))


;; Layer 2 — eviction proof.
(:wat::test::should-panic "channel-pair-deadlock")
(:deftest-service :wat-tests::holon::lru::HologramCacheService::test-hcs-spawn-put-3-eviction
  (:test::hcs-spawn-put-3-eviction
    (:wat::holon::leaf :first)
    (:wat::holon::leaf :second)
    (:wat::holon::leaf :third)
    (:wat::holon::leaf :payload)))


;; ─────────────────────────────────────────────────────────────────────────
;; Steps 3-6 — use :deftest-service. Prelude 2 triggers arc-126 at freeze.
;; All :should-panic("channel-pair-deadlock") + :time-limit "200ms".
;; ─────────────────────────────────────────────────────────────────────────

;; ─── Step 3 — HologramCacheService/spawn + helper verbs (Put N items) ────
;;
;; Arc 119 discipline correction: consumer vantage. Spawn, put 3 items via
;; /put helper verb, verify all 3 are present via /get.
;;
;; Arc 126 — Put-ack helper-verb cycle fires at freeze: expected panic.
(:wat::test::should-panic "channel-pair-deadlock")
(:wat::test::time-limit "200ms")
(:deftest-service :wat-tests::holon::lru::HologramCacheService::test-step3-put-only
  (:test::hcs-spawn-put-3-verify
    (:wat::holon::leaf :alpha)  (:wat::holon::leaf :av)
    (:wat::holon::leaf :beta)   (:wat::holon::leaf :bv)
    (:wat::holon::leaf :gamma)  (:wat::holon::leaf :gv)))

;; ─── Step 4 — Put then Get round-trip via helper verbs ───────────────────
;;
;; Arc 119 discipline correction: consumer vantage. Put one item, get it
;; back, assert the value is present.
;;
;; Arc 126 — same Put-ack cycle; expected panic.
(:wat::test::should-panic "channel-pair-deadlock")
(:wat::test::time-limit "200ms")
(:deftest-service :wat-tests::holon::lru::HologramCacheService::test-step4-put-get-roundtrip
  (:wat::core::let*
    (((results :wat::core::Vector<wat::core::Option<wat::holon::HolonAST>>)
      (:test::hcs-spawn-put-get 16
        (:wat::holon::leaf :alpha)
        (:wat::holon::leaf :av))))
    (:test::hcs-assert-hit results "cache-miss")))

;; ─── Step 5 — full Service constructor + HandlePool fan-in ───────────────
;;
;; Arc 119 discipline correction: consumer vantage. Two clients, each pops
;; its own req-tx, uses helper verbs, each sees its own data.
;;
;; Arc 126 — multi-client holds req-tx + ack-tx pairs in inner scope;
;; expected panic.
(:wat::test::should-panic "channel-pair-deadlock")
(:wat::test::time-limit "200ms")
(:deftest-service :wat-tests::holon::lru::HologramCacheService::test-step5-multi-client-via-constructor
  (:test::hcs-spawn-2clients-put-get-verify
    (:wat::holon::leaf :alpha)  (:wat::holon::leaf :av)
    (:wat::holon::leaf :beta)   (:wat::holon::leaf :bv)))

;; ─── Step 6 — LRU eviction visible through Service Get/Put round-trips ────
;;
;; Arc 119 discipline correction: consumer vantage. cap=2, put k1/k2/k3 —
;; k1 evicts. get(k1) returns None; get(k2) returns Some.
;;
;; Arc 126 — same Pattern B cycle; expected panic.
(:wat::test::should-panic "channel-pair-deadlock")
(:wat::test::time-limit "200ms")
(:deftest-service :wat-tests::holon::lru::HologramCacheService::test-step6-lru-eviction-via-service
  (:test::hcs-spawn-put-3-eviction
    (:wat::holon::leaf :first)
    (:wat::holon::leaf :second)
    (:wat::holon::leaf :third)
    (:wat::holon::leaf :payload)))
