;; PROBE — room 4 judgment call, site `src/runtime.rs:13011` (`is_match_canonical`, the CEK
;; stepper's "is this scrutinee already a value" check used by `:wat::eval::walk`/`:wat::eval-step!`
;; — distinct from the tree-walk interpreter's `try_match_pattern` covered by the main probe).
;;
;; ⭐ MEASURED FINDING: reachable. Before the fix, a quoted match form whose scrutinee was written
;; `(:wat::core::Option::Some 5)` was NOT match-canonical (only the bare FQDN heads were
;; recognised), so `step_match` tried to reduce it as an ordinary call and failed with
;; `no-step-rule for op: :wat::core::variant` — the same shape of gap as the tree-walk matcher's
;; `PatternMatchFailed`, one layer down in the small-step stepper. After extending the `matches!`
;; at :13011 with the three qualified constructor spellings, the walk completes and returns the
;; expected `(terminal-value, visit-count)` pair — `[5 2]`, the same shape the DESIGN's own
;; precedent measured for the bare-symbol shorthand before its retirement.

(:wat::core::defn :my::test::count-visit
  [acc <- :wat::core::i64 form <- :wat::WatAST step <- :wat::eval::StepResult]
  -> (:wat::eval::WalkStep :- [:wat::core::i64])
  (:wat::eval::WalkStep::Continue (:wat::i64::+ acc 1)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match
    (:wat::eval::walk
      (:wat::core::quote
        (:wat::core::match (:wat::core::Option::Some 5)
          ((:wat::core::Option::Some n) n)
          (:wat::core::Option::None 0)))
      0
      :my::test::count-visit)
    ((:wat::core::Ok pair)
      (:wat::kernel::println pair))
    ((:wat::core::Err e)
      (:wat::kernel::println e))))
