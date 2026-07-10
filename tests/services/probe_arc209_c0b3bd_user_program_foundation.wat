(:wat::core::defrecord :user::MyEnv [token <- :wat::core::i64])
;; main returns :nil (arc-170 wall); per the IPC triangle (recovery §13) it writes the
;; injected user.program as EDN to stdout, which the test captures + checks structurally.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::program::Env/user.program (:wat::program::env))))
