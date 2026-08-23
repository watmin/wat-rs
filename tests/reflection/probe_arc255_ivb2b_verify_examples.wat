;; tests/reflection/probe_arc255_ivb2b_verify_examples.wat
;; just-eval fixture for probe_arc255_ivb2b_verify_examples.rs.
;;
;; Wraps :wat::doctest::verify-examples (wat/doctest.wat) so the Rust driver can
;; count the returned Vector<Failure> (empty = every doctest passed).
(:wat::core::defn :user::verify []
  -> (:wat::core::Vector :- [:wat::doctest::Failure])
  (:wat::doctest::verify-examples))
