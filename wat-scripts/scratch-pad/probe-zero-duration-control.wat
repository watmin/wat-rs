

;; probe-zero-duration-control.wat — THE DISCRIMINATOR for Stone A.
;;
;; ⚠ ITS SUBJECT CHANGED WHEN THE WALL LANDED, and the old header is kept below the line
;; so the change is legible rather than silent.
;;
;; BEFORE Stone A it was the control for probe-zero-duration-disarms-at-process.wat: same
;; program, zero-timer cell removed, proving the WEDGE (subject EXIT=124 / control EXIT=0)
;; came from the zero duration and not from slowness.
;;
;; AFTER Stone A that job is gone — `(:wat::time::Nanosecond 0)` has no form, so the
;; subject relocated to tests/kernel/probe_zero_is_not_a_wait.wat.bad and is gated by a
;; Rust test asserting it FAILS to freeze.
;;
;; ★ The job that remains is the one that matters: A WALL THAT REJECTS EVERYTHING IS NOT A
;; WALL. This file proves the wall DISCRIMINATES — a positive duration still fires, at BOTH
;; loci, which is what makes the refusal of zero meaningful rather than a blanket ban.
;;
;;   thread-1ms=FIRED
;;   process-1ms=FIRED
;;   EXIT=0
;;
;; ⛔ The orchestrator's EXPECTATIONS row 2 asked this file to "still pass unchanged, both
;; cells FIRED" — where cell one was `(fire-thread 0)`. That row was UNSATISFIABLE by
;; construction: it required a zero wait to keep working in the stone that removes zero
;; waits. The row was the orchestrator's error, not the executor's, and the BRIEF compounded
;; it by ordering the file left unedited. Recorded so the next reader does not restore the
;; zero cell "to match the row".

(:wat::config::set-redef! true)

(:wat::core::defn :zd::fire-thread [ns <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Nanoseconds ns) :done))
    ((:wat::kernel::RecvOutcome::Message _m) "FIRED")
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED") (:wat::kernel::RecvOutcome::TimedOut "LOST")))

(:wat::core::defn :zd::fire-process [ns <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::process (:wat::time::Nanoseconds ns) :done))
    ((:wat::kernel::RecvOutcome::Message _m) "FIRED")
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED") (:wat::kernel::RecvOutcome::TimedOut "LOST")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:zd::fire-thread 1000000)
     _ (:wat::kernel::println (:wat::core::format "thread-1ms={a}" :a a))
     b (:zd::fire-process 1000000)
     _ (:wat::kernel::println (:wat::core::format "process-1ms={b}" :b b))
     both (:wat::core::if
            (:wat::core::and (:wat::core::= a "FIRED") (:wat::core::= b "FIRED"))
            "yes" "NO")]
    (:wat::kernel::println (:wat::core::format "wall-discriminates={c}" :c both))))
