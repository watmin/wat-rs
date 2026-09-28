;; struct_destructure_single_field.wat — single field keys-destructure.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- wat.type/String
   grace-residue <- wat.type/f64])
(:wat::core::defn :user::compute [] -> wat.type/String
  (:wat::core::let
    [p (:test::PaperResolved :outcome "Grace" :grace-residue 7.5)
     {:keys [outcome]} p]
    outcome))
