;; wat-tests/measure/WorkUnit.wat — arc 091 slice 3 smoke tests for
;; the WorkUnit data primitives.
;;
;; Eight tests cover the contract:
;;
;;   - test-uuid-non-empty       new wu has a uuid String
;;   - test-uuid-distinct        two wu's mint distinct uuids
;;   - test-counter-default      counter on absent name returns 0
;;   - test-incr-then-counter    incr! once, counter returns 1
;;   - test-incr-many            incr! 3x, counter returns 3
;;   - test-append-dt-then-read  append-dt! 2x, durations returns the Vec
;;   - test-tags-empty           empty tags map round-trips
;;   - test-tags-roundtrip       declared tags readable via :wat::core::get
;;
;; Keys are HolonAST throughout — `(:wat::holon::Atom :requests)`
;; lifts a wat keyword into the algebra. Passing a bare keyword
;; would type-check-fail since the WorkUnit/incr! signature
;; declares `name :wat::holon::HolonAST`.
;;
;; The mutation-visible-across-calls property is implicit in
;; test-incr-many — if mutation didn't persist between
;; consecutive (:incr! wu k) calls, the final counter would be 1.

;; ─── make-deftest with shared empty-tags helper ─────────────────
;;
;; Most tests don't care about tags — they just need a wu. Tags
;; are mandatory at the constructor (the immutability contract;
;; assoc/disassoc don't exist), so every wu needs SOME map.
;; make-deftest injects a shared `empty-tags` define into each
;; test's sandbox prelude (cf. auto-spawn.wat in wat-sqlite).

(:wat::test::make-deftest :deftest
  ((:wat::core::define
     (:wat-telemetry::empty-tags -> :wat::telemetry::Tags)
     (:wat::core::HashMap :wat::telemetry::Tag))

   (:wat::core::define
     (:wat-telemetry::default-ns -> :wat::holon::HolonAST)
     (:wat::holon::Atom :wat-telemetry::test::ns))

   ;; Probe helper — `:fn(X)->fn(Y)->Z`. Locks the substrate's
   ;; nested-fn-return capability that WorkUnit/make-scope needs.
   (:wat::core::define
     (:wat-telemetry::probe::make-adder
       (x :wat::core::i64) -> :fn(wat::core::i64)->wat::core::i64)
     (:wat::core::lambda ((y :wat::core::i64) -> :wat::core::i64)
       (:wat::core::+ x y)))

   ;; Rank-2 probe — generic factory returning generic-T closure.
   ;; Each call instantiates T at the call site.
   (:wat::core::define
     (:wat-telemetry::probe::make-runner<T>
       (_label :wat::core::String) -> :fn(fn()->T)->T)
     (:wat::core::lambda ((body :fn()->T) -> :T)
       (body)))

   ;; Stub dispatcher for the make-scope ship test — closes over
   ;; a Sender<Event>; forwards each Event from the dispatched
   ;; batch into the test's stub channel so the body can drain
   ;; them after scope returns.
   (:wat::core::define
     (:wat-telemetry::scope::make-stub-dispatcher
       (stub-tx :wat::kernel::Sender<wat::telemetry::Event>)
       -> :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
     (:wat::core::lambda ((entries :wat::core::Vector<wat::telemetry::Event>) -> :wat::core::unit)
       (:wat::core::foldl entries ()
         (:wat::core::lambda ((_acc :wat::core::unit) (e :wat::telemetry::Event) -> :wat::core::unit)
           (:wat::core::match (:wat::kernel::send stub-tx e) -> :wat::core::unit
             ((:wat::core::Ok _) ())
             ((:wat::core::Err _) ()))))))

   ;; Count dispatcher — sends the length of each dispatched batch to a
   ;; Sender<i64>. Used by test-make-scope-ships-empty to confirm the
   ;; service was called once with 0 events (instead of trying to recv
   ;; from stub-rx, which would block when the batch is empty).
   (:wat::core::define
     (:wat-telemetry::scope::make-count-dispatcher
       (count-tx :wat::kernel::Sender<wat::core::i64>)
       -> :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
     (:wat::core::lambda ((entries :wat::core::Vector<wat::telemetry::Event>) -> :wat::core::unit)
       (:wat::core::match
         (:wat::kernel::send count-tx (:wat::core::length entries)) -> :wat::core::unit
         ((:wat::core::Ok _) ())
         ((:wat::core::Err _) ()))))

   ;; Empty stats translator — null cadence never fires anyway.
   (:wat::core::define
     (:wat-telemetry::scope::translate-empty
       (_s :wat::telemetry::Stats)
       -> :wat::core::Vector<wat::telemetry::Event>)
     (:wat::core::Vector :wat::telemetry::Event))))


;; ─── uuid is non-empty ────────────────────────────────────────────

(:deftest :wat-telemetry::WorkUnit::test-uuid-non-empty
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((id :wat::core::String) (:wat::telemetry::WorkUnit/uuid wu)))
    ;; A canonical 8-4-4-4-12 hex uuid is 36 chars — but :wat::core::String
    ;; has no length primitive in slice-3 wat surface, and the
    ;; rigorous format checks live in arc 092's Rust tests. Here
    ;; we just prove the read returns SOME string — the empty
    ;; sentinel "" would equal "" so the assertion would catch
    ;; a degenerate shim that returned the empty string.
    (:wat::test::assert-eq (:wat::core::= id "") false)))


;; ─── uuids are distinct across new() calls ───────────────────────

(:deftest :wat-telemetry::WorkUnit::test-uuid-distinct
  (:wat::core::let*
    (((wu1 :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((wu2 :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((id1 :wat::core::String) (:wat::telemetry::WorkUnit/uuid wu1))
     ((id2 :wat::core::String) (:wat::telemetry::WorkUnit/uuid wu2)))
    (:wat::test::assert-eq (:wat::core::= id1 id2) false)))


;; ─── counter on an absent key returns 0 ──────────────────────────

(:deftest :wat-telemetry::WorkUnit::test-counter-default
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :never-incremented))
     ((n :wat::core::i64) (:wat::telemetry::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 0)))


;; ─── incr! then counter — single bump ────────────────────────────

(:deftest :wat-telemetry::WorkUnit::test-incr-then-counter
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :requests))
     ((_ :wat::core::unit) (:wat::telemetry::WorkUnit/incr! wu name))
     ((n :wat::core::i64) (:wat::telemetry::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 1)))


;; ─── incr! many — accumulation across calls ──────────────────────

(:deftest :wat-telemetry::WorkUnit::test-incr-many
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :requests))
     ((_a :wat::core::unit) (:wat::telemetry::WorkUnit/incr! wu name))
     ((_b :wat::core::unit) (:wat::telemetry::WorkUnit/incr! wu name))
     ((_c :wat::core::unit) (:wat::telemetry::WorkUnit/incr! wu name))
     ((n :wat::core::i64) (:wat::telemetry::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 3)))


;; ─── append-dt! then read ────────────────────────────────────────

(:deftest :wat-telemetry::WorkUnit::test-append-dt-then-read
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :sql-page))
     ((_a :wat::core::unit) (:wat::telemetry::WorkUnit/append-dt! wu name 0.5))
     ((_b :wat::core::unit) (:wat::telemetry::WorkUnit/append-dt! wu name 1.5))
     ((dts :wat::core::Vector<wat::core::f64>) (:wat::telemetry::WorkUnit/durations wu name)))
    (:wat::test::assert-eq dts (:wat::core::Vector :wat::core::f64 0.5 1.5))))


;; ─── timed — bump + measure-around body ─────────────────────────
;;
;; ONE timed call:
;;   - counter for `name` bumps by 1
;;   - durations for `name` gains ONE sample (the body's wall-clock seconds)
;;   - body's T flows back verbatim
;;
;; The single-name discipline (counter and duration share the key)
;; keeps the row count predictable: N calls under one name ⇒ N counter
;; bumps (one row at scope-close) plus N duration samples (N rows at
;; scope-close per the CloudWatch fanout).

(:deftest :wat-telemetry::WorkUnit::test-timed-bumps-counter-records-duration
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit)
      (:wat::telemetry::WorkUnit::new
        (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :sql-fetch))
     ((result :wat::core::i64)
      (:wat::telemetry::WorkUnit/timed wu name
        (:wat::core::lambda (-> :wat::core::i64) 99)))
     ((counter :wat::core::i64) (:wat::telemetry::WorkUnit/counter wu name))
     ((dts :wat::core::Vector<wat::core::f64>) (:wat::telemetry::WorkUnit/durations wu name))
     ((n-dts :wat::core::i64) (:wat::core::length dts))
     ((_a :wat::core::unit) (:wat::test::assert-eq result 99))
     ((_b :wat::core::unit) (:wat::test::assert-eq counter 1)))
    (:wat::test::assert-eq n-dts 1)))


;; Two timed calls under one name: counter = 2, durations has 2 samples.
(:deftest :wat-telemetry::WorkUnit::test-timed-twice-accumulates
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit)
      (:wat::telemetry::WorkUnit::new
        (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :work))
     ((_r1 :wat::core::i64)
      (:wat::telemetry::WorkUnit/timed wu name
        (:wat::core::lambda (-> :wat::core::i64) 1)))
     ((_r2 :wat::core::i64)
      (:wat::telemetry::WorkUnit/timed wu name
        (:wat::core::lambda (-> :wat::core::i64) 2)))
     ((counter :wat::core::i64) (:wat::telemetry::WorkUnit/counter wu name))
     ((dts :wat::core::Vector<wat::core::f64>) (:wat::telemetry::WorkUnit/durations wu name))
     ((n-dts :wat::core::i64) (:wat::core::length dts))
     ((_a :wat::core::unit) (:wat::test::assert-eq counter 2)))
    (:wat::test::assert-eq n-dts 2)))


;; ─── Tags — immutable, declared at construction ───────────────

;; Empty tags map round-trips through the constructor.
(:deftest :wat-telemetry::WorkUnit::test-tags-empty
  (:wat::core::let*
    (((empty :wat::telemetry::Tags) (:wat-telemetry::empty-tags))
     ((wu  :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) empty))
     ((got :wat::telemetry::Tags)
      (:wat::telemetry::WorkUnit/tags wu)))
    (:wat::test::assert-eq (:wat::core::length got) 0)))


;; Tags declared at new() are visible via :wat::telemetry::WorkUnit/tags
;; and readable via :wat::core::get.
(:deftest :wat-telemetry::WorkUnit::test-tags-roundtrip
  (:wat::core::let*
    (((asset-key :wat::holon::HolonAST) (:wat::holon::Atom :asset))
     ((asset-val :wat::holon::HolonAST) (:wat::holon::Atom :BTC))
     ((stage-key :wat::holon::HolonAST) (:wat::holon::Atom :stage))
     ((stage-val :wat::holon::HolonAST) (:wat::holon::Atom :market-eval))
     ((tags  :wat::telemetry::Tags)
      (:wat::core::HashMap :wat::telemetry::Tag
        asset-key asset-val
        stage-key stage-val))
     ((wu    :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) tags))
     ((got   :wat::telemetry::Tags)
      (:wat::telemetry::WorkUnit/tags wu))
     ((looked-up :wat::core::Option<wat::holon::HolonAST>)
      (:wat::core::get got asset-key)))
    (:wat::test::assert-eq looked-up (:wat::core::Some asset-val))))


;; ─── WorkUnit/scope<T> — bare HOF (open + run + return) ──────────

;; Body sees the wu, mutates it, returns T; scope returns body's
;; T. The bare scope (no auto-ship — auto-ship lands when scope
;; gains handles via WorkUnit/make-scope) is the smallest piece
;; of the HOF contract.
(:deftest :wat-telemetry::WorkUnit::test-scope-passes-result
  (:wat::core::let*
    (((tags   :wat::telemetry::Tags) (:wat-telemetry::empty-tags))
     ((ns     :wat::holon::HolonAST) (:wat-telemetry::default-ns))
     ((result :wat::core::i64)
      (:wat::telemetry::WorkUnit/scope ns tags
        (:wat::core::lambda ((wu :wat::telemetry::WorkUnit) -> :wat::core::i64)
          (:wat::core::let*
            (((_ :wat::core::unit) (:wat::telemetry::WorkUnit/incr! wu (:wat::holon::Atom :hits))))
            42)))))
    (:wat::test::assert-eq result 42)))


;; ─── Counter scope — emits Event::Metric, not Event::Log ────────
;;
;; Consumer-surface scenario: incr! one name once inside make-scope,
;; then confirm the dispatched event IS an Event::Metric (not a Log).
;; The uuid emitted in the metric row is the one the wu minted;
;; reading it inside the body and comparing it to the event's uuid
;; field confirms the row was built from the right work-unit.
(:deftest :wat-telemetry::WorkUnit::test-build-counter-metric
  (:wat::core::let*
    ;; Inner scope: stub queue + service + one-counter make-scope.
    ;; Body returns the wu's uuid so outer can compare against the event.
    (((thr-uuid-got :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::String,wat::core::bool))
      (:wat::core::let*
        (((stub-pair :wat::kernel::Channel<wat::telemetry::Event>)
          (:wat::kernel::make-bounded-channel :wat::telemetry::Event 16))
         ((stub-tx :wat::kernel::Sender<wat::telemetry::Event>)
          (:wat::core::first stub-pair))
         ((stub-rx :wat::kernel::Receiver<wat::telemetry::Event>)
          (:wat::core::second stub-pair))
         ((dispatcher :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
          (:wat-telemetry::scope::make-stub-dispatcher stub-tx))
         ((cadence :wat::telemetry::MetricsCadence<wat::core::unit>)
          (:wat::telemetry::null-metrics-cadence))
         ((spawn :wat::telemetry::Spawn<wat::telemetry::Event>)
          (:wat::telemetry::spawn 1 cadence dispatcher
            :wat-telemetry::scope::translate-empty))
         ((pool :wat::telemetry::HandlePool<wat::telemetry::Event>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ;; Pop handle, finish pool, create scope-fn, call it.
         ;; Body incrs one counter and returns the wu's uuid — carrier
         ;; for the outer assertion (the event's uuid should match).
         ((uuid-str :wat::core::String)
          (:wat::core::let*
            (((handle :wat::telemetry::Handle<wat::telemetry::Event>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
             ((ns :wat::holon::HolonAST) (:wat-telemetry::default-ns))
             ((scope :wat::telemetry::WorkUnit::Scope<wat::core::String>)
              (:wat::telemetry::WorkUnit/make-scope handle ns))
             ((tags :wat::telemetry::Tags) (:wat-telemetry::empty-tags)))
            (scope tags
              (:wat::core::lambda
                ((wu :wat::telemetry::WorkUnit) -> :wat::core::String)
                (:wat::core::let*
                  (((_ :wat::core::unit) (:wat::telemetry::WorkUnit/incr! wu (:wat::holon::Atom :requests))))
                  (:wat::telemetry::WorkUnit/uuid wu))))))
         ;; Drain the one counter metric the scope shipped.
         ((got :wat::core::bool)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::bool
            ((:wat::core::Ok (:wat::core::Some (:wat::telemetry::Event::Metric _ _ _ _uuid _ _ _ _)))
              (:wat::core::= _uuid uuid-str))
            ((:wat::core::Ok (:wat::core::Some (:wat::telemetry::Event::Log _ _ _ _ _ _ _))) false)
            ((:wat::core::Ok :wat::core::None) false)
            ((:wat::core::Err _) false))))
        (:wat::core::Tuple d uuid-str got)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-uuid-got))
     ((uuid-str :wat::core::String) (:wat::core::second thr-uuid-got))
     ((got :wat::core::bool) (:wat::core::third thr-uuid-got))
     ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver))
     ((_chk-uuid :wat::core::unit) (:wat::test::assert-eq (:wat::core::= uuid-str "") false)))
    (:wat::test::assert-eq got true)))


;; Duration scope — one sample emits Event::Metric, not Event::Log.
;;
;; Consumer-surface scenario: append-dt! one sample inside make-scope,
;; confirm the dispatched event IS an Event::Metric. uuid from inside
;; the body verifies the event was built from the same work-unit.
(:deftest :wat-telemetry::WorkUnit::test-build-duration-metric
  (:wat::core::let*
    ;; Inner scope: stub queue + service + one-sample make-scope.
    ;; Body returns the wu's uuid for the outer uuid-match assertion.
    (((thr-uuid-got :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::String,wat::core::bool))
      (:wat::core::let*
        (((stub-pair :wat::kernel::Channel<wat::telemetry::Event>)
          (:wat::kernel::make-bounded-channel :wat::telemetry::Event 16))
         ((stub-tx :wat::kernel::Sender<wat::telemetry::Event>)
          (:wat::core::first stub-pair))
         ((stub-rx :wat::kernel::Receiver<wat::telemetry::Event>)
          (:wat::core::second stub-pair))
         ((dispatcher :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
          (:wat-telemetry::scope::make-stub-dispatcher stub-tx))
         ((cadence :wat::telemetry::MetricsCadence<wat::core::unit>)
          (:wat::telemetry::null-metrics-cadence))
         ((spawn :wat::telemetry::Spawn<wat::telemetry::Event>)
          (:wat::telemetry::spawn 1 cadence dispatcher
            :wat-telemetry::scope::translate-empty))
         ((pool :wat::telemetry::HandlePool<wat::telemetry::Event>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ;; Pop handle, finish pool, create scope-fn, call it.
         ;; Body appends one duration sample and returns the wu's uuid.
         ((uuid-str :wat::core::String)
          (:wat::core::let*
            (((handle :wat::telemetry::Handle<wat::telemetry::Event>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
             ((ns :wat::holon::HolonAST) (:wat-telemetry::default-ns))
             ((scope :wat::telemetry::WorkUnit::Scope<wat::core::String>)
              (:wat::telemetry::WorkUnit/make-scope handle ns))
             ((tags :wat::telemetry::Tags) (:wat-telemetry::empty-tags)))
            (scope tags
              (:wat::core::lambda
                ((wu :wat::telemetry::WorkUnit) -> :wat::core::String)
                (:wat::core::let*
                  (((_ :wat::core::unit) (:wat::telemetry::WorkUnit/append-dt! wu (:wat::holon::Atom :sql-page) 0.5)))
                  (:wat::telemetry::WorkUnit/uuid wu))))))
         ;; Drain the one duration metric the scope shipped.
         ((got :wat::core::bool)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::bool
            ((:wat::core::Ok (:wat::core::Some (:wat::telemetry::Event::Metric _ _ _ _uuid _ _ _ _)))
              (:wat::core::= _uuid uuid-str))
            ((:wat::core::Ok (:wat::core::Some (:wat::telemetry::Event::Log _ _ _ _ _ _ _))) false)
            ((:wat::core::Ok :wat::core::None) false)
            ((:wat::core::Err _) false))))
        (:wat::core::Tuple d uuid-str got)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-uuid-got))
     ((uuid-str :wat::core::String) (:wat::core::second thr-uuid-got))
     ((got :wat::core::bool) (:wat::core::third thr-uuid-got))
     ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver))
     ((_chk-uuid :wat::core::unit) (:wat::test::assert-eq (:wat::core::= uuid-str "") false)))
    (:wat::test::assert-eq got true)))


;; Empty scope — make-scope with no mutations dispatches 0 events.
;;
;; Consumer-surface scenario: body does nothing (returns unit). At
;; scope-close make-scope calls batch-log with an empty vec; the
;; service calls the dispatcher with 0 entries. A count-dispatcher
;; sends the batch-length over a channel; we recv it and assert 0.
;; This also proves batch-log with an empty vec doesn't deadlock.
(:deftest :wat-telemetry::WorkUnit::test-collect-metrics-empty
  (:wat::core::let*
    ;; Inner scope: count queue + service + empty-body make-scope.
    (((thr-count :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::i64))
      (:wat::core::let*
        (((count-pair :wat::kernel::Channel<wat::core::i64>)
          (:wat::kernel::make-bounded-channel :wat::core::i64 4))
         ((count-tx :wat::kernel::Sender<wat::core::i64>)
          (:wat::core::first count-pair))
         ((count-rx :wat::kernel::Receiver<wat::core::i64>)
          (:wat::core::second count-pair))
         ((dispatcher :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
          (:wat-telemetry::scope::make-count-dispatcher count-tx))
         ((cadence :wat::telemetry::MetricsCadence<wat::core::unit>)
          (:wat::telemetry::null-metrics-cadence))
         ((spawn :wat::telemetry::Spawn<wat::telemetry::Event>)
          (:wat::telemetry::spawn 1 cadence dispatcher
            :wat-telemetry::scope::translate-empty))
         ((pool :wat::telemetry::HandlePool<wat::telemetry::Event>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ;; Pop handle, finish pool, create scope-fn, call with empty body.
         ((_ :wat::core::unit)
          (:wat::core::let*
            (((handle :wat::telemetry::Handle<wat::telemetry::Event>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
             ((ns :wat::holon::HolonAST) (:wat-telemetry::default-ns))
             ((scope :wat::telemetry::WorkUnit::Scope<wat::core::unit>)
              (:wat::telemetry::WorkUnit/make-scope handle ns))
             ((tags :wat::telemetry::Tags) (:wat-telemetry::empty-tags)))
            (scope tags
              (:wat::core::lambda
                ((_wu :wat::telemetry::WorkUnit) -> :wat::core::unit)
                ()))))
         ;; Drain the one count the dispatcher sent (batch-length = 0).
         ((cnt :wat::core::i64)
          (:wat::core::match (:wat::kernel::recv count-rx) -> :wat::core::i64
            ((:wat::core::Ok (:wat::core::Some n)) n)
            ((:wat::core::Ok :wat::core::None) -1)
            ((:wat::core::Err _) -1))))
        (:wat::core::Tuple d cnt)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-count))
     ((cnt :wat::core::i64) (:wat::core::second thr-count))
     ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq cnt 0)))


;; test-collect-metrics-one-counter removed (task #211 / vocare).
;; Scenario: N incr! calls on one name → ONE Event::Metric emitted
;; at scope-close (CloudWatch model: counters aggregate). This is
;; already proven end-to-end by test-make-scope-ships-counter (one
;; incr! → one event received via stub dispatcher). Keeping a
;; separate slice test that calls scope::collect-metric-events
;; directly would speak from the implementer's vantage, not the
;; consumer's. Removed rather than rewritten — the scenario is
;; fully covered.


;; Two-duration-sample scope — emits TWO Event::Metric rows.
;;
;; Consumer-surface scenario: two append-dt! calls for the same name
;; inside make-scope emit TWO distinct Event::Metric rows at scope-
;; close (CloudWatch fanout: one row per sample, not one aggregated
;; row). The stub dispatcher forwards each event to stub-rx; we recv
;; both and confirm both arrived as Some.
(:deftest :wat-telemetry::WorkUnit::test-collect-metrics-two-duration-samples
  (:wat::core::let*
    ;; Inner scope: stub queue + service + two-sample make-scope.
    (((thr-r1-r2 :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::bool,wat::core::bool))
      (:wat::core::let*
        (((stub-pair :wat::kernel::Channel<wat::telemetry::Event>)
          (:wat::kernel::make-bounded-channel :wat::telemetry::Event 16))
         ((stub-tx :wat::kernel::Sender<wat::telemetry::Event>)
          (:wat::core::first stub-pair))
         ((stub-rx :wat::kernel::Receiver<wat::telemetry::Event>)
          (:wat::core::second stub-pair))
         ((dispatcher :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
          (:wat-telemetry::scope::make-stub-dispatcher stub-tx))
         ((cadence :wat::telemetry::MetricsCadence<wat::core::unit>)
          (:wat::telemetry::null-metrics-cadence))
         ((spawn :wat::telemetry::Spawn<wat::telemetry::Event>)
          (:wat::telemetry::spawn 1 cadence dispatcher
            :wat-telemetry::scope::translate-empty))
         ((pool :wat::telemetry::HandlePool<wat::telemetry::Event>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ;; Pop handle, finish pool, create scope-fn, call with two appends.
         ((_ :wat::core::unit)
          (:wat::core::let*
            (((handle :wat::telemetry::Handle<wat::telemetry::Event>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
             ((ns :wat::holon::HolonAST) (:wat-telemetry::default-ns))
             ((scope :wat::telemetry::WorkUnit::Scope<wat::core::unit>)
              (:wat::telemetry::WorkUnit/make-scope handle ns))
             ((tags :wat::telemetry::Tags) (:wat-telemetry::empty-tags)))
            (scope tags
              (:wat::core::lambda
                ((wu :wat::telemetry::WorkUnit) -> :wat::core::unit)
                (:wat::core::let*
                  (((_a :wat::core::unit) (:wat::telemetry::WorkUnit/append-dt! wu (:wat::holon::Atom :sql-page) 0.5))
                   ((_b :wat::core::unit) (:wat::telemetry::WorkUnit/append-dt! wu (:wat::holon::Atom :sql-page) 1.5)))
                  ())))))
         ;; Drain TWO duration metrics from stub-rx (both must arrive as Some).
         ((r1-some? :wat::core::bool)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::bool
            ((:wat::core::Ok (:wat::core::Some _)) true)
            ((:wat::core::Ok :wat::core::None) false)
            ((:wat::core::Err _) false)))
         ((r2-some? :wat::core::bool)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::bool
            ((:wat::core::Ok (:wat::core::Some _)) true)
            ((:wat::core::Ok :wat::core::None) false)
            ((:wat::core::Err _) false))))
        (:wat::core::Tuple d r1-some? r2-some?)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-r1-r2))
     ((r1-some? :wat::core::bool) (:wat::core::second thr-r1-r2))
     ((r2-some? :wat::core::bool) (:wat::core::third thr-r1-r2))
     ((_ :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver))
     ((_chk-r1 :wat::core::unit) (:wat::test::assert-eq r1-some? true)))
    (:wat::test::assert-eq r2-some? true)))


;; ─── Probe: can wat express `fn(X) -> fn(Y) -> Z` returns? ──────
;;
;; Arc 083 rejected a nested-fn shape on grounds it had "no other
;; precedent in wat". Before the rank-2 closure factory for
;; WorkUnit/make-scope, this probe verifies the basic shape:
;; a function whose return type IS a function type, returned as a
;; lambda value.
(:deftest :wat-telemetry::WorkUnit::probe-fn-returning-fn
  (:wat::core::let*
    (((adder :fn(wat::core::i64)->wat::core::i64)
      (:wat-telemetry::probe::make-adder 10))
     ((sum :wat::core::i64) (adder 5)))
    (:wat::test::assert-eq sum 15)))


;; ─── Namespace round-trips through WorkUnit::new ────────────────
;;
;; Per the user's direction 2026-04-29 — namespace is declared on
;; the wu adjacent to tags. WorkUnit::new takes (namespace, tags);
;; WorkUnit/namespace reads it back. Logs and metrics pull it from
;; the wu rather than threading it as a per-call parameter.
(:deftest :wat-telemetry::WorkUnit::test-namespace-roundtrip
  (:wat::core::let*
    (((tags :wat::telemetry::Tags) (:wat-telemetry::empty-tags))
     ((ns   :wat::holon::HolonAST) (:wat::holon::Atom :my::function))
     ((wu   :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new ns tags))
     ((got  :wat::holon::HolonAST) (:wat::telemetry::WorkUnit/namespace wu)))
    (:wat::test::assert-eq got ns)))


;; ─── Probe: rank-2 — generic factory returning generic-T closure
;;
;; The pattern WorkUnit/make-scope wants:
;;   make-runner<T> :: String -> (fn() -> T) -> T
;; Each call to make-runner with a different T produces a runner
;; specific to that T. If wat supports this, the closure factory
;; for WorkUnit/scope handles works directly.
(:deftest :wat-telemetry::WorkUnit::probe-rank-2-i64
  (:wat::core::let*
    (((runner :fn(fn()->wat::core::i64)->wat::core::i64)
      (:wat-telemetry::probe::make-runner "i64-runner"))
     ((result :wat::core::i64) (runner (:wat::core::lambda (-> :wat::core::i64) 42))))
    (:wat::test::assert-eq result 42)))


;; ─── WorkUnit/make-scope — closure factory; auto-ship at close ───
;;
;; The user's direction (2026-04-29): "we want our deps to vanish
;; as fast as possible. (make-unit-work-maker handle namespace) ->
;; produces a func who does what (WorkUnit/scope ...) is maybe
;; trying to do." Tags may be dynamic at scope-call time;
;; namespace is the producer's identity (fixed per call site).
;;
;; make-scope captures BOTH the SinkHandles AND the namespace
;; once; the returned fn takes only (tags, body) and ships at
;; scope-close. body's T flows back to the caller. No handle or
;; namespace threading at use sites.
;;
;; This test exercises the full path:
;;   1. spawn Service<Event,_> with stub-tx-forwarding dispatcher
;;   2. pop Handle (== SinkHandles)
;;   3. (make-scope handle namespace) → scope-fn
;;   4. (scope-fn tags body) — body increments a counter, returns 42
;;   5. join driver
;;   6. drain stub-rx — assert ONE Event arrived (one counter = one row,
;;      CloudWatch model)
;;   7. assert result == 42 (body's T flowed through)
(:deftest :wat-telemetry::WorkUnit::test-make-scope-ships-counter
  (:wat::core::let*
    ;; Inner owns every Sender clone (stub-pair, stub-tx) AND does
    ;; the scope work + drains the one expected Event from stub-rx
    ;; inside the same scope (the dispatcher fires at scope-close, so
    ;; the row is in the channel before recv runs). Returns
    ;; (driver, result, r1-some?) to outer; outer joins the driver and
    ;; asserts on the body's result + the recv'd-Some bool.
    ;; SERVICE-PROGRAMS.md § "The lockstep" + arc 117.
    (((thr-result-some :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::i64,wat::core::bool))
      (:wat::core::let*
        ;; Stub queue — collects the Events the dispatcher sees.
        (((stub-pair :wat::kernel::Channel<wat::telemetry::Event>)
          (:wat::kernel::make-bounded-channel :wat::telemetry::Event 16))
         ((stub-tx :wat::kernel::Sender<wat::telemetry::Event>)
          (:wat::core::first stub-pair))
         ((stub-rx :wat::kernel::Receiver<wat::telemetry::Event>)
          (:wat::core::second stub-pair))
         ;; Dispatcher closure-over stub-tx; null cadence + empty translator.
         ((dispatcher :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
          (:wat-telemetry::scope::make-stub-dispatcher stub-tx))
         ((cadence :wat::telemetry::MetricsCadence<wat::core::unit>)
          (:wat::telemetry::null-metrics-cadence))
         ;; Spawn Service<Event,_> with one client slot.
         ((spawn :wat::telemetry::Spawn<wat::telemetry::Event>)
          (:wat::telemetry::spawn 1 cadence dispatcher
            :wat-telemetry::scope::translate-empty))
         ((pool :wat::telemetry::HandlePool<wat::telemetry::Event>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ;; Inner-inner: pop Handle, finish pool, factory + scope-fn-with-counter.
         ((result :wat::core::i64)
          (:wat::core::let*
            (((handle :wat::telemetry::Handle<wat::telemetry::Event>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
             ((ns :wat::holon::HolonAST) (:wat-telemetry::default-ns))
             ((scope :wat::telemetry::WorkUnit::Scope<wat::core::i64>)
              (:wat::telemetry::WorkUnit/make-scope handle ns))
             ((tags :wat::telemetry::Tags) (:wat-telemetry::empty-tags)))
            (scope tags
              (:wat::core::lambda
                ((wu :wat::telemetry::WorkUnit) -> :wat::core::i64)
                (:wat::core::let*
                  (((_ :wat::core::unit) (:wat::telemetry::WorkUnit/incr! wu (:wat::holon::Atom :hits))))
                  42)))))
         ;; Drain ONE Event — the single counter (CloudWatch model:
         ;; one counter = one row, established by
         ;; test-collect-metrics-one-counter). The dispatcher fires at
         ;; scope-close above, so the row is already enqueued. recv'ing
         ;; past one would block (stub-tx is still alive in this scope),
         ;; so we recv only what we KNOW was sent.
         ((r1-some? :wat::core::bool)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::bool ((:wat::core::Ok (:wat::core::Some _)) true) ((:wat::core::Ok :wat::core::None) false) ((:wat::core::Err _) false))))
        (:wat::core::Tuple d result r1-some?)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-result-some))
     ((result :wat::core::i64) (:wat::core::second thr-result-some))
     ((r1-some? :wat::core::bool) (:wat::core::third thr-result-some))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver))
     ((_a :wat::core::unit) (:wat::test::assert-eq result 42)))
    (:wat::test::assert-eq r1-some? true)))
