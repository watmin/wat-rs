;; Does a plain top-level defrecord (no defservice) break the process bracket?
;; CLAIM: bracket::map over a process locus, with a plain defrecord in scope, returns [2 4 6].
(:wat::core::defrecord :probe::Foo [x <- wat.type/i64])
(:wat::core::defn :probe::double [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [nums (wat.type/Vector :- [wat.type/i64] 1 2 3)
     pr   (:wat::bracket::map (:wat::spawn::process) nums :probe::double)]
    (:wat::core::do
      (:wat::test::assert-eq pr (wat.type/Vector :- [wat.type/i64] 2 4 6))
      (:wat::kernel::println (:wat::edn::write pr)))))
