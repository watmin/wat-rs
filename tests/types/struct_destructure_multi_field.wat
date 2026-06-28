;; struct_destructure_multi_field.wat — multiple field keys-destructure.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
    [p (:test::PaperResolved/new "Grace" 7.5)
     {:keys [outcome grace-residue]} p]
    grace-residue))
