;; Row 9 of EXPECTATIONS-zero-is-not-a-wait.md — computed (not literal) zero.
;; span.wat:131 is `(Millisecond (Record/metrics-flush-after-ms rec2))`.
;; The constructor argument is an i64 binding, not IntLit 0, so this type-checks
;; and the runtime wall is what fires.
(:wat::config::set-redef! true)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [n 0
     _ (:wat::time::Millisecond n)]
    (:wat::kernel::println "unreachable")))
