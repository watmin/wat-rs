;; arc109-reap-the-twelve-row1-operators.wat — BRIEF-STONE-reap-the-twelve.md acceptance row 1.
;;
;; The operator names `:wat::core::<`, `:wat::core::>`, `:wat::core::>=` contain a REAL `<`/`>`
;; character and are the entire reason the twelve deleted strips carried a "don't slice an
;; operator" guard in the first place. `<-` and `->` are the binder/arrow characters used in
;; ordinary fn syntax (`[x <- :T]`, `[T1 T2 :-> R]`). This file proves all five still work with
;; the strips gone.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "row1a (:wat::core::< 1 2):")
    (:wat::kernel::println (:wat::core::show (:wat::core::< 1 2)))

    (:wat::kernel::println "row1b (:wat::core::> 2 1):")
    (:wat::kernel::println (:wat::core::show (:wat::core::> 2 1)))

    (:wat::kernel::println "row1c (:wat::core::>= 2 2):")
    (:wat::kernel::println (:wat::core::show (:wat::core::>= 2 2)))

    (:wat::kernel::println "row1d <- binder in an ordinary fn signature:")
    (:wat::kernel::println (:wat::core::show (:user::add-one 41)))
    nil))

(:wat::core::defn :user::add-one [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::+ x 1))
