;; struct_destructure_multi_form_body.wat — destructure + multi-form body.
(:wat::core::defstruct :test::PaperResolved
  [outcome       <- wat.type/String
   grace-residue <- wat.type/f64])
(:wat::core::defn :user::compute [] -> wat.type/f64
  (:wat::core::let
    [p (:test::PaperResolved :outcome "Grace" :grace-residue 1.0)
     {:keys [outcome grace-residue]} p]
    (:wat::f64::+ grace-residue 99.0)
    (:wat::f64::+ grace-residue 50.0)
    (:wat::f64::+ grace-residue 41.0)))
