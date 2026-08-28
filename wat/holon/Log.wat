;; vigilatum: 2026-06-04T06:49:40Z — vigilia 4-spell L1+L2=0, checker-clean + deftest-green(ReciprocalLog)
;;
;; :wat::holon::Log — stdlib macro for log-scale range encoding.
;;
;; (Log value min max) expands to (Thermometer (ln value) (ln min) (ln max)) —
;; log-transform the inputs, then encode the result with Thermometer
;; (the locality-preserving gradient primitive).
;; Natural log is conventional; any base cancels because the encoding is
;; by ratio. Callers guarantee positive inputs (user responsibility —
;; trading-lab callers use `.max(0.0001)` guards).

(:wat::core::defmacro :wat::holon::Log
  [value <- :wat::WatAST
   min   <- :wat::WatAST
   max   <- :wat::WatAST]
  -> :wat::WatAST
  `(:wat::holon::Thermometer
     (:wat::math::ln ~value)
     (:wat::math::ln ~min)
     (:wat::math::ln ~max)))
