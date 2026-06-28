;; tests/rete/probe_arc278_2a_alpha_match.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Defines the :user::Temp record used by the alpha-match tests.

(:wat::core::defrecord :user::Temp [value <- :wat::core::i64])
(:wat::core::defn :user::main [] -> :wat::core::nil nil)
