;; tests/reflection/probe_arc255_ivb2a_examples_seam.wat
;; just-eval fixture for probe_arc255_ivb2a_examples_seam.rs.
;;
;; Wraps the :wat::intrinsic::examples reflection seam so the Rust driver can
;; inspect the returned Vector<Example> (registered scheme, check.rs).
(:wat::core::defn :user::examples []
  -> (:wat::core::Vector :- [:wat::intrinsic::Example])
  (:wat::intrinsic::examples))
