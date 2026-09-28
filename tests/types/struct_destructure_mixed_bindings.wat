;; struct_destructure_mixed_bindings.wat — regular + keys-destructure in one let.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- wat.type/String
   grace-residue <- wat.type/f64])
(:wat::core::defn :user::compute [] -> wat.type/f64
  (:wat::core::let
    [p (:test::PaperResolved :outcome "Grace" :grace-residue 3.5)
     whole p
     {:keys [outcome grace-residue]} whole]
    grace-residue))
