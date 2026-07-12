;; tests/types/probe_arc234_stone5_holon_auto_dispatch.wat
;; Co-located fixture for probe_arc234_stone5_holon_auto_dispatch.rs (arc 234 Stone 234.5).
;; Uses :wat::holon::defrecord (the holon-flavored macro).

(:wat::holon::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

;; ─── Probe 1: to-holon returns holon_form ────────────────────────────────────
(:wat::core::defn :user::probe-1 [] -> :wat::holon::HolonAST
  (:wat::core::let [v (:myapp::Voltage :magnitude 5.0)] (:wat::holon::to-holon v)))

;; ─── Probe 2: cosine accepts records ─────────────────────────────────────────
(:wat::core::defn :user::probe-2 [] -> :wat::core::f64
  (:wat::core::let
    [r1 (:myapp::Voltage :magnitude 5.0)
     r2 (:myapp::Voltage :magnitude 5.0)]
    (:wat::holon::cosine r1 r2)))

;; ─── Probe 3: Bind accepts record as right arg ───────────────────────────────
(:wat::core::defn :user::probe-3 [] -> :wat::holon::HolonAST
  (:wat::core::let [r (:myapp::Voltage :magnitude 5.0)]
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "wrapper"))
      r)))

;; ─── Probe 4: Bundle accepts records as children ─────────────────────────────
(:wat::core::defn :user::probe-4 [] -> :wat::holon::HolonAST
  (:wat::core::let
    [r1 (:myapp::Voltage :magnitude 1.0)
     r2 (:myapp::Voltage :magnitude 2.0)
     r3 (:myapp::Voltage :magnitude 3.0)]
    (:wat::core::Result/expect
      (:wat::holon::Bundle [r1 r2 r3])
      "Bundle failed in Probe 4")))

;; ─── Probe 5: extract-classifier on record ───────────────────────────────────
(:wat::core::defn :user::probe-5 [] -> :wat::core::String
  (:wat::core::let [r (:myapp::Voltage :magnitude 5.0)]
    (:wat::holon::extract-classifier r)))

;; ─── Probe 6: mixed records and holon ASTs ───────────────────────────────────
(:wat::core::defn :user::probe-6 [] -> :wat::holon::HolonAST
  (:wat::core::let
    [r          (:myapp::Voltage :magnitude 5.0)
     classifier (:wat::holon::Atom (:wat::holon::to-holon "wrapper"))]
    (:wat::holon::Bind
      classifier
      (:wat::core::Result/expect
        (:wat::holon::Bundle
          [r
           (:wat::holon::Atom (:wat::holon::to-holon "marker"))])
        "Bundle failed in Probe 6"))))
