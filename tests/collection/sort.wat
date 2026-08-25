;; tests/collection/sort.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). Functions RETURN their results as String/i64
;; so tests use eval_in_frozen instead of stdout capture.

(:wat::core::defn :sort::ascending-i64 [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::i64 3 1 4 1 5 9 2 6)
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
          (:wat::core::< a b))
        xs)]
    (:wat::string::join ","
      (:wat::core::mapv
        (:wat::core::fn [n <- :wat::core::i64] -> :wat::core::String
          (:wat::core::i64::to-string n))
        sorted))))

(:wat::core::defn :sort::descending-f64 [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::f64 1.5 0.5 2.5 1.0)
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- :wat::core::f64 b <- :wat::core::f64] -> :wat::core::bool
          (:wat::core::> a b))
        xs)]
    (:wat::string::join ","
      (:wat::core::mapv
        (:wat::core::fn [x <- :wat::core::f64] -> :wat::core::String
          (:wat::core::f64::to-string x))
        sorted))))

(:wat::core::defn :sort::string-asc [] -> :wat::core::String
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::String "banana" "apple" "cherry")
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- :wat::core::String b <- :wat::core::String] -> :wat::core::bool
          (:wat::core::< a b))
        xs)]
    (:wat::string::join "," sorted)))

(:wat::core::defn :sort::empty-length [] -> :wat::core::i64
  (:wat::core::let
    [xs (:wat::core::Vector :wat::core::i64)
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- :wat::core::i64 b <- :wat::core::i64] -> :wat::core::bool
          (:wat::core::< a b))
        xs)]
    (:wat::core::length sorted)))

(:wat::core::defn :sort::tuple-first-field [] -> :wat::core::String
  (:wat::core::let
    [xs
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])
        (:wat::core::Tuple 30 "alice")
        (:wat::core::Tuple 25 "carol")
        (:wat::core::Tuple 28 "bob"))
     sorted
      (:wat::core::sort
        (:wat::core::fn [a <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String]) b <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])] -> :wat::core::bool
          (:wat::core::< (:wat::core::first a) (:wat::core::first b)))
        xs)]
    (:wat::string::join ","
      (:wat::core::mapv
        (:wat::core::fn [p <- (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String])] -> :wat::core::String
          (:wat::core::second p))
        sorted))))
