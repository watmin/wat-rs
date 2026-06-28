;; struct_destructure_hyphenated_field.wat — hyphenated field name binds correctly.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
    [p (:test::PaperResolved/new "Grace" 9.25)
     {:keys [grace-residue]} p]
    grace-residue))
