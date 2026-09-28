(:wat::core::newtype :u::T :wat::core::i64)
(:wat::core::extend-type :u::T :wat::core::Orderable)
(:wat::core::defstruct :u::St [n <- wat.type/i64])
(:wat::core::defenum :u::Imp :wat::enum::Impure :A :B)
(:wat::core::defenum :u::Pure :wat::enum::Pure :A :B)
(:wat::core::defn :user::ord-nt [] -> wat.type/bool
  (:wat::core::< (:u::T 1) (:u::T 2)))
(:wat::core::defn :user::eq-nil [] -> wat.type/bool
  (:wat::core::= nil nil))
(:wat::core::defn :user::num [] -> wat.type/bool
  (:wat::core::< 1 2.0))
(:wat::core::defn :user::as-enum [x <- :u::Pure] -> :u::Pure x)
(:wat::core::defn :user::variant [] -> wat.type/bool
  (:wat::core::= (:user::as-enum (:u::Pure.A {})) (:u::Pure.A {})))
