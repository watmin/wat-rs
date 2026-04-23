;; wat-tests/holon/Trigram.wat — tests for wat/holon/Trigram.wat (→ Ngram → Sequential).
;;
;; Trigram(a,b,c,d) = Bundle([Sequential(a,b,c), Sequential(b,c,d)]).
;; Presence of the first 3-window's Sequential against the full
;; Trigram is above the noise floor (it's a participant in the
;; bundle); presence of an unrelated atom is below. Exercises the
;; full stdlib chain: Trigram → Ngram → map over window → Sequential
;; → foldl + map-with-index + Permute + Bind.

(:wat::config::set-capacity-mode! :error)
(:wat::config::set-dims! 1024)

(:wat::test::deftest :wat-tests::holon::Trigram::test-window-participant-above-floor
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST) (:wat::holon::Atom "a"))
     ((b :wat::holon::HolonAST) (:wat::holon::Atom "b"))
     ((c :wat::holon::HolonAST) (:wat::holon::Atom "c"))
     ((d :wat::holon::HolonAST) (:wat::holon::Atom "d"))
     ((window-1 :wat::holon::HolonAST)
      (:wat::holon::Sequential (:wat::core::list :wat::holon::HolonAST a b c)))
     ;; Trigram returns :Result<HolonAST, CapacityExceeded>. 4 atoms at
     ;; d=1024 is well under the capacity budget; Err is unreachable
     ;; but the type system still demands we acknowledge it.
     ((full :wat::holon::HolonAST)
      (:wat::core::match
        (:wat::holon::Trigram (:wat::core::list :wat::holon::HolonAST a b c d))
        -> :wat::holon::HolonAST
        ((Ok h) h)
        ((Err _) a))))
    (:wat::test::assert-eq (:wat::holon::presence? window-1 full) true)))

(:wat::test::deftest :wat-tests::holon::Trigram::test-outsider-below-floor
  ()
  (:wat::core::let*
    (((a :wat::holon::HolonAST) (:wat::holon::Atom "a"))
     ((b :wat::holon::HolonAST) (:wat::holon::Atom "b"))
     ((c :wat::holon::HolonAST) (:wat::holon::Atom "c"))
     ((d :wat::holon::HolonAST) (:wat::holon::Atom "d"))
     ((z :wat::holon::HolonAST) (:wat::holon::Atom "unrelated-z"))
     ((full :wat::holon::HolonAST)
      (:wat::core::match
        (:wat::holon::Trigram (:wat::core::list :wat::holon::HolonAST a b c d))
        -> :wat::holon::HolonAST
        ((Ok h) h)
        ((Err _) a))))
    (:wat::test::assert-eq (:wat::holon::presence? z full) false)))
