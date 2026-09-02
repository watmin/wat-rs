;; Named driver for BRIEF-the-fanout-is-concurrent row 1. Prints :user::fanout-is-max.
(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::fanout-is-max)))
