;; tests/program/wat_arc170_program_contracts_t17_run_hermetic.wat — happy-path hermetic run.
;;
;; Arc 278 IPC de-prime — migrated off `:wat::test::run-hermetic` onto the primed peer wire.
;; What run-hermetic did (spawn a child, run its body, drain stdout), the composed primes do
;; directly: `spawn-program' (process)` spawns the peer, its `:user::main` computes 2+2 and
;; `println`s it, and the parent drains that single value off the peer via `recv'` — the doubled
;; value arrives as a `RecvOutcome::Message` (proven: tests/kernel/wat_hermetic_round_trip.wat).
;;
;; This defn returns the received i64 directly so the test measures the value that genuinely
;; crossed the wire (== 4). The peer's death (were it to die) is surfaced via the Lost arm,
;; NEVER swallowed.
(:wat::core::defn :my::test::two-plus-two [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil
             (:wat::kernel::println (:wat::i64::+ 2 2)))))]
    (:wat::core::match (:wat::kernel::recv p)
      ((:wat::kernel::RecvOutcome::Message m) m)
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "two-plus-two: stop requested before child sent its value — child was ALIVE, channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "two-plus-two: child closed before sending its value" :wat::core::None :wat::core::None)))))
