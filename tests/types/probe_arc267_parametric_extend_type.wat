;; tests/types/probe_arc267_parametric_extend_type.wat — co-located fixture
;;
;; Arc 267 — parametric extend-type / parametric protocol bounds.

(:wat::core::defstruct :t::Box<T> [val <- :T])

(:wat::core::defprotocol :t::Tagged
  (tag [self <- :t::Tagged] -> :wat::core::String))

(:wat::core::extend-type :t::Box :t::Tagged (tag [self] "box"))

(:wat::core::defn :user::tag-of [x <- :t::Tagged] -> :wat::core::String
  (:t::Tagged/tag x))

(:wat::core::defn :user::go [] -> :wat::core::String
  (:user::tag-of (:t::Box/new 5)))

