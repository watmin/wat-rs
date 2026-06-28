;; tests/program/wat_arc170_slice_1f_gamma_orchestrator_row_b.wat — three child threads + main.
(:wat::core::defn :test::child-a [_in <- :wat::kernel::Receiver<wat::core::nil> _out <- :wat::kernel::Sender<wat::core::nil>] -> :wat::core::nil (:wat::kernel::println "child-a"))

(:wat::core::defn :test::child-b [_in <- :wat::kernel::Receiver<wat::core::nil> _out <- :wat::kernel::Sender<wat::core::nil>] -> :wat::core::nil (:wat::kernel::println "child-b"))

(:wat::core::defn :test::child-c [_in <- :wat::kernel::Receiver<wat::core::nil> _out <- :wat::kernel::Sender<wat::core::nil>] -> :wat::core::nil (:wat::kernel::println "child-c"))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [thr-a (:wat::kernel::spawn-thread :test::child-a)
     thr-b (:wat::kernel::spawn-thread :test::child-b)
     thr-c (:wat::kernel::spawn-thread :test::child-c)
     _a (:wat::kernel::Thread/drain-and-join thr-a)
     _b (:wat::kernel::Thread/drain-and-join thr-b)
     _c (:wat::kernel::Thread/drain-and-join thr-c)]
    nil))
