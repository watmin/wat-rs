;; tests/program/wat_arc170_slice_1f_gamma_orchestrator_row_e.wat — main that reads from stdin.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_s (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))]
    nil))
