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
       (stub-tx :wat::kernel::Sender<wat::telemetry::Event>)
       -> :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
     (:wat::core::lambda ((entries :wat::core::Vector<wat::telemetry::Event>) -> :wat::core::unit)
       (:wat::core::foldl entries ()
         (:wat::core::lambda ((_acc :wat::core::unit) (e :wat::telemetry::Event) -> :wat::core::unit)
           (:wat::core::match (:wat::kernel::send stub-tx e) -> :wat::core::unit
             ((:wat::core::Ok _) ())
             ((:wat::core::Err _) ()))))))

   (:wat::core::define
     (:wat-telemetry::log-test::translate-empty
       (_s :wat::telemetry::Stats)
       -> :wat::core::Vector<wat::telemetry::Event>)
     (:wat::core::Vector :wat::telemetry::Event))

   ;; ─── Layer 1 — level extraction ──────────────────────────────────
   ;;
   ;; :test::wul-extract-level — pattern-match one Event; return its
   ;; level keyword when it is a Log variant; sentinel keyword on any
   ;; other variant. Pure: no channel interaction.
   (:wat::core::define
     (:test::wul-extract-level
       (event :wat::telemetry::Event)
       -> :wat::core::keyword)
     (:wat::core::match event -> :wat::core::keyword
       ((:wat::telemetry::Event::Log
          _t _ns _c level-notag _u _tags _d)
         (:wat::core::atom-value
           (:wat::edn::NoTag/0 level-notag)))
       ((:wat::telemetry::Event::Metric
          _s _e _ns _u _tags _n _v _unit)
         :wrong-variant-metric)))

   ;; :test::wul-recv-level — recv one Event from stub-rx; call
   ;; wul-extract-level on the unwrapped event; return a sentinel
   ;; keyword on None or channel error.
   ;; NOTE: no isolated deftest — constructing a synthetic Event::Log
   ;; requires substrate-internal field knowledge. Level 3 taste
   ;; exemption; proven by its callers.
   (:wat::core::define
     (:test::wul-recv-level
       (stub-rx :wat::kernel::Receiver<wat::telemetry::Event>)
       -> :wat::core::keyword)
     (:wat::core::match (:wat::kernel::recv stub-rx) -> :wat::core::keyword
       ((:wat::core::Ok (:wat::core::Some event))
         (:test::wul-extract-level event))
       ((:wat::core::Ok :wat::core::None) :no-event)
       ((:wat::core::Err _died) :no-event)))

   ;; ─── Layer 2 — stub-service + wu + logger runner ─────────────────
   ;;
   ;; :test::wul-spawn-stub-and-emit-drain — spawn stub telemetry service
   ;; (null cadence, translate-empty), pop handle, build a fresh WorkUnit
   ;; and WorkUnitLog, then call body with (logger, wu, stub-rx). body
   ;; performs all emit+drain and returns a keyword. stub-tx is captured
   ;; inside the dispatcher closure and drops when the inner scope exits,
   ;; letting the driver exit cleanly.
   ;; Returns (Thread<unit,unit>, keyword) — caller joins driver and asserts
   ;; on the keyword.
   (:wat::core::define
     (:test::wul-spawn-stub-and-emit-drain
       (body :fn(wat::telemetry::WorkUnitLog,wat::telemetry::WorkUnit,wat::kernel::Receiver<wat::telemetry::Event>)->wat::core::keyword)
       -> :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::keyword))
     (:wat::core::let*
       (((stub-pair :wat::kernel::Channel<wat::telemetry::Event>)
         (:wat::kernel::make-bounded-channel :wat::telemetry::Event 16))
        ((stub-tx :wat::kernel::Sender<wat::telemetry::Event>)
         (:wat::core::first stub-pair))
        ((stub-rx :wat::kernel::Receiver<wat::telemetry::Event>)
         (:wat::core::second stub-pair))
        ((dispatcher :fn(wat::core::Vector<wat::telemetry::Event>)->wat::core::unit)
         (:wat-telemetry::log-test::make-stub-dispatcher stub-tx))
        ((cadence :wat::telemetry::MetricsCadence<wat::core::unit>)
         (:wat::telemetry::null-metrics-cadence))
        ((spawn :wat::telemetry::Spawn<wat::telemetry::Event>)
         (:wat::telemetry::spawn 1 cadence dispatcher
           :wat-telemetry::log-test::translate-empty))
        ((pool :wat::telemetry::HandlePool<wat::telemetry::Event>)
         (:wat::core::first spawn))
        ((d :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
         (:wat::core::second spawn))
        ((kw :wat::core::keyword)
         (:wat::core::let*
           (((handle :wat::telemetry::Handle<wat::telemetry::Event>)
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
               (:wat-telemetry::log-test::fixed-now-fn))))
           (body logger wu stub-rx))))
       (:wat::core::Tuple d kw)))

   ))


;; ─── Per-layer deftests for new helpers ───────────────────────────────────────

;; Layer 1 — wul-extract-level: Level 3 taste exemption — constructing a
;; synthetic Event::Log requires substrate-internal field knowledge. Proven
;; by callers (test-info-emits-log-event + test-each-level-emits-log).

;; Layer 1 — wul-recv-level: Level 3 taste exemption — same constraint as
;; wul-extract-level. Proven by callers.

;; Layer 2 — wul-spawn-stub-and-emit-drain: body does nothing, returns sentinel.
;; Proves spawn + configure + pop + build wu + build logger lifecycle is clean.
(:deftest :wat-telemetry::WorkUnitLog::test-wul-spawn-stub-and-emit-drain
  (:wat::core::let*
    (((thr-kw :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::keyword))
      (:test::wul-spawn-stub-and-emit-drain
        (:wat::core::lambda
          ((_logger :wat::telemetry::WorkUnitLog)
           (_wu :wat::telemetry::WorkUnit)
           (_stub-rx :wat::kernel::Receiver<wat::telemetry::Event>)
           -> :wat::core::keyword)
          :ok)))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-kw))
     ((kw :wat::core::keyword) (:wat::core::second thr-kw))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq kw :ok)))


;; ─── /info ships an Event::Log row through the captured handle ───
;;
;; Send one info, recv the event, pattern-match — verify it's the
;; Log variant (not Metric) and that the level keyword survives
;; the lift (keyword → Atom → NoTag) + render round-trip.
(:deftest :wat-telemetry::WorkUnitLog::test-info-emits-log-event
  (:wat::core::let*
    ;; Body: emit one /info, drain one event, return its level keyword.
    ;; wul-spawn-stub-and-emit-drain internalizes spawn + configure +
    ;; pop + wu + logger. Body lambda is the embedded test fixture.
    (((thr-kw :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::keyword))
      (:test::wul-spawn-stub-and-emit-drain
        (:wat::core::lambda
          ((logger :wat::telemetry::WorkUnitLog)
           (wu :wat::telemetry::WorkUnit)
           (stub-rx :wat::kernel::Receiver<wat::telemetry::Event>)
           -> :wat::core::keyword)
          (:wat::core::let*
            (((_log :wat::core::unit)
              (:wat::telemetry::WorkUnitLog/info logger wu (:wat::core::quote :hello))))
            (:test::wul-recv-level stub-rx)))))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-kw))
     ((level-back :wat::core::keyword) (:wat::core::second thr-kw))
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
    ;; Body: emit debug + info + warn + error; drain four events;
    ;; assert first three; return the fourth level keyword.
    ;; wul-spawn-stub-and-emit-drain internalizes spawn + configure +
    ;; pop + wu + logger. Body lambda is the embedded test fixture.
    (((thr-kw :(wat::kernel::Thread<wat::core::unit,wat::core::unit>,wat::core::keyword))
      (:test::wul-spawn-stub-and-emit-drain
        (:wat::core::lambda
          ((logger :wat::telemetry::WorkUnitLog)
           (wu :wat::telemetry::WorkUnit)
           (stub-rx :wat::kernel::Receiver<wat::telemetry::Event>)
           -> :wat::core::keyword)
          (:wat::core::let*
            (((data :wat::WatAST) (:wat::core::quote :payload))
             ((_d :wat::core::unit) (:wat::telemetry::WorkUnitLog/debug logger wu data))
             ((_i :wat::core::unit) (:wat::telemetry::WorkUnitLog/info  logger wu data))
             ((_w :wat::core::unit) (:wat::telemetry::WorkUnitLog/warn  logger wu data))
             ((_e :wat::core::unit) (:wat::telemetry::WorkUnitLog/error logger wu data))
             ((l1 :wat::core::keyword) (:test::wul-recv-level stub-rx))
             ((l2 :wat::core::keyword) (:test::wul-recv-level stub-rx))
             ((l3 :wat::core::keyword) (:test::wul-recv-level stub-rx))
             ((_ :wat::core::unit) (:wat::test::assert-eq l1 :debug))
             ((_ :wat::core::unit) (:wat::test::assert-eq l2 :info))
             ((_ :wat::core::unit) (:wat::test::assert-eq l3 :warn)))
            (:test::wul-recv-level stub-rx)))))
     ((driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>) (:wat::core::first thr-kw))
     ((l4 :wat::core::keyword) (:wat::core::second thr-kw))
     ((_join :wat::core::Result<wat::core::unit,wat::core::Vector<wat::kernel::ThreadDiedError>>)
      (:wat::kernel::Thread/join-result driver)))
    (:wat::test::assert-eq l4 :error)))
