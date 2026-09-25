;; tests/comms/probe_arc293_W2c_controls.wat
;; Control fixture for probe_arc293_W2c_compile_time_send.rs.
;;
;; Arc 293.W.2c — guards against over-rejection by the send' wire-wall.
;;
;; Two controls that MUST type-check (world loads without error):
;;
;;   1. struct over a THREAD peer — exempt (in-locus, same address space,
;;      crossbeam channel; no serialization). The gate must NOT fire for Thread'.
;;
;;   2. record over a PROCESS peer — portable (records are wire-serializable).
;;      The gate must NOT fire for portable payload types.

(:wat::core::defrecord :w2c_ctrl::R [val <- :wat::core::i64])

;; 255.30 — the struct-on-thread arm moved. A struct on a thread peer is refused
;; (tests/kernel/probe_arc255_30_struct_on_thread_peer.wat.bad). This file keeps the
;; record-on-process control, which must still load.

;; Record control: parent sends a portable record to a PROCESS child.
;; Records are wire-serializable; the gate must not fire.
(:wat::core::defn :w2c_ctrl::probe-send-record-to-process [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defrecord :w2c_ctrl::R [val <- :wat::core::i64])
           (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "spawned child"))))]
    (:wat::core::match (:wat::kernel::send p (:w2c_ctrl::R :val 42))
      [:wat::kernel::SendOutcome.Sent {} nil]
      [:wat::kernel::SendOutcome.Closed {} nil]
      [:wat::kernel::SendOutcome.Lost {:cause _c} nil]
      [:wat::kernel::SendOutcome.Stopped {} nil]))) ;; arc 278 #73 — fire-and-forget record send; outcome ignored uniformly regardless of cause
