;; struct_destructure_unknown_field_bad.wat — unknown field in :keys. Must FAIL.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- :wat::core::String
   grace-residue <- :wat::core::f64])
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [p (:test::PaperResolved/new "Grace" 5.5)
     {:keys [nonexistent]} p]
    nonexistent))
