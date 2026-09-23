;; Arc 251.8d-ii (EIGHTH DRAW) — THE CONTROL, in its own file ON PURPOSE.
;;
;; The identical `Type/member` macro shape declared in the KEYWORD surface. It must be
;; green on BOTH binaries — the cure widens a SPELLING, it may not move a keyword-spelled
;; program. It lives apart from the cure row because the cure row's fixture does not
;; FREEZE pre-cure: a control sharing that file would be unreadable on the binary it
;; exists to measure.
(:wat::core::defrecord :user::Crate [n <- :wat::core::i64])

(:wat::core::defmacro :user::Crate/of [n <- :wat::WatAST] -> :wat::WatAST
  `(:user::Crate :n ~n))

(:wat::core::defn :user::control [] -> :wat::core::i64
  (:user::Crate/n (:user::Crate/of 9)))
