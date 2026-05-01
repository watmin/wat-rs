;; wat-tests/telemetry/WorkUnitLog.wat — arc 091 slice 5 smoke tests.
;;
;; The Log emission surface. Each /log (or /debug, /info, /warn,
;; /error sugar) builds one Event::Log row, ships it through the
;; logger's captured Service handle as a single-element batch, and
;; blocks on ack. One emission = one synchronous round-trip; same
;; shape as ConsoleLogger (arc 087) and the lab archive's
;; `DatabaseHandle.send(entry)` (pre-wat-native/src/programs/stdlib/
;; database.rs:60).
;;
;; Each test:
;;   - spawns a stub-forwarding Service<Event,_>
;;   - pops one Handle
;;   - builds a fresh WorkUnit (namespace + empty tags)
;;   - builds a WorkUnitLog (handle, caller, now-fn)
;;   - calls /info (or /debug, /warn, /error) with a HolonAST data payload
;;   - drains ONE event off the stub queue and inspects it
;;
;; recv only what we KNOW was sent — outer let* still holds stub-tx
;; until the body terminates, so an over-recv would block (the
;; gotcha called out in SERVICE-PROGRAMS.md).

(:wat::test::make-deftest :deftest
  ((:wat::core::define
     (:wat-telemetry::log-test::empty-tags -> :wat::telemetry::Tags)
     (:wat::core::HashMap :wat::telemetry::Tag))

   (:wat::core::define
     (:wat-telemetry::log-test::default-ns -> :wat::holon::HolonAST)
     (:wat::holon::Atom :wat-telemetry::log-test::ns))

   (:wat::core::define
     (:wat-telemetry::log-test::default-caller -> :wat::core::keyword)
     :wat-telemetry::log-test::caller)

   ;; Clock injection — fixed instant so emissions are deterministic
   ;; in tests. Real producers pass `(:wat::time::now)` (closure form
   ;; mirrors arc 087's ConsoleLogger).
   (:wat::core::define
     (:wat-telemetry::log-test::fixed-now-fn
       -> :fn(wat::core::unit)->wat::time::Instant)
     (:wat::core::lambda ((_u :wat::core::unit) -> :wat::time::Instant)
       (:wat::time::now)))

   ;; Stub dispatcher — same shape as the make-scope tests'.
   (:wat::core::define
     (:wat-telemetry::log-test::make-stub-dispatcher
       (stub-tx :wat::kernel::QueueSender<wat::telemetry::Event>)
       -> :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
     (:wat::core::lambda ((entries :wat::core::Vector<wat::telemetry::Event>) -> :wat::core::unit)
       (:wat::core::foldl entries ()
         (:wat::core::lambda ((_acc :wat::core::unit) (e :wat::telemetry::Event) -> :wat::core::unit)
           (:wat::core::match (:wat::kernel::send stub-tx e) -> :wat::core::unit
             ((Ok _) ())
             ((Err _) ()))))))

   (:wat::core::define
     (:wat-telemetry::log-test::translate-empty
       (_s :wat::telemetry::Service::Stats)
       -> :wat::core::Vector<wat::telemetry::Event>)
     (:wat::core::Vector :wat::telemetry::Event))))


;; ─── /info ships an Event::Log row through the captured handle ───
;;
;; Send one info, recv the event, pattern-match — verify it's the
;; Log variant (not Metric) and that the level keyword survives
;; the lift (keyword → Atom → NoTag) + render round-trip.
(:deftest :wat-telemetry::WorkUnitLog::test-info-emits-log-event
  (:wat::core::let*
    ;; Inner owns every QueueSender clone (stub-pair, stub-tx) AND
    ;; emits + drains the one /info event before returning. Returns
    ;; (driver, level-back) so outer can join the driver and assert on
    ;; the level keyword. SERVICE-PROGRAMS.md § "The lockstep" + arc 117.
    (((thr-and-level :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::keyword))
      (:wat::core::let*
        (((stub-pair :wat::kernel::QueuePair<wat::telemetry::Event>)
          (:wat::kernel::make-bounded-queue :wat::telemetry::Event 16))
         ((stub-tx :wat::kernel::QueueSender<wat::telemetry::Event>)
          (:wat::core::first stub-pair))
         ((stub-rx :wat::kernel::QueueReceiver<wat::telemetry::Event>)
          (:wat::core::second stub-pair))
         ((dispatcher :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
          (:wat-telemetry::log-test::make-stub-dispatcher stub-tx))
         ((cadence :wat::telemetry::Service::MetricsCadence<wat::core::unit>)
          (:wat::telemetry::Service/null-metrics-cadence))
         ((spawn :wat::telemetry::Service::Spawn<wat::telemetry::Event>)
          (:wat::telemetry::Service/spawn 1 cadence dispatcher
            :wat-telemetry::log-test::translate-empty))
         ((pool :wat::telemetry::Service::HandlePool<wat::telemetry::Event>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ;; Inner-inner: pop handle, build wu + logger, emit one /info.
         ((_inner :wat::core::unit)
          (:wat::core::let*
            (((handle :wat::telemetry::Service::Handle<wat::telemetry::Event>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
             ((wu :wat::telemetry::WorkUnit)
              (:wat::telemetry::WorkUnit::new
                (:wat-telemetry::log-test::default-ns)
                (:wat-telemetry::log-test::empty-tags)))
             ((logger :wat::telemetry::WorkUnitLog)
              (:wat::telemetry::WorkUnitLog/new
                handle
                (:wat-telemetry::log-test::default-caller)
                (:wat-telemetry::log-test::fixed-now-fn)))
             ((_log :wat::core::unit)
              (:wat::telemetry::WorkUnitLog/info logger wu (:wat::core::quote :hello))))
            ()))
         ;; Drain the one event in the same scope (stub-tx still alive,
         ;; but the row is already enqueued by the synchronous /info
         ;; round-trip). Match-at-source per arc 110.
         ((level-back :wat::core::keyword)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::keyword
            ((Ok (:wat::core::Some event))
              (:wat::core::match event -> :wat::core::keyword
                ((:wat::telemetry::Event::Log
                   _t _ns _c level-notag _u _tags _d)
                  (:wat::core::atom-value
                    (:wat::edn::NoTag/0 level-notag)))
                ((:wat::telemetry::Event::Metric
                   _s _e _ns _u _tags _n _v _unit)
                  :wrong-variant-metric)))
            ((Ok :wat::core::None) :no-event)
            ((Err _died) :no-event))))
        (:wat::core::Tuple d level-back)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-and-level))
     ((level-back :wat::core::keyword) (:wat::core::second thr-and-level))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq level-back :info)))


;; ─── Each level keyword surfaces on its own row ─────────────────
;;
;; Emit one of each level; recv four; verify each level keyword
;; round-trips. Order is preserved (single-element batches sequenced
;; through one ack channel) — but the assertion doesn't depend on
;; order; we extract each level and check the SET of levels seen.

(:deftest :wat-telemetry::WorkUnitLog::test-each-level-emits-log
  (:wat::core::let*
    ;; Inner owns every QueueSender clone (stub-pair, stub-tx) AND
    ;; emits the four /level calls and drains all four events in the
    ;; same scope (the synchronous round-trip means each row is
    ;; enqueued by the time recv runs). Returns (driver, l4) so outer
    ;; joins the driver and asserts the last level keyword.
    ;; SERVICE-PROGRAMS.md § "The lockstep" + arc 117.
    (((thr-and-l4 :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::keyword))
      (:wat::core::let*
        (((stub-pair :wat::kernel::QueuePair<wat::telemetry::Event>)
          (:wat::kernel::make-bounded-queue :wat::telemetry::Event 16))
         ((stub-tx :wat::kernel::QueueSender<wat::telemetry::Event>)
          (:wat::core::first stub-pair))
         ((stub-rx :wat::kernel::QueueReceiver<wat::telemetry::Event>)
          (:wat::core::second stub-pair))
         ((dispatcher :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
          (:wat-telemetry::log-test::make-stub-dispatcher stub-tx))
         ((cadence :wat::telemetry::Service::MetricsCadence<wat::core::unit>)
          (:wat::telemetry::Service/null-metrics-cadence))
         ((spawn :wat::telemetry::Service::Spawn<wat::telemetry::Event>)
          (:wat::telemetry::Service/spawn 1 cadence dispatcher
            :wat-telemetry::log-test::translate-empty))
         ((pool :wat::telemetry::Service::HandlePool<wat::telemetry::Event>)
          (:wat::core::first spawn))
         ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::second spawn))
         ((_inner :wat::core::unit)
          (:wat::core::let*
            (((handle :wat::telemetry::Service::Handle<wat::telemetry::Event>)
              (:wat::kernel::HandlePool::pop pool))
             ((_finish :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
             ((wu :wat::telemetry::WorkUnit)
              (:wat::telemetry::WorkUnit::new
                (:wat-telemetry::log-test::default-ns)
                (:wat-telemetry::log-test::empty-tags)))
             ((logger :wat::telemetry::WorkUnitLog)
              (:wat::telemetry::WorkUnitLog/new
                handle
                (:wat-telemetry::log-test::default-caller)
                (:wat-telemetry::log-test::fixed-now-fn)))
             ((data :wat::WatAST) (:wat::core::quote :payload))
             ((_d :wat::core::unit) (:wat::telemetry::WorkUnitLog/debug logger wu data))
             ((_i :wat::core::unit) (:wat::telemetry::WorkUnitLog/info  logger wu data))
             ((_w :wat::core::unit) (:wat::telemetry::WorkUnitLog/warn  logger wu data))
             ((_e :wat::core::unit) (:wat::telemetry::WorkUnitLog/error logger wu data)))
            ()))
         ;; Arc 110: extract-level takes the unwrapped Event; the
         ;; match-at-source on recv at each call site supplies the
         ;; :None default. recv can no longer hide inside a function-arg.
         ((extract-level :fn(wat::telemetry::Event)->wat::core::keyword)
          (:wat::core::lambda
            ((event :wat::telemetry::Event) -> :wat::core::keyword)
            (:wat::core::match event -> :wat::core::keyword
              ((:wat::telemetry::Event::Log
                 _t _ns _c level-notag _u _tags _d)
                (:wat::core::atom-value
                  (:wat::edn::NoTag/0 level-notag)))
              ((:wat::telemetry::Event::Metric
                 _s _e _ns _u _tags _n _v _unit)
                :wrong-variant-metric))))
         ((l1 :wat::core::keyword)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::keyword
            ((Ok (:wat::core::Some event)) (extract-level event))
            ((Ok :wat::core::None) :no-event)
            ((Err _died) :no-event)))
         ((l2 :wat::core::keyword)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::keyword
            ((Ok (:wat::core::Some event)) (extract-level event))
            ((Ok :wat::core::None) :no-event)
            ((Err _died) :no-event)))
         ((l3 :wat::core::keyword)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::keyword
            ((Ok (:wat::core::Some event)) (extract-level event))
            ((Ok :wat::core::None) :no-event)
            ((Err _died) :no-event)))
         ((l4 :wat::core::keyword)
          (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::keyword
            ((Ok (:wat::core::Some event)) (extract-level event))
            ((Ok :wat::core::None) :no-event)
            ((Err _died) :no-event)))
         ((_a :wat::core::unit) (:wat::test::assert-eq l1 :debug))
         ((_b :wat::core::unit) (:wat::test::assert-eq l2 :info))
         ((_c :wat::core::unit) (:wat::test::assert-eq l3 :warn)))
        (:wat::core::Tuple d l4)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-and-l4))
     ((l4 :wat::core::keyword) (:wat::core::second thr-and-l4))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq l4 :error)))
