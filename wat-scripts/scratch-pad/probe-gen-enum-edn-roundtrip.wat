;; Disconfirming probe for BRIEF 2, second half — ENUMS. Different EDN mechanism from a
;; record: `#ns/Variant [body]` for a tagged variant and `#ns/Variant []` for a unit one.
;; The unit variant is the interesting case; arc 278 A.0 gave it its own encoding.
(:wat::core::defenum :probe::Shape :wat::enum::Pure
  :Dot   []
  :Line  [len <- :wat::core::i64]
  :Rect  [w <- :wat::core::i64  h <- :wat::core::i64])

(:wat::core::defn :probe::law [s <- :probe::Shape] -> :wat::core::bool
  (:wat::core::= (:wat::edn::read (:wat::edn::write s)) s))

;; one generator per variant, unioned — `one-of` is the union combinator
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [dots  (:wat::gen::gen 1 (:wat::core::fn [_i <- :wat::core::i64] -> :probe::Shape
                               (:probe::Shape::Dot)))
     lines (:wat::gen::fmap (:wat::core::fn [n <- :wat::core::i64] -> :probe::Shape
                              (:probe::Shape::Line n))
                            (:wat::gen::ints 0 3))
     g     (:wat::gen::one-of (:wat::core::PersistentVector dots lines))
     o     (:wat::gen::check g :probe::law)]
    (:wat::kernel::println (:wat::edn::write o))))
