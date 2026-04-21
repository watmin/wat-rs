;; wat-tests/std/Trigram.wat — tests for wat/std/Trigram.wat (→ Ngram → Sequential).
;;
;; Trigram(a,b,c,d) = Bundle([Sequential(a,b,c), Sequential(b,c,d)]).
;; Presence of the first 3-window's Sequential against the full
;; Trigram is above the noise floor (it's a participant in the
;; bundle); presence of an unrelated atom is below. Exercises the
;; full stdlib chain: Trigram → Ngram → map over window → Sequential
;; → foldl + map-with-index + Permute + Bind.

(:wat::config::set-dims! 1024)
(:wat::config::set-capacity-mode! :error)

(:wat::test::deftest :wat-tests::std::Trigram::test-window-participant-above-floor 1024 :error
  (:wat::core::let*
    (((a :holon::HolonAST) (:wat::algebra::Atom "a"))
     ((b :holon::HolonAST) (:wat::algebra::Atom "b"))
     ((c :holon::HolonAST) (:wat::algebra::Atom "c"))
     ((d :holon::HolonAST) (:wat::algebra::Atom "d"))
     ((window-1 :holon::HolonAST)
      (:wat::std::Sequential (:wat::core::list :holon::HolonAST a b c)))
     ;; Trigram returns :Result<HolonAST, CapacityExceeded>. 4 atoms at
     ;; d=1024 is well under the capacity budget; Err is unreachable
     ;; but the type system still demands we acknowledge it.
     ((full :holon::HolonAST)
      (:wat::core::match
        (:wat::std::Trigram (:wat::core::list :holon::HolonAST a b c d))
        -> :holon::HolonAST
        ((Ok h) h)
        ((Err _) a))))
    (:wat::test::assert-eq (:wat::algebra::presence? window-1 full) true)))

(:wat::test::deftest :wat-tests::std::Trigram::test-outsider-below-floor 1024 :error
  (:wat::core::let*
    (((a :holon::HolonAST) (:wat::algebra::Atom "a"))
     ((b :holon::HolonAST) (:wat::algebra::Atom "b"))
     ((c :holon::HolonAST) (:wat::algebra::Atom "c"))
     ((d :holon::HolonAST) (:wat::algebra::Atom "d"))
     ((z :holon::HolonAST) (:wat::algebra::Atom "unrelated-z"))
     ((full :holon::HolonAST)
      (:wat::core::match
        (:wat::std::Trigram (:wat::core::list :holon::HolonAST a b c d))
        -> :holon::HolonAST
        ((Ok h) h)
        ((Err _) a))))
    (:wat::test::assert-eq (:wat::algebra::presence? z full) false)))
