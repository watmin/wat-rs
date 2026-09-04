;; arc 292 — RED probe for the PROCESS-tier timer-Peer (send_after / time-as-select).
;;
;; Mirror of wat-tests/timer-after.wat, but the LOCUS is (:wat::spawn::process),
;; so (:wat::kernel::after ...) must return (Process' :- [nil O]) and drop into the
;; PROCESS-tier select' (io_uring reactor) — not the thread tier.
;;
;; The contract (DESIGN.md rev2 + arc-292 doctrine): a one-shot timer DELIVERS a
;; caller-chosen, typed message of the select' set's own type O, after a Duration
;; (Erlang's send_after). Best-of-breed Linux: the delay arrives via the io_uring
;; reactor's OWN native timeout op (IORING_OP_TIMEOUT) — zero extra fds, one
;; timing mechanism across the all-io_uring process reactor.
;;
;; RED at HEAD: eval_kernel_after rejects a ProcessOpts locus with a MalformedForm
;; ("io_uring timer not yet implemented", runtime.rs:25035), AND infer_kernel_after
;; returns (Thread' :- [nil O]) for every locus (check.rs:11182) so the Process'-typed
;; Vector mismatches at check time. Either way the probe fails on EXACTLY the one
;; missing primitive: the process-tier after. Everything else (process select',
;; Vector, ServiceEvent, :wat::time::Millisecond) already exists.

(:wat::test::deftest :wat-tests::timer::after-delivers-its-message-process
  
  (:wat::test::assert-eq
    (:wat::core::match
      (:wat::kernel::select
        (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])]
          (:wat::kernel::after :wat::program::PeerKind::process (:wat::time::Milliseconds 50) :tick)))
       
      ((:wat::spawn::ServiceEvent::Message _idx msg) msg)
      ((:wat::spawn::ServiceEvent::Closed _idx) :no-tick)
      ((:wat::spawn::ServiceEvent::Lost _idx _cause) :no-tick)
      ((:wat::spawn::ServiceEvent::Malformed _idx _cause) :no-tick)  ;; arc 278 — unreachable for a timer
      ((:wat::spawn::ServiceEvent::Rejected _idx _cause) :no-tick)   ;; arc 278 Stone 1a — unreachable for a timer
      (:wat::spawn::ServiceEvent::Shutdown :no-tick)
      ((:wat::spawn::ServiceEvent::Connection _peer) :no-tick)
      ((:wat::spawn::ServiceEvent::Admin _msg) :no-tick))
    :tick))
