;; CLAIM: :wat::deporder::verify-stdlib finds zero dependency-order violations in the stdlib
;; as loaded (an empty vector — the stdlib's own def order is self-consistent).
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [violations (:wat::deporder::verify-stdlib)]
    (:wat::core::do
      (:wat::test::assert-eq (:wat::core::length violations) 0)
      (:wat::kernel::println (:wat::edn::write violations)))))
