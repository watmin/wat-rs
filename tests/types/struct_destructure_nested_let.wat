;; struct_destructure_nested_let.wat — outer destructure feeds inner let.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
    [p (:test::PaperResolved :outcome "Grace" :grace-residue 4.0)
     {:keys [outcome grace-residue]} p]
    (:wat::core::let
      [doubled (:wat::f64::* grace-residue 2.0)]
      doubled)))
