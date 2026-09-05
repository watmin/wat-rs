;; tests/process/probe_run_hermetic_ast_stdout_capture.wat — co-located fixture for probe_run_hermetic_ast_stdout_capture.rs
;;
;; Arc 278 IPC de-prime (MAP unit). Historically drove the non-prime
;; `:wat::test::run-hermetic` capture model (fork + OS-pipe stdout scrape →
;; :wat::kernel::RunResult), reading the child's `println` back out of
;; RunResult/stdout as the EDN-quoted line "\"hello-from-probe\"". Migrated onto
;; the PRIMED peer wire — a direct `(:wat::test::spawn-peer
;; (:wat::spawn::process) (:wat::core::forms …))` child + `(:wat::kernel::recv' p)`.
;; On the wire the child's printed value crosses DECODED: `(println "hello-from-probe")`
;; arrives as RecvOutcome::Message["hello-from-probe"] (native String), NOT the
;; EDN-quoted stdout scrape. Lost[LociDiedError] / Closed are never swallowed.
;; (shape: tests/kernel/wat_run_sandboxed_ast.wat compute-prints-hello.)

(:wat::core::defn :probe::ast::capture-stdout [] -> :wat::core::String
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println "hello-from-probe"))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "capture-stdout: stop requested before child sent its value — child was ALIVE, channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "capture-stdout: child closed before sending its value" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None)))))
