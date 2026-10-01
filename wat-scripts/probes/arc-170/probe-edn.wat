;; CLAIM: edn/write . edn/read round-trips a Vector[i64] to data equal to the original —
;; asserted as DATA (the parsed value equals the original vector), never by comparing the
;; rendered text (coordinator correction, 255.76 weigh: "measuring strings is anti-wat").
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [original     (wat.type/Vector :- [wat.type/i64] 2 4 6)
     rendered     (:wat::edn::write original)
     roundtripped (:wat::core::ann-form (:wat::edn::read rendered) (wat.type/Vector :- [wat.type/i64]))]
    (:wat::core::do
      (:wat::test::assert-eq roundtripped original)
      (:wat::kernel::println rendered))))
