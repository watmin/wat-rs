;; tests/diagnostics/probe_ex003_stone_d3_frames.wat — co-located fixture for
;; probe_ex003_stone_d3_frames.rs — excursus 003, envelope step 2 (D3).
;;
;; Four fixtures, one per "Prove it" row in
;; BRIEF-envelope-step-2-every-error-carries-its-frames.md:
;;   (a) three-deep-raise:      :user::outer -> :user::middle -> :user::inner -> DivisionByZero.
;;   (b) tail-loop:             1,000,000 tail-recursive iterations, then raises — CALL_STACK
;;                               must stay flat (replace_top_frame), not 1,000,000 deep.
;;   (c) non-tail-deep:         300 NON-tail-recursive calls (each wrapped in `i64::+`, so the
;;                               recursive call is never in tail position), then raises — deep
;;                               enough to exceed the 40-frame cap. NOT anywhere near the
;;                               brief's ~110,000-frame ceiling (the-little-wat F-099): MEASURED
;;                               on this harness (nextest gives each test its own, smaller-than-
;;                               main thread stack) that 5,000 already overflows the Rust stack
;;                               through this interpreter's own eval/apply_function recursion —
;;                               see the depth constant's own comment below.
;;   (d) assert-deep:           a deftest whose assert-eq fails three user-calls deep — proves
;;                               AssertionPayload's frames use the same Frame shape.

;; `+ 0 (call)` (not a bare tail call) so each of these three calls pushes its OWN
;; CALL_STACK frame instead of tail-call-collapsing into one (`replace_top_frame`) —
;; the three-deep case needs three DISTINCT frames, not TCO'd flatness (that is what
;; the tail-loop fixture below proves instead).
(:wat::core::defn :user::inner [] -> :wat::core::i64
  (:wat::i64::/ 1 0))

(:wat::core::defn :user::middle [] -> :wat::core::i64
  (:wat::i64::+ 0 (:user::inner)))

(:wat::core::defn :user::outer [] -> :wat::core::i64
  (:wat::i64::+ 0 (:user::middle)))

(:wat::core::defn :user::tail-loop [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0)
    (:wat::i64::/ 1 0)
    (:user::tail-loop (:wat::i64::- n 1))))

(:wat::core::defn :user::tail-loop-entry [] -> :wat::core::i64
  (:user::tail-loop 1000000))

(:wat::core::defn :user::non-tail-loop [n <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= n 0)
    (:wat::i64::/ 1 0)
    (:wat::i64::+ 1 (:user::non-tail-loop (:wat::i64::- n 1)))))

;; 300, not the brief's ~110,000-frame overflow ceiling: MEASURED on this harness (nextest
;; runs each test on its own thread, a smaller stack than main's) that non-tail recursion
;; through this interpreter's own `eval`/`apply_function` overflows the Rust stack far below
;; 110,000 — 5,000 already `SIGABRT`'d here. 300 is comfortably above the 40-frame cap
;; (proves capping) and comfortably below the observed overflow floor.
(:wat::core::defn :user::non-tail-loop-entry [] -> :wat::core::i64
  (:user::non-tail-loop 300))

(:wat::core::defn :user::assert-inner [] -> :wat::core::nil
  (:wat::test::assert-eq 1 2))

(:wat::core::defn :user::assert-middle [] -> :wat::core::nil
  (:user::assert-inner))

(:wat::test::deftest :user::assert-deep
  (:user::assert-middle))
