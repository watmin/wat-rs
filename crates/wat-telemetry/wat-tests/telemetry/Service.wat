;; wat-tests/std/telemetry/Service.wat — arc 080 + arc 089 + arc 095
;; smoke tests for the Service<E,G> shell.
;;
;; Arc 130 — complectēns rewrite. Top-down dependency graph in ONE file.
;;
;; ─── Layers ──────────────────────────────────────────────────────────
;;
;;   Layer 0  :test::svc-tel-make-dispatcher    ; stub-tx → dispatcher-fn
;;            :test::svc-tel-null-translator    ; → null stats-translator
;;            :test::svc-tel-active-translator  ; → translator that returns [-1]
;;
;;   Layer 1  :test::svc-tel-spawn-shutdown     ; spawn + shutdown, no traffic
;;            :test::svc-tel-spawn-and-log      ; spawn, batch-log entries, return (Thread, Receiver)
;;
;;   Layer 2  :test::svc-tel-assert-drain-3     ; drain 3 values from Receiver, assert each
;;
;;   Final    :wat-telemetry::test-spawn-drop-join    (1 line)
;;            :wat-telemetry::test-batch-roundtrip    (5 lines)
;;            :wat-telemetry::test-cadence-fires      (5 lines)
;;
;; No arc-126 concern: the stub channel's stub-tx enters the dispatcher
;; closure (via svc-tel-make-dispatcher); stub-rx is returned separately.
;; They are never passed to the same function simultaneously. The
;; req-tx / ack-rx from HandlePool::pop are opaque (not traced back
;; through a WAT-level make-bounded-channel), so batch-log is safe too.

(:wat::test::make-deftest :deftest
  (
   ;; ─── Layer 0 — pure builders ──────────────────────────────────
   ;;
   ;; :test::svc-tel-make-dispatcher — build the stub dispatcher lambda.
   ;; Captures stub-tx in a foldl; each entry is forwarded to the
   ;; stub channel so the test can drain them after join.
   (:wat::core::define
     (:test::svc-tel-make-dispatcher
       (stub-tx :wat::kernel::Sender<wat::core::i64>)
       -> :fn(wat::core::Vector<wat::core::i64>)->wat::core::unit)
     (:wat::core::lambda ((entries :wat::core::Vector<wat::core::i64>) -> :wat::core::unit)
       (:wat::core::foldl entries ()
         (:wat::core::lambda ((_acc :wat::core::unit) (e :wat::core::i64) -> :wat::core::unit)
           (:wat::core::match (:wat::kernel::send stub-tx e) -> :wat::core::unit
             ((:wat::core::Ok _) ())
             ((:wat::core::Err _) ()))))))


   ;; :test::svc-tel-null-translator — build a stats translator that
   ;; always returns an empty i64 vector. Used by tests that don't
   ;; need cadence-fired entries.
   (:wat::core::define
     (:test::svc-tel-null-translator
       -> :fn(wat::telemetry::Stats)->wat::core::Vector<wat::core::i64>)
     (:wat::core::lambda
       ((_s :wat::telemetry::Stats) -> :wat::core::Vector<wat::core::i64>)
       (:wat::core::Vector :wat::core::i64)))


   ;; :test::svc-tel-active-translator — build a stats translator that
   ;; returns a sentinel value [-1]. Used by the cadence-fires test to
   ;; distinguish a cadence-triggered entry from a user-submitted one.
   (:wat::core::define
     (:test::svc-tel-active-translator
       -> :fn(wat::telemetry::Stats)->wat::core::Vector<wat::core::i64>)
     (:wat::core::lambda
       ((_s :wat::telemetry::Stats) -> :wat::core::Vector<wat::core::i64>)
       (:wat::core::Vector :wat::core::i64 -1)))


   ;; ─── Layer 1 — spawn scenarios ────────────────────────────────
   ;;
   ;; :test::svc-tel-spawn-shutdown — full lifecycle with NO traffic.
   ;; Builds a stub channel + null dispatcher + null translator +
   ;; null cadence, spawns 1 client, pops the handle (pop-before-finish),
   ;; finishes the pool, driver exits, joins. Returns unit.
   ;; Driver is Thread<unit,unit> — no recv-before-join needed.
   (:wat::core::define
     (:test::svc-tel-spawn-shutdown -> :wat::core::unit)
     (:wat::core::let*
       (((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
         (:wat::core::let*
           (((stub-pair :wat::kernel::Channel<wat::core::i64>)
             (:wat::kernel::make-bounded-channel :wat::core::i64 16))
            ((stub-tx :wat::kernel::Sender<wat::core::i64>) (:wat::core::first stub-pair))
            ((dispatcher :fn(wat::core::Vector<wat::core::i64>)->wat::core::unit)
             (:test::svc-tel-make-dispatcher stub-tx))
            ((translator :fn(wat::telemetry::Stats)->wat::core::Vector<wat::core::i64>)
             (:test::svc-tel-null-translator))
            ((cadence :wat::telemetry::MetricsCadence<wat::core::unit>)
             (:wat::telemetry::null-metrics-cadence))
            ((spawn :wat::telemetry::Spawn<wat::core::i64>)
             (:wat::telemetry::spawn 1 cadence dispatcher translator))
            ((pool :wat::telemetry::HandlePool<wat::core::i64>)
             (:wat::core::first spawn))
            ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
             (:wat::core::second spawn))
            ((_handle :wat::telemetry::Handle<wat::core::i64>)
             (:wat::kernel::HandlePool::pop pool))
            ((_finish :wat::core::unit)
             (:wat::kernel::HandlePool::finish pool)))
           d))
        ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
         (:wat::kernel::Thread/join-result driver)))
       ()))


   ;; :test::svc-tel-spawn-and-log — spawn telemetry service with given
   ;; entries, translator, and cadence. Builds stub channel + dispatcher,
   ;; spawns 1 client, pops handle, finishes pool, calls batch-log, then
   ;; returns (Thread, stub-rx) for the caller to join and drain.
   ;; Inner scope drops stub-tx (inside dispatcher closure at inner exit)
   ;; and the handle's req-tx, signalling the driver to exit.
   (:wat::core::define
     (:test::svc-tel-spawn-and-log
       (entries :wat::core::Vector<wat::core::i64>)
       (translator :fn(wat::telemetry::Stats)->wat::core::Vector<wat::core::i64>)
       (cadence :wat::telemetry::MetricsCadence<wat::core::i64>)
       -> :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::kernel::Receiver<wat::core::i64>))
     (:wat::core::let*
       (((thr-and-rx :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::kernel::Receiver<wat::core::i64>))
         (:wat::core::let*
           (((stub-pair :wat::kernel::Channel<wat::core::i64>)
             (:wat::kernel::make-bounded-channel :wat::core::i64 16))
            ((stub-tx :wat::kernel::Sender<wat::core::i64>) (:wat::core::first stub-pair))
            ((stub-rx :wat::kernel::Receiver<wat::core::i64>) (:wat::core::second stub-pair))
            ((dispatcher :fn(wat::core::Vector<wat::core::i64>)->wat::core::unit)
             (:test::svc-tel-make-dispatcher stub-tx))
            ((spawn :wat::telemetry::Spawn<wat::core::i64>)
             (:wat::telemetry::spawn 1 cadence dispatcher translator))
            ((pool :wat::telemetry::HandlePool<wat::core::i64>)
             (:wat::core::first spawn))
            ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
             (:wat::core::second spawn))
            ((_inner :wat::core::unit)
             (:wat::core::let*
               (((handle :wat::telemetry::Handle<wat::core::i64>)
                 (:wat::kernel::HandlePool::pop pool))
                ((_finish :wat::core::unit)
                 (:wat::kernel::HandlePool::finish pool))
                ((req-tx :wat::telemetry::ReqTx<wat::core::i64>)
                 (:wat::core::first handle))
                ((ack-rx :wat::telemetry::AckRx)
                 (:wat::core::second handle))
                ((_log :wat::core::unit)
                 (:wat::telemetry::batch-log req-tx ack-rx entries)))
               ())))
           (:wat::core::Tuple d stub-rx))))
       thr-and-rx))


   ;; ─── Layer 2 — drain-and-assert helper ───────────────────────
   ;;
   ;; :test::svc-tel-assert-drain-3 — recv three values from stub-rx
   ;; (match-at-source) and assert each equals the expected value.
   ;; Assumes the caller has already joined the driver so the stub
   ;; channel is fully flushed.
   (:wat::core::define
     (:test::svc-tel-assert-drain-3
       (stub-rx :wat::kernel::Receiver<wat::core::i64>)
       (e1 :wat::core::i64)
       (e2 :wat::core::i64)
       (e3 :wat::core::i64)
       -> :wat::core::unit)
     (:wat::core::let*
       (((v1 :wat::core::i64)
         (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64
           ((:wat::core::Ok (:wat::core::Some v)) v)
           ((:wat::core::Ok :wat::core::None) -99)
           ((:wat::core::Err _) -99)))
        ((v2 :wat::core::i64)
         (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64
           ((:wat::core::Ok (:wat::core::Some v)) v)
           ((:wat::core::Ok :wat::core::None) -99)
           ((:wat::core::Err _) -99)))
        ((v3 :wat::core::i64)
         (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64
           ((:wat::core::Ok (:wat::core::Some v)) v)
           ((:wat::core::Ok :wat::core::None) -99)
           ((:wat::core::Err _) -99)))
        ((_ :wat::core::unit) (:wat::test::assert-eq v1 e1))
        ((_ :wat::core::unit) (:wat::test::assert-eq v2 e2)))
       (:wat::test::assert-eq v3 e3)))

   ))


;; ─── Per-layer deftests ────────────────────────────────────────────────────
;;
;; Each layer carries its own proof. Top-down: helpers proven before
;; they are composed into higher layers and final scenarios.

;; Layer 0 — make-dispatcher: dispatcher forwards entries to stub-rx.
;; Proves the closure captures stub-tx correctly: one entry is forwarded.
(:deftest :wat-telemetry::test-svc-tel-make-dispatcher
  (:wat::core::let*
    (((stub-pair :wat::kernel::Channel<wat::core::i64>)
      (:wat::kernel::make-bounded-channel :wat::core::i64 4))
     ((stub-tx :wat::kernel::Sender<wat::core::i64>) (:wat::core::first stub-pair))
     ((stub-rx :wat::kernel::Receiver<wat::core::i64>) (:wat::core::second stub-pair))
     ((dispatcher :fn(wat::core::Vector<wat::core::i64>)->wat::core::unit)
      (:test::svc-tel-make-dispatcher stub-tx))
     ((_ :wat::core::unit)
      (dispatcher (:wat::core::Vector :wat::core::i64 42)))
     ((v :wat::core::i64)
      (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64
        ((:wat::core::Ok (:wat::core::Some v)) v)
        ((:wat::core::Ok :wat::core::None) -1)
        ((:wat::core::Err _) -1))))
    (:wat::test::assert-eq v 42)))


;; Layer 0 — null-translator: returns empty vector.
(:deftest :wat-telemetry::test-svc-tel-null-translator
  (:wat::core::let*
    (((t :fn(wat::telemetry::Stats)->wat::core::Vector<wat::core::i64>)
      (:test::svc-tel-null-translator))
     ((result :wat::core::Vector<wat::core::i64>)
      (t (:wat::telemetry::Stats/new 1 2 3))))
    (:wat::test::assert-eq (:wat::core::length result) 0)))


;; Layer 0 — active-translator: returns [-1].
(:deftest :wat-telemetry::test-svc-tel-active-translator
  (:wat::core::let*
    (((t :fn(wat::telemetry::Stats)->wat::core::Vector<wat::core::i64>)
      (:test::svc-tel-active-translator))
     ((result :wat::core::Vector<wat::core::i64>)
      (t (:wat::telemetry::Stats/new 0 0 0))))
    (:wat::test::assert-eq (:wat::core::first result) (:wat::core::Some -1))))


;; Layer 1 — spawn-shutdown: full lifecycle with no traffic.
(:deftest :wat-telemetry::test-svc-tel-spawn-shutdown
  (:test::svc-tel-spawn-shutdown))


;; Layer 1 — spawn-and-log: returns (Thread, Receiver) after batch-log.
;; Proves the helper by joining and draining one batch [7].
(:deftest :wat-telemetry::test-svc-tel-spawn-and-log
  (:wat::core::let*
    (((thr-and-rx :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::kernel::Receiver<wat::core::i64>))
      (:test::svc-tel-spawn-and-log
        (:wat::core::Vector :wat::core::i64 7)
        (:test::svc-tel-null-translator)
        (:wat::telemetry::MetricsCadence/new
          0
          (:wat::core::lambda
            ((g :wat::core::i64) (_s :wat::telemetry::Stats) -> :(wat::core::i64,wat::core::bool))
            (:wat::core::Tuple 0 false)))))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::first thr-and-rx))
     ((stub-rx :wat::kernel::Receiver<wat::core::i64>)
      (:wat::core::second thr-and-rx))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver))
     ((v :wat::core::i64)
      (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::i64
        ((:wat::core::Ok (:wat::core::Some v)) v)
        ((:wat::core::Ok :wat::core::None) -1)
        ((:wat::core::Err _) -1))))
    (:wat::test::assert-eq v 7)))


;; Layer 2 — assert-drain-3: drains and asserts three values.
;; Proves with a direct stub channel (no service spawn needed).
(:deftest :wat-telemetry::test-svc-tel-assert-drain-3
  (:wat::core::let*
    (((pair :wat::kernel::Channel<wat::core::i64>)
      (:wat::kernel::make-bounded-channel :wat::core::i64 4))
     ((tx :wat::kernel::Sender<wat::core::i64>) (:wat::core::first pair))
     ((rx :wat::kernel::Receiver<wat::core::i64>) (:wat::core::second pair))
     ((_ :wat::core::unit)
      (:wat::core::match (:wat::kernel::send tx 10) -> :wat::core::unit
        ((:wat::core::Ok _) ()) ((:wat::core::Err _) ())))
     ((_ :wat::core::unit)
      (:wat::core::match (:wat::kernel::send tx 20) -> :wat::core::unit
        ((:wat::core::Ok _) ()) ((:wat::core::Err _) ())))
     ((_ :wat::core::unit)
      (:wat::core::match (:wat::kernel::send tx 30) -> :wat::core::unit
        ((:wat::core::Ok _) ()) ((:wat::core::Err _) ()))))
    (:test::svc-tel-assert-drain-3 rx 10 20 30)))


;; ─── Final scenario deftests ───────────────────────────────────────────────

;; ─── Test 1: spawn + drop + join (no traffic) ────────────────────────────

(:deftest :wat-telemetry::test-spawn-drop-join
  (:test::svc-tel-spawn-shutdown))


;; ─── Test 2: one-batch round-trip ─────────────────────────────────────────
;;
;; Send one batch of 3 entries; drain the stub-rx; assert all three
;; arrived in order.

(:deftest :wat-telemetry::test-batch-roundtrip
  (:wat::core::let*
    (((thr-and-rx :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::kernel::Receiver<wat::core::i64>))
      (:test::svc-tel-spawn-and-log
        (:wat::core::Vector :wat::core::i64 10 20 30)
        (:test::svc-tel-null-translator)
        (:wat::telemetry::MetricsCadence/new
          0
          (:wat::core::lambda
            ((g :wat::core::i64) (_s :wat::telemetry::Stats) -> :(wat::core::i64,wat::core::bool))
            (:wat::core::Tuple 0 false)))))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::first thr-and-rx))
     ((stub-rx :wat::kernel::Receiver<wat::core::i64>)
      (:wat::core::second thr-and-rx))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:test::svc-tel-assert-drain-3 stub-rx 10 20 30)))


;; ─── Test 3: cadence fires → translator called ────────────────────────────

(:deftest :wat-telemetry::test-cadence-fires
  (:wat::core::let*
    (((thr-and-rx :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::kernel::Receiver<wat::core::i64>))
      (:test::svc-tel-spawn-and-log
        (:wat::core::Vector :wat::core::i64 100 200)
        (:test::svc-tel-active-translator)
        (:wat::telemetry::MetricsCadence/new
          0
          (:wat::core::lambda
            ((g :wat::core::i64) (_s :wat::telemetry::Stats) -> :(wat::core::i64,wat::core::bool))
            (:wat::core::Tuple 0 true)))))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
      (:wat::core::first thr-and-rx))
     ((stub-rx :wat::kernel::Receiver<wat::core::i64>)
      (:wat::core::second thr-and-rx))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:test::svc-tel-assert-drain-3 stub-rx 100 200 -1)))
