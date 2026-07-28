;; Fixture for wat_cli::sigterm_reaches_a_program_blocked_on_stdin.
;;
;; The shape every interactive wat program has: print a marker, then BLOCK waiting for
;; input. This is the REPL's shape, the stdio-service demo's shape, and repl-daemon's.
;;
;; The contract under test is the same one `sigterm_to_cli_cascades_via_polling_contract`
;; asserts for a COMPUTE loop — SIGTERM is a flag the program observes, not a kill — but
;; measured where the program is parked in a read instead of spinning on `stopped?`.
;; A compute loop reaches its poll on its own; a blocked read never reaches anything
;; unless the read itself is multiplexed against the shutdown signal.

(:wat::core::defn :demo::loop [] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::read-frame )
    ((:wat::kernel::ReadFrameOutcome::Frame text)
      (:wat::core::do
        (:wat::kernel::println text)
        (:demo::loop)))
    (:wat::kernel::ReadFrameOutcome::Eof ())))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "READY")
    (:demo::loop)))
