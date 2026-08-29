;; wat-tests/holon/Bigram.wat — tests for wat/holon/Bigram.wat.
;;
;; :wat::holon::Bigram (058-013) expands to (Ngram 2 xs): pure sugar
;; for pair-wise adjacency encoding. Two claims:
;;
;; 1. Sugar equivalence — (Bigram xs) coincides with (Ngram 2 xs).
;;    Proves the macro is literally "same semantics as (Ngram 2 xs)."
;; 2. Window participant — the first 2-window's Sequential is present
;;    in the Bigram bundle; an unrelated atom is not.


;; ─── 1. sugar equivalence ──────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Bigram::test-bigram-is-ngram-2
  
  (:wat::core::let
    [a (:wat::holon::to-holon "a")
     b (:wat::holon::to-holon "b")
     c (:wat::holon::to-holon "c")
     xs (:wat::core::Vector :- [:wat::holon::HolonAST] a b c)
     bigram
       (:wat::core::match
         (:wat::holon::Bigram xs)
         
         ((:wat::core::Ok h) h)
         ((:wat::core::Err _) a))
     ngram2
       (:wat::core::match
         (:wat::holon::Ngram 2 xs)
         
         ((:wat::core::Ok h) h)
         ((:wat::core::Err _) a))]
    (:wat::test::assert-eq
      (:wat::holon::coincident? bigram ngram2)
      true)))

;; ─── 2. window participant ──────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Bigram::test-bigram-window-participant-above-floor
  
  (:wat::core::let
    [a (:wat::holon::to-holon "a")
     b (:wat::holon::to-holon "b")
     c (:wat::holon::to-holon "c")
     ;; The first 2-window in [a b c] is Sequential([a b]).
     window-1
       (:wat::holon::Sequential (:wat::core::Vector :- [:wat::holon::HolonAST] a b))
     full
       (:wat::core::match
         (:wat::holon::Bigram (:wat::core::Vector :- [:wat::holon::HolonAST] a b c))
         
         ((:wat::core::Ok h) h)
         ((:wat::core::Err _) a))]
    (:wat::test::assert-eq (:wat::holon::presence? window-1 full) true)))

(:wat::test::deftest :wat-tests::holon::Bigram::test-bigram-outsider-below-floor
  
  (:wat::core::let
    [a (:wat::holon::to-holon "a")
     b (:wat::holon::to-holon "b")
     c (:wat::holon::to-holon "c")
     z (:wat::holon::to-holon "unrelated-z")
     full
       (:wat::core::match
         (:wat::holon::Bigram (:wat::core::Vector :- [:wat::holon::HolonAST] a b c))
         
         ((:wat::core::Ok h) h)
         ((:wat::core::Err _) a))]
    (:wat::test::assert-eq (:wat::holon::presence? z full) false)))
