;; wat-tests/std/telemetry/Console.wat — arc 081 smoke tests.
;;
;; Arc 130 — complectēns rewrite. Top-down dependency graph in ONE file.
;;
;; ─── Layers ──────────────────────────────────────────────────────────
;;
;;   Layer 1  :test::tel-stdout-from-result
;;              ; extract stdout Vector<String> from a RunResult
;;
;;   Layer 2  :test::tel-assert-line-once
;;              ; assert exactly one occurrence of msg in stdout
;;
;; Inner programs embedded in run-hermetic-ast run in a subprocess and
;; cannot reference prelude helpers (separate freeze). Helpers apply to
;; the OUTER test logic: extracting stdout, asserting line membership.
;;
;; No arc-126 concern: outer test code has no make-bounded-channel
;; allocations (the inner program manages Console's channel pairs
;; internally; those are opaque to the outer test body).

(:wat::test::make-deftest :deftest
  (
   ;; ─── Layer 1 — RunResult accessor ────────────────────────────
   ;;
   ;; :test::tel-stdout-from-result — extract stdout vector from a
   ;; RunResult. Named wrapper for readable outer composition.
   (:wat::core::define
     (:test::tel-stdout-from-result
       (r :wat::kernel::RunResult)
       -> :wat::core::Vector<wat::core::String>)
     (:wat::kernel::RunResult/stdout r))


   ;; ─── Layer 2 — assertion helper ──────────────────────────────
   ;;
   ;; :test::tel-assert-line-once — assert that stdout contains exactly
   ;; one element equal to msg. Fails with assert-eq if absent.
   (:wat::core::define
     (:test::tel-assert-line-once
       (stdout :wat::core::Vector<wat::core::String>)
       (msg :wat::core::String)
       -> :wat::core::unit)
     (:wat::core::if
       (:wat::core::=
         (:wat::core::length
           (:wat::core::filter stdout
             (:wat::core::lambda ((s :wat::core::String) -> :wat::core::bool)
               (:wat::core::= s msg))))
         1)
       -> :wat::core::unit
       ()
       (:wat::test::assert-eq msg "not found exactly once in stdout")))

   ))


;; ─── Per-layer deftests ────────────────────────────────────────────────────
;;
;; Prove helpers in isolation. stdout-from-result is a thin accessor
;; proven implicitly by the scenario deftests (constructing a RunResult
;; in isolation requires hermetic infrastructure — Level 3 taste gap).

;; Layer 2 — assert-line-once: passes when msg is present exactly once.
(:deftest :wat-telemetry::Console::test-assert-line-once-pass
  (:wat::core::let*
    (((stdout :wat::core::Vector<wat::core::String>)
      (:wat::core::conj
        (:wat::core::conj
          (:wat::core::Vector :wat::core::String)
          "10")
        "20")))
    (:test::tel-assert-line-once stdout "10")))


;; ─── Test 1: EDN format renders i64 entries as bare integers ──────────────
;;
;; Dispatcher dispatches three i64 entries as ONE batch via EDN format.
;; Verifies that 10, 20, and 30 each appear exactly once in stdout.

(:deftest :wat-telemetry::Console::test-dispatcher-edn
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          ;; App-level concrete typealias collapses the substrate's
          ;; generic Dispatcher<E> to a single name for THIS app's
          ;; entry type. Same pattern as the lab's
          ;; `:trading::telemetry::Spawn` alias collapsing
          ;; `Service::Spawn<trading::log::LogEntry>` — substrate
          ;; ships generic shapes, apps alias them concrete.
          (:wat::core::typealias :my::Dispatcher
            :wat::telemetry::Console::Dispatcher<wat::core::i64>)

          ;; Helper — takes a Console::Handle, builds an EDN
          ;; dispatcher, dispatches three i64 entries as ONE batch.
          ;; Arc 089 slice 3: dispatcher takes wat::core::Vector<E>.
          ;; Arc 089 slice 5: dispatcher closes over a Console::Handle
          ;; so the per-entry Console/out call gets in-memory TCP for free.
          (:wat::core::define
            (:my::dispatch-three-edn
              (handle :wat::console::Handle)
              -> :wat::core::unit)
            (:wat::core::let*
              (((d :my::Dispatcher)
                (:wat::telemetry::Console/dispatcher
                  handle :wat::telemetry::Console::Format::Edn))
               ((batch :wat::core::Vector<wat::core::i64>) (:wat::core::Vector :wat::core::i64 10 20 30)))
              (d batch)))
          ;; Main — outer holds Console driver Thread; inner owns the
          ;; spawn-tuple, pool, handle, and dispatch work; inner returns
          ;; the Thread; pool drops at inner exit; outer joins.
          ;; SERVICE-PROGRAMS.md § "The lockstep" + arc 117 + arc 131.
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::core::let*
              (((console-driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                (:wat::core::let*
                  (((spawn :wat::console::Spawn)
                    (:wat::console::spawn stdout stderr 1))
                   ((pool :wat::kernel::HandlePool<wat::console::Handle>)
                    (:wat::core::first spawn))
                   ((cd :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                    (:wat::core::second spawn))
                   ((handle :wat::console::Handle)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_0 :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
                   ((_work :wat::core::unit) (:my::dispatch-three-edn handle)))
                  cd)))
              (:wat::core::match (:wat::kernel::Thread/join-result console-driver) -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::test::assert-eq "console-driver-died" ""))))))
        (:wat::core::Vector :wat::core::String)))
     ((stdout :wat::core::Vector<wat::core::String>) (:test::tel-stdout-from-result r))
     ((_ :wat::core::unit) (:test::tel-assert-line-once stdout "10"))
     ((_ :wat::core::unit) (:test::tel-assert-line-once stdout "20")))
    (:test::tel-assert-line-once stdout "30")))


;; ─── Test 2: JSON format renders wat::core::Vector<i64> as JSON array ──────────

(:deftest :wat-telemetry::Console::test-dispatcher-json
  (:wat::core::let*
    (((r :wat::kernel::RunResult)
      (:wat::test::run-hermetic-ast
        (:wat::test::program
          ;; App-level concrete aliases. Two layers — Row is the
          ;; entry shape; Dispatcher is the dispatcher's concrete
          ;; type. Every signature site reads `:my::Dispatcher`
          ;; instead of `:fn(wat::core::Vector<wat::core::Vector<i64>>)->()` or
          ;; `:wat::telemetry::Console::Dispatcher<wat::core::Vector<i64>>`.
          (:wat::core::typealias :my::Row :wat::core::Vector<wat::core::i64>)
          (:wat::core::typealias :my::Dispatcher
            :wat::telemetry::Console::Dispatcher<my::Row>)

          ;; Arc 089 slice 3: dispatcher takes wat::core::Vector<E>. The
          ;; dispatcher renders each element on its own line —
          ;; one batch with one Row entry → one line "[1,2,3]".
          (:wat::core::define
            (:my::dispatch-row-json
              (handle :wat::console::Handle)
              -> :wat::core::unit)
            (:wat::core::let*
              (((d :my::Dispatcher)
                (:wat::telemetry::Console/dispatcher
                  handle :wat::telemetry::Console::Format::Json))
               ((row :my::Row) (:wat::core::Vector :wat::core::i64 1 2 3))
               ((batch :wat::core::Vector<my::Row>)
                (:wat::core::Vector :my::Row row)))
              (d batch)))
          (:wat::core::define
            (:user::main
              (stdin  :wat::io::IOReader)
              (stdout :wat::io::IOWriter)
              (stderr :wat::io::IOWriter)
              -> :wat::core::unit)
            (:wat::core::let*
              ;; Outer holds Console driver Thread; inner owns the
              ;; spawn-tuple, pool, handle, and dispatch work; inner
              ;; returns the Thread. SERVICE-PROGRAMS.md § "The lockstep".
              (((console-driver :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                (:wat::core::let*
                  (((spawn :wat::console::Spawn)
                    (:wat::console::spawn stdout stderr 1))
                   ((pool :wat::kernel::HandlePool<wat::console::Handle>)
                    (:wat::core::first spawn))
                   ((cd :wat::kernel::Thread<wat::core::unit,wat::core::unit>)
                    (:wat::core::second spawn))
                   ((handle :wat::console::Handle)
                    (:wat::kernel::HandlePool::pop pool))
                   ((_0 :wat::core::unit) (:wat::kernel::HandlePool::finish pool))
                   ((_work :wat::core::unit) (:my::dispatch-row-json handle)))
                  cd)))
              (:wat::core::match (:wat::kernel::Thread/join-result console-driver) -> :wat::core::unit
                ((:wat::core::Ok _) ())
                ((:wat::core::Err _) (:wat::test::assert-eq "console-driver-died" ""))))))
        (:wat::core::Vector :wat::core::String)))
     ((stdout :wat::core::Vector<wat::core::String>) (:test::tel-stdout-from-result r)))
    (:test::tel-assert-line-once stdout "[1,2,3]")))
