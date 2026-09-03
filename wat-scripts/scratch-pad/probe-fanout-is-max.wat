;; Named driver for BRIEF-the-topic-is-durable row 1. Prints :user::durable-ok.
(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::durable-ok)))
