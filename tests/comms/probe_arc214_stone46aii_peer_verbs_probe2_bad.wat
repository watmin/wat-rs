;; tests/comms/probe_arc214_stone46aii_peer_verbs_probe2_bad.wat
;; Arc 214 Stone 4.6a-ii Probe 2 (CHECK NEGATIVE): recv' return type projects O.
;; A defn declaring -> :String whose body recv's from an i64-peer MUST fail at check.
;; startup_from_file on this file is expected to return Err.

(:wat::core::defn :user::bad-recv [] -> :wat::core::String
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                           (:wat::core::fn [self <- :wat::kernel::Peer'<wat::core::i64,wat::core::i64>] -> :wat::core::nil
                             (:wat::kernel::send' self (:wat::kernel::recv' self))))]
    (:wat::kernel::recv' peer)))

(:wat::core::defn :user::main [] -> :wat::core::nil nil)
