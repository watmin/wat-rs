;; CLAIM: edn/write of a Vector[i64] renders it as EDN text, "[2 4 6]" exactly — the probe's
;; own subject IS the rendering function, so asserting its string output is asserting the
;; function's return value, not standing a string in for something else.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [rendered (:wat::edn::write (wat.type/Vector :- [wat.type/i64] 2 4 6))]
    (:wat::core::do
      (:wat::test::assert-eq rendered "[2 4 6]")
      (:wat::kernel::println rendered))))
