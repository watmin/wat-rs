;; probe-zero-duration-control.wat — THE CONTROL for
;; probe-zero-duration-disarms-at-process.wat.
;;
;; Byte-identical to the subject except the ONE zero-duration process timer is not armed.
;; That is the only variable.
;;
;;   subject (arms after(process, Nanosecond 0))  -> EXIT=124, 3/3   (cannot shut down)
;;   control (does not arm it)                    -> EXIT=0,   3/3   (clean exit)
;;
;; This is what establishes that the zero-duration timer WEDGES TEARDOWN, rather than
;; the program merely being slow to exit. Read the subject's header for the full finding.

(:wat::config::set-redef! true)

(:wat::core::defn :zd::fire-thread [ns <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Nanosecond ns) :done))
    ((:wat::kernel::RecvOutcome::Message _m) "FIRED")
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED")))

(:wat::core::defn :zd::fire-process [ns <- :wat::core::i64] -> :wat::core::String
  (:wat::core::match
    (:wat::kernel::recv
      (:wat::kernel::after :wat::program::PeerKind::process (:wat::time::Nanosecond ns) :done))
    ((:wat::kernel::RecvOutcome::Message _m) "FIRED")
    ((:wat::kernel::RecvOutcome::Lost _c) "LOST")
    (:wat::kernel::RecvOutcome::Stopped "STOPPED")
    (:wat::kernel::RecvOutcome::Closed "CLOSED")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [a (:zd::fire-thread 0)
     _ (:wat::kernel::println (:wat::core::format "thread-ns0={a}" :a a))
     b (:zd::fire-process 1000000)
     _ (:wat::kernel::println (:wat::core::format "process-ns1ms={b}" :b b))
     c "SKIPPED-no-zero-timer"]
    (:wat::kernel::println (:wat::core::format "process-ns0={c}" :c c))))
