;; tests/rete/probe_arc278_3a_root_join.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :user::Temp record used by the root-join tests.

(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
