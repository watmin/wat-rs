;; Disconfirming probe for BRIEF 2 — CAN a generated record survive `read ∘ write`?
;; Ten lines that attempt exactly the thing the brief would assume. If this fails, the
;; brief is wrong and grok would be sent into a wall.
(:wat::core::defrecord :probe::Point [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :probe::law [p <- :probe::Point] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write p)) p))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [g (:wat::gen::record :probe::Point (:wat::gen::ints 0 3) (:wat::gen::ints 0 3))
     o (:wat::gen::check g :probe::law)]
    (:wat::kernel::println (:wat::edn::write o))))
