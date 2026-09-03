;; Rows 5/6/7 of EXPECTATIONS-zero-is-not-a-wait.md — measurement, readout, arith.
(:wat::config::set-redef! true)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [t  (:wat::time::now)
     z  (:wat::time::- t t)
     c  (:wat::time::Millisecond 5)
     t2 (:wat::time::+ t c)
     n  (:wat::time::nanoseconds z)
     m  (:wat::time::milliseconds c)]
    (:wat::kernel::println (:wat::core::format "row5 show={s} ns={n}"
      :s (:wat::core::show z) :n n))
    (:wat::kernel::println (:wat::core::format "row6 ctor-ms={m} measure-ns={n}"
      :m m :n n))
    (:wat::kernel::println (:wat::core::format "row7 later={ok}"
      :ok (:wat::core::> (:wat::time::epoch-nanos t2) (:wat::time::epoch-nanos t))))))
