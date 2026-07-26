;; A top-level `let` is const-eval'd during freeze; its initializer's
;; Result/expect panics on an eval-ast! Err (unknown verb) — a freeze-time
;; panic that must surface structured, not vanish silently. See
;; tests/cli/wat_cli.rs::freeze_time_panic_surfaces_structured_not_silent
;; (arc 278 no-hidden-failures, R41 EGO SVM LEX).
(:wat::core::let
  [pf (:wat::core::Result/expect
        (:wat::eval-ast! (:wat::core::read-string "(:wat::core::this-verb-does-not-exist)"))
        "freeze-time boom")]
  pf)
