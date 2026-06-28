;; tests/rete/probe_arc278_3b_hash_join.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the Temperature and WindSpeed records for hash-join tests.

(:wat::core::defrecord :user::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :user::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
