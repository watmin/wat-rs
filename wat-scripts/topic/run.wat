;; Entry for the 3 3 differential. set-redef! belongs in the entry file only —
;; sns-fanout.wat is load-file!'d by the circuit and cannot hold a setter.
(:wat::config::set-redef! true)
(:wat::load-file! "sns-fanout.wat")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::loci)))
