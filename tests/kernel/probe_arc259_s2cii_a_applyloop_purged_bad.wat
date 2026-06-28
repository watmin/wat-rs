;; Rejection fixture for probe_arc259_s2cii_a_applyloop_purged.rs — s2cii_a_apply_loop_prog_rejected.
;; Post-purge: apply-loop prog [i64]->i64 handed to spawn-program' :thread must be REJECTED.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [peer (:wat::kernel::spawn-program' (:wat::spawn::thread)
                           (:wat::core::fn [input <- :wat::core::i64] -> :wat::core::i64 input))]
    nil))
