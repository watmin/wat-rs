;; tests/comms/probe_arc293_W2c_compile_time_send.wat
;; Co-located fixture for probe_arc293_W2c_compile_time_send.rs (startup_beside).
;;
;; Arc 293.W.2c — compile-time send' wire-wall (the higher rung above the 2a runtime guard).
;;
;; A typed struct sent to a PROCESS peer is rejected by the type-checker
;; (infer_send_prime portability gate, src/check.rs). The world FAILS TO LOAD
;; with a check error. The runtime backstop (2a) is the lower rung; the compile-
;; time gate (2c) catches it before execution.
;;
;; This file contains ONLY the compile-time-rejectable form. The controls
;; (thread-exempt and record-portable cases) live in probe_arc293_W2c_controls.wat.

(:wat::core::defstruct :w2c::S [val <- :wat::core::i64])

;; Sends a bare struct (:w2c::S 99) to a PROCESS peer.
;; The 2c gate rejects this at CHECK: send' payload must be portable over a
;; wire peer; a struct is in-locus only (§7).
(:wat::core::defn :w2c::probe-send-struct [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::kernel::spawn-program' (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defn :user::main [] -> :wat::core::nil nil)))]
    (:wat::kernel::send' p (:w2c::S 99))))
