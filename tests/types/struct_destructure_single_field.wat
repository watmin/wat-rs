;; struct_destructure_single_field.wat — single field keys-destructure.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [p (:test::PaperResolved/new "Grace" 7.5)
     {:keys [outcome]} p]
    outcome))
