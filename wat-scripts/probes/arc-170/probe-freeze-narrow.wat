;; Does a plain top-level defrecord (no defservice) break the process bracket?
(:wat::core::defrecord :probe::Foo [x <- wat.type/i64])
(:wat::core::defn :probe::double [n <- wat.type/i64] -> wat.type/i64 (:wat::i64::* n 2))
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [nums (wat.type/Vector :- [wat.type/i64] 1 2 3)
     pr   (:wat::bracket::map (:wat::spawn::process) nums :probe::double)]
    (:wat::kernel::println (:wat::edn::write pr))))
