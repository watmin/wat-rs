;; probe-refused-gap.wat — drive :user::refused-is-retried with a 300 ms gap.
;; Row 2: the race becomes an assertion, or passes. Never stalls.

(:wat::config::set-redef! true)
(:wat::load-file! "../topic/sns-fanout.wat")

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println (:user::refused-is-retried))
     _ (:wat::kernel::println (:user::refused-is-retried-gap 300))]
    (:wat::kernel::println (:user::stalled-does-not-stall))))
