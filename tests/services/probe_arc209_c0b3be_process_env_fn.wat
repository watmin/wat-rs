(:wat::core::defrecord :app::Env [token <- :wat::core::i64])
(:wat::core::defn :app::make-env [] -> :wat::core::Record (:app::Env 7))
