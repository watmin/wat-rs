;; struct_destructure_multi_field.wat — multiple field keys-destructure.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- wat.type/String
   grace-residue <- wat.type/f64])
(:wat::core::defn :user::compute [] -> wat.type/f64
  (:wat::core::let
    [p (:test::PaperResolved :outcome "Grace" :grace-residue 7.5)
     {:keys [outcome grace-residue]} p]
    grace-residue))
