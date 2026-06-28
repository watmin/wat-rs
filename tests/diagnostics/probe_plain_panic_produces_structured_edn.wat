;; tests/diagnostics/probe_plain_panic_produces_structured_edn.wat — co-located fixture for the
;; sibling probe (.rs), slurped via startup_beside(file!()).
;;
;; Body: dim_count=1 → budget=floor(sqrt(1))=1; a Bundle with 2 atoms exceeds capacity
;; and triggers panic!("...: capacity exceeded ...") — a bare Rust String panic, NOT an
;; AssertionPayload. This is the only reliably reachable non-AssertionPayload panic path
;; from a wat body.
(:wat::core::defn :probe::plain-panic [] -> :wat::kernel::RunResult
  (:wat::test::run-hermetic
      (:wat::core::do
        (:wat::config::set-dim-count! 1)
        (:wat::config::set-capacity-mode! :panic)
        ;; Two Atom children exceed floor(sqrt(1))=1 budget
        ;; → panic!("capacity exceeded under :panic") fires inside eval_algebra_bundle.
        (:wat::core::let
          [_bundle
            (:wat::holon::Bundle
              (:wat::core::Vector :wat::holon::HolonAST
                (:wat::holon::to-holon "key1")
                (:wat::holon::to-holon "key2")))]
          :wat::core::nil))))
