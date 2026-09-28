;; struct_destructure_hyphenated_field.wat — hyphenated field name binds correctly.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- wat.type/String
   grace-residue <- wat.type/f64])
(:wat::core::defn :user::compute [] -> wat.type/f64
  (:wat::core::let
    [p (:test::PaperResolved :outcome "Grace" :grace-residue 9.25)
     {:keys [grace-residue]} p]
    grace-residue))
