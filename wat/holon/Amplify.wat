;; :wat::holon::Amplify — scaled component emphasis per 058-015.
;;
;; (Amplify x y s) expands to (Blend x y 1 s): anchor x at unit
;; emphasis, scale y's contribution by s. `s` is a runtime :wat::core::f64
;; expression — the Blend weights are literal at hash time because the
;; macro expansion commits the `1` and captures whatever the caller
;; wrote for `s`.

(:wat::core::defmacro
  (:wat::holon::Amplify
    (x :AST<wat::holon::HolonAST>)
    (y :AST<wat::holon::HolonAST>)
    (s :AST<f64>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::holon::Blend ,x ,y 1.0 ,s))
