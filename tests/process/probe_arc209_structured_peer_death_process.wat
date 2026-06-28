;; tests/process/probe_arc209_structured_peer_death_process.wat
;; co-located fixture for probe_arc209_structured_peer_death_process.rs
;; startup_beside(file!()) world — structured peer death, PROCESS tier (Arc 209 C0b).
;;
;; :user::compute spawns a :process peer via spawn-program', sends it 0 (prompting readln),
;; then recv' — the child calls assertion-failed! carrying actual + expected, which raises.
;; The test asserts the raised error contains both structured fields.

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::process)
                           (:wat::core::forms
                             (:wat::core::defn :user::main [] -> :wat::core::nil
                               (:wat::core::let [n (:wat::kernel::readln -> :wat::core::i64)
                                                  _ (:wat::kernel::assertion-failed! "proc-structured-marker"
                                                      (:wat::core::Some "PROC-ACTUAL-5521")
                                                      (:wat::core::Some "PROC-EXPECTED-8841"))]
                                 nil))))
                    _ (:wat::kernel::send' peer 0)
                    got (:wat::kernel::recv' peer)]
    got))

