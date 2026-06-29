(:wat::core::defrecord :user::MyEnv [token <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::Record
  (:wat::program::Env/user.program (:wat::program::env)))
