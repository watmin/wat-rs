;; tests/collection/sort.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Functions RETURN their results as String/i64
;; so tests use eval_in_frozen instead of stdout capture.

(:wat::core::defn :sort::ascending-i64 [] -> wat.type/String
  (:wat::core::let
    [xs (wat.type/Vector :- [wat.type/i64] 3 1 4 1 5 9 2 6)
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- wat.type/i64 b <- wat.type/i64] -> wat.type/bool
          (:wat::core::< a b))
        xs)]
    (:wat::string::join ","
      (:wat::core::mapv
        (:wat::core::fn [n <- wat.type/i64] -> wat.type/String
          (:wat::i64::to-string n))
        sorted))))

(:wat::core::defn :sort::descending-f64 [] -> wat.type/String
  (:wat::core::let
    [xs (wat.type/Vector :- [wat.type/f64] 1.5 0.5 2.5 1.0)
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- wat.type/f64 b <- wat.type/f64] -> wat.type/bool
          (:wat::core::> a b))
        xs)]
    (:wat::string::join ","
      (:wat::core::mapv
        (:wat::core::fn [x <- wat.type/f64] -> wat.type/String
          (:wat::f64::to-string x))
        sorted))))

(:wat::core::defn :sort::string-asc [] -> wat.type/String
  (:wat::core::let
    [xs (wat.type/Vector :- [wat.type/String] "banana" "apple" "cherry")
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- wat.type/String b <- wat.type/String] -> wat.type/bool
          (:wat::core::< a b))
        xs)]
    (:wat::string::join "," sorted)))

(:wat::core::defn :sort::empty-length [] -> wat.type/i64
  (:wat::core::let
    [xs (wat.type/Vector :- [wat.type/i64])
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- wat.type/i64 b <- wat.type/i64] -> wat.type/bool
          (:wat::core::< a b))
        xs)]
    (:wat::core::length sorted)))

(:wat::core::defn :sort::tuple-first-field [] -> wat.type/String
  (:wat::core::let
    [xs
      (wat.type/Vector :- [(wat.type/Tuple :- [wat.type/i64 wat.type/String])]
        (wat.type/Tuple :- [wat.type/i64 wat.type/String] 30 "alice")
        (wat.type/Tuple :- [wat.type/i64 wat.type/String] 25 "carol")
        (wat.type/Tuple :- [wat.type/i64 wat.type/String] 28 "bob"))
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- (wat.type/Tuple :- [wat.type/i64 wat.type/String]) b <- (wat.type/Tuple :- [wat.type/i64 wat.type/String])] -> wat.type/bool
          (:wat::core::< (:wat::core::first a) (:wat::core::first b)))
        xs)]
    (:wat::string::join ","
      (:wat::core::mapv
        (:wat::core::fn [p <- (wat.type/Tuple :- [wat.type/i64 wat.type/String])] -> wat.type/String
          (:wat::core::second p))
        sorted))))
