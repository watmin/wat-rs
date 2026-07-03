;; tests/rete/probe_arc278_2b_insert_alpha.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :user::Temp record used by the insert/fire tests.

(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])
