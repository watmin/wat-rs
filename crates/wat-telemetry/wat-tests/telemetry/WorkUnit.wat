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
     (:wat::core::HashMap :wat::telemetry::Tag))))


;; ─── uuid is non-empty ────────────────────────────────────────────

(:deftest :wat-telemetry::WorkUnit::test-uuid-non-empty
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::empty-tags)))
     ((id :String) (:wat::telemetry::WorkUnit/uuid wu)))
    ;; A canonical 8-4-4-4-12 hex uuid is 36 chars — but :String
    ;; has no length primitive in slice-3 wat surface, and the
    ;; rigorous format checks live in arc 092's Rust tests. Here
    ;; we just prove the read returns SOME string — the empty
    ;; sentinel "" would equal "" so the assertion would catch
    ;; a degenerate shim that returned the empty string.
    (:wat::test::assert-eq (:wat::core::= id "") false)))


;; ─── uuids are distinct across new() calls ───────────────────────

(:deftest :wat-telemetry::WorkUnit::test-uuid-distinct
  (:wat::core::let*
    (((wu1 :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::empty-tags)))
     ((wu2 :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::empty-tags)))
     ((id1 :String) (:wat::telemetry::WorkUnit/uuid wu1))
     ((id2 :String) (:wat::telemetry::WorkUnit/uuid wu2)))
    (:wat::test::assert-eq (:wat::core::= id1 id2) false)))


;; ─── counter on an absent key returns 0 ──────────────────────────

(:deftest :wat-telemetry::WorkUnit::test-counter-default
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :never-incremented))
     ((n :i64) (:wat::telemetry::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 0)))


;; ─── incr! then counter — single bump ────────────────────────────

(:deftest :wat-telemetry::WorkUnit::test-incr-then-counter
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :requests))
     ((_ :()) (:wat::telemetry::WorkUnit/incr! wu name))
     ((n :i64) (:wat::telemetry::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 1)))


;; ─── incr! many — accumulation across calls ──────────────────────

(:deftest :wat-telemetry::WorkUnit::test-incr-many
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :requests))
     ((_a :()) (:wat::telemetry::WorkUnit/incr! wu name))
     ((_b :()) (:wat::telemetry::WorkUnit/incr! wu name))
     ((_c :()) (:wat::telemetry::WorkUnit/incr! wu name))
     ((n :i64) (:wat::telemetry::WorkUnit/counter wu name)))
    (:wat::test::assert-eq n 3)))


;; ─── append-dt! then read ────────────────────────────────────────

(:deftest :wat-telemetry::WorkUnit::test-append-dt-then-read
  (:wat::core::let*
    (((wu :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new (:wat-telemetry::empty-tags)))
     ((name :wat::holon::HolonAST) (:wat::holon::Atom :sql-page))
     ((_a :()) (:wat::telemetry::WorkUnit/append-dt! wu name 0.5))
     ((_b :()) (:wat::telemetry::WorkUnit/append-dt! wu name 1.5))
     ((dts :Vec<f64>) (:wat::telemetry::WorkUnit/durations wu name)))
    (:wat::test::assert-eq dts (:wat::core::vec :f64 0.5 1.5))))


;; ─── Tags — immutable, declared at construction ───────────────

;; Empty tags map round-trips through the constructor.
(:deftest :wat-telemetry::WorkUnit::test-tags-empty
  (:wat::core::let*
    (((empty :wat::telemetry::Tags) (:wat-telemetry::empty-tags))
     ((wu  :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new empty))
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
     ((wu    :wat::telemetry::WorkUnit) (:wat::telemetry::WorkUnit::new tags))
     ((got   :wat::telemetry::Tags)
      (:wat::telemetry::WorkUnit/tags wu))
     ((looked-up :Option<wat::holon::HolonAST>)
      (:wat::core::get got asset-key)))
    (:wat::test::assert-eq looked-up (Some asset-val))))


;; ─── WorkUnit/scope<T> — bare HOF (open + run + return) ──────────

;; Body sees the wu, mutates it, returns T; scope returns body's
;; T. The bare scope (no auto-ship yet — that lands in slice
;; 4-ship) is the smallest piece of the HOF contract.
(:deftest :wat-telemetry::WorkUnit::test-scope-passes-result
  (:wat::core::let*
    (((tags   :wat::telemetry::Tags) (:wat-telemetry::empty-tags))
     ((result :i64)
      (:wat::telemetry::WorkUnit/scope tags
        (:wat::core::lambda ((wu :wat::telemetry::WorkUnit) -> :i64)
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
