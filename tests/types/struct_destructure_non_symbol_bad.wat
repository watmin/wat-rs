;; struct_destructure_non_symbol_bad.wat — non-symbol in map binder. Must FAIL.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [p (:test::PaperResolved "Grace" 5.5)
     {42} p]
    "unreachable"))
