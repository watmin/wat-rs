;; tests/diagnostics/probe_arc296_here.wat — co-located fixture
;;
;; Arc 296 — `(:wat::kernel::here)` nullary intrinsic.
;;
;; RED at HEAD: :wat::kernel::here is unknown → startup fails at the type-check
;; step with an unresolved-verb error.
;;
;; GREEN after arc 296 addition: startup succeeds; main runs and asserts that
;; Location/line > 0 (proves the returned Location carries a real source coord).

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [loc (:wat::kernel::here)]
    (:wat::test::assert-true (:wat::core::> (:wat::kernel::Location/line loc) 0))))
