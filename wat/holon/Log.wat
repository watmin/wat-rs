;; :wat::holon::Log — stdlib macro per 058-017.
;;
;; (Log value min max) expands to (Thermometer (ln value) (ln min) (ln max)) —
;; log-transform the inputs, then encode the result with Thermometer
;; (the locality-preserving gradient primitive; see
;; eval_algebra_thermometer in runtime.rs for attribution and the
;; substrate-level role).
;; Natural log is conventional; any base cancels because the encoding is
;; by ratio. Callers guarantee positive inputs (user responsibility —
;; trading-lab callers use `.max(0.0001)` guards).

(:wat::core::defmacro
  (:wat::holon::Log
    (value :AST<f64>)
    (min :AST<f64>)
    (max :AST<f64>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::holon::Thermometer
     (:wat::std::math::ln ,value)
     (:wat::std::math::ln ,min)
     (:wat::std::math::ln ,max)))
