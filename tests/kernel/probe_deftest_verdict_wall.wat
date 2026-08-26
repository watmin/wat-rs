;; Co-located fixture for probe_deftest_verdict_wall.rs — arc 278, the vacuous-gate wall.
;;
;; Three subjects, one per channel the wall must keep honest:
;;
;;   :user::verdict-wall-passes — a deftest whose assertion HOLDS. The verdict verb must
;;                                report Passed.
;;   :user::verdict-wall-fails  — a deftest whose assertion CANNOT hold (2+2 = 4242). This is
;;                                the load-bearing one: before the wall, driving it through
;;                                `call_beside(...).is_ok()` returned Ok — a fired assertion
;;                                landed a Failure in the returned RunResult's slot and the
;;                                caller read "did it evaluate?" as "did it pass?". The verdict
;;                                verb must report Failed and carry the structured Failure.
;;   :user::plain-value         — a plain zero-arg fn (NOT a deftest). The value verb must hand
;;                                back its Value; the verdict verb must REFUSE it.
;;
;; The deliberately-false assertion here is the POINT of the fixture, not a bug: it is the
;; only shape that can prove a failure is unreadable as a pass. Do not "fix" it.

(:wat::test::deftest :user::verdict-wall-passes
  (:wat::test::assert-eq (:wat::i64::+ 2 2) 4))

(:wat::test::deftest :user::verdict-wall-fails
  (:wat::test::assert-eq (:wat::i64::+ 2 2) 4242))

(:wat::core::defn :user::plain-value [] -> :wat::core::i64
  (:wat::i64::+ 2 5))
