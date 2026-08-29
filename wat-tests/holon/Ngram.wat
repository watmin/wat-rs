;; wat-tests/holon/Ngram.wat — tests for wat/holon/Ngram.wat.
;;
;; :wat::holon::Ngram (058-013) slides a size-n window across xs and
;; bundles Sequential encodings of every window. Two load-bearing
;; claims beyond what Trigram already covers:
;;
;; 1. Empty bundle on oversize n — (Ngram n xs) with n > length(xs)
;;    returns Ok with an empty bundle; no window item is present.
;;    Anchors the documented edge case (Q2: n > xs.len() → empty bundle).
;; 2. n-parametricity — (Ngram 2 xs) and (Ngram 3 xs) produce
;;    non-coincident encodings for the same xs. Proves that the
;;    window width is load-bearing and distinguishes Ngram from a
;;    fixed-arity Sequential call.


;; ─── 1. empty bundle on oversize n ─────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Ngram::test-ngram-empty-on-oversize
  
  (:wat::core::let
    [a (:wat::holon::to-holon "a")
     b (:wat::holon::to-holon "b")
     c (:wat::holon::to-holon "c")
     ;; n=5 > len([a b c])=3 → window returns [] → Bundle([]) → Ok(empty bundle).
     result
       (:wat::core::match
         (:wat::holon::Ngram 5 (:wat::core::Vector :- [:wat::holon::HolonAST] a b c))
         
         ((:wat::core::Ok h) h)
         ((:wat::core::Err _) a))
     ;; An empty bundle carries no signal — none of the input atoms
     ;; are present in it.
     ]
    (:wat::test::assert-eq (:wat::holon::presence? a result) false)))

;; ─── 2. n-parametricity ────────────────────────────────────────────

(:wat::test::deftest :wat-tests::holon::Ngram::test-ngram-n2-vs-n3-differ
  
  (:wat::core::let
    [a (:wat::holon::to-holon "a")
     b (:wat::holon::to-holon "b")
     c (:wat::holon::to-holon "c")
     d (:wat::holon::to-holon "d")
     xs (:wat::core::Vector :- [:wat::holon::HolonAST] a b c d)
     ;; Ngram 2 [a b c d] = Bundle([Seq(a,b), Seq(b,c), Seq(c,d)]) — three 2-windows.
     ;; Ngram 3 [a b c d] = Bundle([Seq(a,b,c), Seq(b,c,d)])       — two 3-windows.
     ;; Different window sizes → structurally distinct bundles → not coincident.
     n2
       (:wat::core::match
         (:wat::holon::Ngram 2 xs)
         
         ((:wat::core::Ok h) h)
         ((:wat::core::Err _) a))
     n3
       (:wat::core::match
         (:wat::holon::Ngram 3 xs)
         
         ((:wat::core::Ok h) h)
         ((:wat::core::Err _) a))]
    (:wat::test::assert-eq
      (:wat::holon::coincident? n2 n3)
      false)))
