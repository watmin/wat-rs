;; HashMap receiver — unchanged (already passes — MUST STILL PASS)
(:wat::core::defn :u::f [] -> :wat::core::i64
  (:wat::core::let
    [m {:port 8080}
     v (:port m)]
    (:wat::core::Option/expect v "expected :port")))
(:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "ok"))
