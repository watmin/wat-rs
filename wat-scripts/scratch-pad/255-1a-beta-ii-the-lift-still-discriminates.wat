;; Arc 255 Stone 1a-β-ii — ⛔ STOP-4, run against the NEW binary.
;;
;; `split_body_prelude` now asks the registry instead of a hand-list. A registry query
;; that returned `true` for everything would pass every test that only checks the
;; POSITIVE case, so both directions are read here through `:wat::kernel::fn-forms`,
;; which reifies a closure through `extract_closure` → `split_body_prelude`.
;;
;;   a body whose prefix IS a declaration   → the prelude lifts   (2 forms)
;;   a body whose first form is NOT one     → nothing lifts       (1 form)
(:wat::core::def :scratch::lifts
  (:wat::core::fn [] -> :wat::core::i64
    (:wat::core::do
      (:wat::core::defenum :scratch::Colour :wat::enum::Pure :Red)
      5)))

(:wat::core::def :scratch::does-not-lift
  (:wat::core::fn [] -> :wat::core::i64
    (:wat::core::do
      5
      (:wat::core::defenum :scratch::Shade :wat::enum::Pure :Dark))))

(:wat::core::def :user::main
  (:wat::core::fn [] -> :wat::core::nil
    (:wat::kernel::println
      (:wat::core::length (:wat::kernel::fn-forms :scratch::lifts :probe-a)))
    (:wat::kernel::println
      (:wat::core::length (:wat::kernel::fn-forms :scratch::does-not-lift :probe-b)))))
