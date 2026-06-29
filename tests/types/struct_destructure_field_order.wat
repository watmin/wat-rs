;; struct_destructure_field_order.wat — field order can differ from declaration.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [p (:test::PaperResolved "Grace" 5.5)
     {:keys [grace-residue outcome]} p]
    outcome))
