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
       (x :wat::core::i64) -> :fn(i64)->i64)
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
   ;; a QueueSender<Event>; forwards each Event from the dispatched
   ;; batch into the test's stub queue so the body can drain
   ;; them after scope returns.
   (:wat::core::define
     (:wat-telemetry::scope::make-stub-dispatcher
       (stub-tx :wat::kernel::QueueSender<wat::telemetry::Event>)
       -> :fn(Vec<wat::telemetry::Event>)->())
     (:wat::core::lambda ((entries :Vec<wat::telemetry::Event>) -> :())
       (:wat::core::foldl entries ()
         (:wat::core::lambda ((_acc :()) (e :wat::telemetry::Event) -> :())
           (:wat::core::match (:wat::kernel::send stub-tx e) -> :()
             ((Some _) ())
             (:None    ()))))))

   ;; Empty stats translator — null cadence never fires anyway.
   (:wat::core::define
     (:wat-telemetry::scope::translate-empty
       (_s :wat::telemetry::Service::Stats)
       -> :Vec<wat::telemetry::Event>)
     (:wat::core::vec :wat::telemetry::Event))))


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
     ((_ :()) (:wat::telemetry::WorkUnit/incr! wu name))
     ((n :wat::core::i64) (:wat::telemetry::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 1)))


;; ─── incr! many — accumulation across calls ──────────────────────

(:deftest :wat-telemetry::WorkUnit::test-incr-many
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :requests))
     ((_a :()) (:wat::telemetry::WorkUnit/incr! wu name))
     ((_b :()) (:wat::telemetry::WorkUnit/incr! wu name))
     ((_c :()) (:wat::telemetry::WorkUnit/incr! wu name))
     ((n :wat::core::i64) (:wat::telemetry::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 3)))


;; ─── append-dt! then read ────────────────────────────────────────

(:deftest :wat-telemetry::WorkUnit::test-append-dt-then-read
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :sql-page))
     ((_a :()) (:wat::telemetry::WorkUnit/append-dt! wu name 0.5))
     ((_b :()) (:wat::telemetry::WorkUnit/append-dt! wu name 1.5))
     ((dts :Vec<f64>) (:wat::telemetry::WorkUnit/durations wu name)))
    (:wat::test::assert-eq dts (:wat::core::vec :wat::core::f64 0.5 1.5))))


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
     ((dts :Vec<f64>) (:wat::telemetry::WorkUnit/durations wu name))
     ((n-dts :wat::core::i64) (:wat::core::length dts))
     ((_a :()) (:wat::test::assert-eq result 99))
     ((_b :()) (:wat::test::assert-eq counter 1)))
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
     ((dts :Vec<f64>) (:wat::telemetry::WorkUnit/durations wu name))
     ((n-dts :wat::core::i64) (:wat::core::length dts))
     ((_a :()) (:wat::test::assert-eq counter 2)))
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
     ((looked-up :Option<wat::holon::HolonAST>)
      (:wat::core::get got asset-key)))
    (:wat::test::assert-eq looked-up (Some asset-val))))


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
            (((_ :()) (:wat::telemetry::WorkUnit/incr! wu (:wat::holon::Atom :hits))))
            42)))))
    (:wat::test::assert-eq result 42)))


;; ─── Slice 4-ship helpers — build-counter-metric ────────────────

;; Helper takes (start-time-ns, end-time-ns, namespace,
;; uuid, tags, name, count) and constructs an Event::Metric with
;; the four NoTag-typed fields wrapped via NoTag/new and the
;; metric-value lifted via leaf. Three primitive fields land
;; verbatim (start-time-ns, end-time-ns, uuid); the rest go
;; through HolonAST encoding. The test asserts the primitive
;; fields and that the variant is Metric (not Log).
(:deftest :wat-telemetry::WorkUnit::test-build-counter-metric
  (:wat::core::let*
    (((tags  :wat::telemetry::Tags)        (:wat-telemetry::empty-tags))
     ((ns    :wat::holon::HolonAST)        (:wat::holon::Atom :my::ns))
     ((name  :wat::holon::HolonAST)        (:wat::holon::Atom :requests))
     ((event :wat::telemetry::Event)
      (:wat::telemetry::WorkUnit/scope::build-counter-metric
        100 200 ns "test-uuid" tags name 7)))
    (:wat::core::match event -> :()
      ((:wat::telemetry::Event::Metric s e _ uuid _ _ _ _)
        (:wat::core::let*
          (((_a :()) (:wat::test::assert-eq s 100))
           ((_b :()) (:wat::test::assert-eq e 200)))
          (:wat::test::assert-eq uuid "test-uuid")))
      ((:wat::telemetry::Event::Log _ _ _ _ _ _ _)
        (:wat::test::assert-eq "expected-Metric-variant" "got-Log-instead")))))


;; build-duration-metric — same shape as build-counter-metric but
;; takes one f64 sample (not a count) and emits unit `:seconds`.
;; ONE sample = ONE row (CloudWatch model). N samples in the wu's
;; durations Vec mean N rows at scope-close, all sharing the same
;; (start, end, namespace, uuid, tags, name) — only metric-value
;; differs across them.
(:deftest :wat-telemetry::WorkUnit::test-build-duration-metric
  (:wat::core::let*
    (((tags  :wat::telemetry::Tags)        (:wat-telemetry::empty-tags))
     ((ns    :wat::holon::HolonAST)        (:wat::holon::Atom :my::ns))
     ((name  :wat::holon::HolonAST)        (:wat::holon::Atom :sql-page))
     ((event :wat::telemetry::Event)
      (:wat::telemetry::WorkUnit/scope::build-duration-metric
        300 400 ns "dur-uuid" tags name 0.5)))
    (:wat::core::match event -> :()
      ((:wat::telemetry::Event::Metric s e _ uuid _ _ _ _)
        (:wat::core::let*
          (((_a :()) (:wat::test::assert-eq s 300))
           ((_b :()) (:wat::test::assert-eq e 400)))
          (:wat::test::assert-eq uuid "dur-uuid")))
      ((:wat::telemetry::Event::Log _ _ _ _ _ _ _)
        (:wat::test::assert-eq "expected-Metric-variant" "got-Log-instead")))))


;; collect-metric-events — at scope-close, walk the wu's counters
;; AND durations into a flat `Vec<Event>` (Metric variants only;
;; Logs ship per-emission, not at scope-close). Empty wu produces
;; empty Vec — the simplest contract case. Subsequent tests add
;; one counter, one duration-sample, then mixed.
(:deftest :wat-telemetry::WorkUnit::test-collect-metrics-empty
  (:wat::core::let*
    (((tags   :wat::telemetry::Tags)         (:wat-telemetry::empty-tags))
     ((wu     :wat::telemetry::WorkUnit)     (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) tags))
     ((ns     :wat::holon::HolonAST)         (:wat::holon::Atom :test::ns))
     ((events :Vec<wat::telemetry::Event>)
      (:wat::telemetry::WorkUnit/scope::collect-metric-events
        wu 100 200)))
    (:wat::test::assert-eq (:wat::core::length events) 0)))


;; One counter incremented thrice → ONE Event::Metric row in the
;; Vec (CloudWatch model — counters emit one row per name with
;; the final count, not one per increment).
(:deftest :wat-telemetry::WorkUnit::test-collect-metrics-one-counter
  (:wat::core::let*
    (((tags  :wat::telemetry::Tags)     (:wat-telemetry::empty-tags))
     ((wu    :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) tags))
     ((name  :wat::holon::HolonAST)     (:wat::holon::Atom :requests))
     ((_a    :())                        (:wat::telemetry::WorkUnit/incr! wu name))
     ((_b    :())                        (:wat::telemetry::WorkUnit/incr! wu name))
     ((_c    :())                        (:wat::telemetry::WorkUnit/incr! wu name))
     ((ns    :wat::holon::HolonAST)     (:wat::holon::Atom :test::ns))
     ((events :Vec<wat::telemetry::Event>)
      (:wat::telemetry::WorkUnit/scope::collect-metric-events
        wu 100 200)))
    (:wat::test::assert-eq (:wat::core::length events) 1)))


;; One duration name with TWO samples → TWO Event::Metric rows
;; (CloudWatch fanout). Same name, same start/end/uuid/tags/ns;
;; different metric-value per row.
(:deftest :wat-telemetry::WorkUnit::test-collect-metrics-two-duration-samples
  (:wat::core::let*
    (((tags  :wat::telemetry::Tags)     (:wat-telemetry::empty-tags))
     ((wu    :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::default-ns) tags))
     ((name  :wat::holon::HolonAST)     (:wat::holon::Atom :sql-page))
     ((_a    :())                        (:wat::telemetry::WorkUnit/append-dt! wu name 0.5))
     ((_b    :())                        (:wat::telemetry::WorkUnit/append-dt! wu name 1.5))
     ((ns    :wat::holon::HolonAST)     (:wat::holon::Atom :test::ns))
     ((events :Vec<wat::telemetry::Event>)
      (:wat::telemetry::WorkUnit/scope::collect-metric-events
        wu 100 200)))
    (:wat::test::assert-eq (:wat::core::length events) 2)))


;; ─── Probe: can wat express `fn(X) -> fn(Y) -> Z` returns? ──────
;;
;; Arc 083 rejected a nested-fn shape on grounds it had "no other
;; precedent in wat". Before the rank-2 closure factory for
;; WorkUnit/make-scope, this probe verifies the basic shape:
;; a function whose return type IS a function type, returned as a
;; lambda value.
(:deftest :wat-telemetry::WorkUnit::probe-fn-returning-fn
  (:wat::core::let*
    (((adder :fn(i64)->i64)
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
    (((runner :fn(fn()->i64)->i64)
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
    ;; Stub queue — collects the Events the dispatcher sees.
    (((stub-pair :wat::kernel::QueuePair<wat::telemetry::Event>)
      (:wat::kernel::make-bounded-queue :wat::telemetry::Event 16))
     ((stub-tx :wat::kernel::QueueSender<wat::telemetry::Event>)
      (:wat::core::first stub-pair))
     ((stub-rx :wat::kernel::QueueReceiver<wat::telemetry::Event>)
      (:wat::core::second stub-pair))
     ;; Dispatcher closure-over stub-tx; null cadence + empty translator.
     ((dispatcher :fn(Vec<wat::telemetry::Event>)->())
      (:wat-telemetry::scope::make-stub-dispatcher stub-tx))
     ((cadence :wat::telemetry::Service::MetricsCadence<()>)
      (:wat::telemetry::Service/null-metrics-cadence))
     ;; Spawn Service<Event,_> with one client slot.
     ((spawn :wat::telemetry::Service::Spawn<wat::telemetry::Event>)
      (:wat::telemetry::Service/spawn 1 cadence dispatcher
        :wat-telemetry::scope::translate-empty))
     ((pool :wat::telemetry::Service::HandlePool<wat::telemetry::Event>)
      (:wat::core::first spawn))
     ((driver :wat::kernel::ProgramHandle<()>) (:wat::core::second spawn))
     ;; Inner: pop Handle, finish pool, factory + scope-fn-with-counter.
     ((result :wat::core::i64)
      (:wat::core::let*
        (((handle :wat::telemetry::Service::Handle<wat::telemetry::Event>)
          (:wat::kernel::HandlePool::pop pool))
         ((_finish :()) (:wat::kernel::HandlePool::finish pool))
         ((ns :wat::holon::HolonAST) (:wat-telemetry::default-ns))
         ((scope :wat::telemetry::WorkUnit::Scope<i64>)
          (:wat::telemetry::WorkUnit/make-scope handle ns))
         ((tags :wat::telemetry::Tags) (:wat-telemetry::empty-tags)))
        (scope tags
          (:wat::core::lambda
            ((wu :wat::telemetry::WorkUnit) -> :wat::core::i64)
            (:wat::core::let*
              (((_ :()) (:wat::telemetry::WorkUnit/incr! wu (:wat::holon::Atom :hits))))
              42)))))
     ((_join :()) (:wat::kernel::join driver))
     ;; Drain — recv ONE Event for the single counter (CloudWatch
     ;; model: one counter = one row, established by
     ;; test-collect-metrics-one-counter). recv'ing past the
     ;; expected count would block on stub-tx — still alive in
     ;; this scope until the let* body terminates — so we recv
     ;; only what we KNOW was sent. Per the test's intent:
     ;; "scope ships the Events" is proven by recv returning
     ;; Some at all; the row's CONTENT is proven elsewhere.
     ((r1-some? :wat::core::bool)
      (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::bool ((Some _) true) (:None false)))
     ((_a :()) (:wat::test::assert-eq result 42)))
    (:wat::test::assert-eq r1-some? true)))
