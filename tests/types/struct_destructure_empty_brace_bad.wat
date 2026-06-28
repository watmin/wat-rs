;; struct_destructure_empty_brace_bad.wat — empty map in binder position. Must FAIL.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [p (:test::PaperResolved/new "Grace" 5.5)
     {} p]
    "unreachable"))
