;; :wat::holon::ReciprocalLog — arc 034 stdlib macro.
;;
;; (ReciprocalLog n value) → Log with reciprocal bounds (1/n, n).
;;
;; Expands to:
;;   (Log value (/ 1.0 n) n)
;;   = (Thermometer (ln value) (ln (1/n)) (ln n))
;;   = (Thermometer (ln value) (-(ln n)) (ln n))
;;
;; ln-space symmetry is automatic: ln(1/n) = -ln(n). No taste-
;; anchored round-number bounds; log-symmetry falls out of the
;; reciprocal construction.
;;
;; Intended for ratio-valued indicators centered near value = 1.0
;; (rate-of-change, volume ratio, ATR ratio, variance ratio, any
;; close/prev-style quantity). Caller picks N from the family:
;;
;;   N = 2   → bounds (0.5, 2.0)     ±doubling
;;   N = 3   → bounds (1/3, 3.0)     ±tripling
;;   N = 10  → bounds (0.1, 10.0)    ±10x
;;
;; Smallest member (N=2) is the default for gently-volatile
;; ratios; larger N widens the gradient before saturation.
;;
;; Preconditions (per 058-017 Q2): n > 0, value > 0. Caller
;; enforces; Thermometer over `ln(non-positive)` produces
;; undefined behavior.
;;
;; Arc 034; named via `/gaze`.

(:wat::core::defmacro
  (:wat::holon::ReciprocalLog
    (n :AST<f64>)
    (value :AST<f64>)
    -> :AST<wat::holon::HolonAST>)
  `(:wat::holon::Log
     ,value
     (:wat::core::/ 1.0 ,n)
     ,n))
