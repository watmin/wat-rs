;; :wat::std::Amplify — scaled component emphasis per 058-015.
;;
;; (Amplify x y s) expands to (Blend x y 1 s): anchor x at unit
;; emphasis, scale y's contribution by s. `s` is a runtime :f64
;; expression — the Blend weights are literal at hash time because the
;; macro expansion commits the `1` and captures whatever the caller
;; wrote for `s`.

(:wat::core::defmacro
  (:wat::std::Amplify
    (x :AST<holon::HolonAST>)
    (y :AST<holon::HolonAST>)
    (s :AST<f64>)
    -> :AST<holon::HolonAST>)
  `(:wat::algebra::Blend ,x ,y 1.0 ,s))
