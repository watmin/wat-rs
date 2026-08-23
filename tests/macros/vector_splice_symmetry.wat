;; tests/macros/vector_splice_symmetry.wat — co-located fixture for vector_splice_symmetry.rs,
;; slurped via startup_beside(file!()) for the three positive tests.
;;
;; Three named compute functions (renamed from :my::compute to unique names).
;; The anaphoric (bad) test is in vector_splice_symmetry.wat.bad.

;; Test 1 (splice_of_vector_bound_symbol_succeeds): ~@xs on a Vector-bound symbol.
(:wat::core::defmacro :my::splice-vec
  [xs <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::Vector :wat::core::i64 ~@xs))

(:wat::core::defn :my::compute-splice [] -> (:wat::core::Vector :- [:wat::core::i64])
  (:my::splice-vec [10 20 30]))

;; Test 3 (hygienic_splice_adder_binds_via_spliced_names): computes names from spliced material.
(:wat::core::defmacro :my::make-adder
  [& params <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  (:wat::core::let
    [n0 (:wat::core::Option/expect (:wat::core::get params 0) "make-adder: missing param name 0")
     n1 (:wat::core::Option/expect (:wat::core::get params 3) "make-adder: missing param name 1")]
    `(:wat::core::fn [~@params] -> :wat::core::i64
        (:wat::core::i64::+ ~n0 ~n1))))

(:wat::core::defn :my::adder [] -> :wat::core::Fn(wat::core::i64,wat::core::i64)->wat::core::i64
  (:my::make-adder a <- :wat::core::i64 b <- :wat::core::i64))

(:wat::core::defn :my::compute-hygienic [] -> :wat::core::i64
  ((:my::adder) 7 35))

;; Test 4 (vector_splice_round_trip_matches_list_splice): Vector and List splice yield same result.
(:wat::core::defmacro :my::sum-list
  [& xs <- (:wat::core::Vector :- [:wat::WatAST])]
  -> :wat::WatAST
  `(:wat::core::i64::+ ~@xs))

(:wat::core::defmacro :my::sum-vec
  [xs <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::core::i64::+ ~@xs))

(:wat::core::defn :my::compute-round-trip [] -> :wat::core::i64
  (:wat::core::i64::-
              (:my::sum-vec [10 32])
              (:my::sum-list 10 32)))

