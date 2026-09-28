;; struct_destructure_field_order.wat — field order can differ from declaration.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- wat.type/String
   grace-residue <- wat.type/f64])
(:wat::core::defn :user::compute [] -> wat.type/String
  (:wat::core::let
    [p (:test::PaperResolved :outcome "Grace" :grace-residue 5.5)
     {:keys [grace-residue outcome]} p]
    outcome))
