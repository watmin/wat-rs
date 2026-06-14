;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(Amplify)
;;
;; :wat::holon::Amplify — scaled component emphasis.
;;
;; (Amplify x y s) expands to (Blend x y 1 s): anchor x at unit
;; emphasis, scale y's contribution by s. `s` is a runtime :wat::core::f64
;; expression — the Blend weights are literal at hash time because the
;; macro expansion commits the `1` and captures whatever the caller
;; wrote for `s`.

(:wat::core::defmacro :wat::holon::Amplify
  [x <- :wat::WatAST
   y <- :wat::WatAST
   s <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::holon::Blend ~x ~y 1.0 ~s))
