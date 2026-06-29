;; struct_destructure_mixed_bindings.wat — regular + keys-destructure in one let.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::f64
  (:wat::core::let
    [p (:test::PaperResolved "Grace" 3.5)
     whole p
     {:keys [outcome grace-residue]} whole]
    grace-residue))
