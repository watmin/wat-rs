;; tests/collection/wat_arc167_vector_ast.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Arc 215 stone 2: [1 2 3] at value position type-checks.

(:wat::core::defn :my::probe [] -> :wat::core::i64 (:wat::core::length [1 2 3]))

