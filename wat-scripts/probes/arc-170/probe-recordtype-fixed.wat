;; Gate 2 — a user defrecord must ship to a process-bracket child WITH its fields.
;; Before the fix: child re-parses a fields-less recordtype and dies with
;;   "malformed recordtype declaration: expected (:recordtype :Name :Parent [fields]) ... got 2 args".
;; After: the freeze reconstructs (:recordtype :Name :root [x <- :i64]) and it works → [2 4 6].
;; CLAIM: bracket::map over a process locus with a user defrecord in scope returns [2 4 6].
(:wat::core::defrecord :probe::Foo [x <- wat.type/i64])

(:wat::core::defn :probe::double [n <- wat.type/i64] -> wat.type/i64
  (:wat::i64::* n 2))

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [result (:wat::bracket::map (:wat::spawn::process)
              (wat.type/Vector :- [wat.type/i64] 1 2 3)
              :probe::double)]
    (:wat::core::do
      (:wat::test::assert-eq result (wat.type/Vector :- [wat.type/i64] 2 4 6))
      (:wat::kernel::println result))))
