;; Stone 255.89 — one deftest in each spelling, in one file.
;;
;; The scanner stores the name as written. `defn` registers the function under
;; `canonical_identity` of that name. `run_single_deftest` looks the function
;; up by that identity, so both of these run.

(wat.test/deftest wat-tests.probe25589.lookup/symbol-spelling
  (wat.test/assert-eq 1 1))

(:wat::test::deftest :wat-tests::probe25589::lookup::keyword-spelling
  (wat.test/assert-eq 1 1))
