;; tests/program/wat_arc170_slice_1f_gamma_orchestrator_row_c.wat — panic child + main.
(:wat::core::defn :test::child-panic [_in <- :wat::kernel::Receiver<wat::core::nil> _out <- :wat::kernel::Sender<wat::core::nil>] -> :wat::core::nil (:wat::runtime::panic! "child panicked intentionally"))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [thr (:wat::kernel::spawn-thread :test::child-panic)
     _join (:wat::kernel::Thread/drain-and-join thr)]
    nil))
