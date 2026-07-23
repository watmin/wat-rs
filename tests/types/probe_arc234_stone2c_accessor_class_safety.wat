;; tests/types/probe_arc234_stone2c_accessor_class_safety.wat
;; Co-located fixture for probe_arc234_stone2c_accessor_class_safety.rs (arc 234 Stone 234.2c).
;; Panic probes (2-4) load this world and then call accessors with wrong-class receivers.

(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::core::defrecord :myapp::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :myapp::Triple [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
(:wat::core::defrecord :myapp::Other [x <- :wat::core::i64])

;; ─── Probe 1: correct-class accessor returns value ────────────────────────────
(:wat::core::defn :user::probe-1 [] -> :wat::core::f64
  (:wat::core::let [v (:myapp::Voltage :magnitude 42.5)] (:myapp::Voltage/magnitude v)))

;; ─── Probe 2+3: wrong-class receiver panics ──────────────────────────────────
;; Evaluating :user::probe-23 calls Voltage/magnitude on a Point → should panic.
(:wat::core::defn :user::probe-23 [] -> :wat::core::f64
  (:wat::core::let [p (:myapp::Point :x 3 :y 4)] (:myapp::Voltage/magnitude p)))

;; ─── Probe 4: multi-field each accessor checks class ─────────────────────────
;; Evaluating :user::probe-4 calls Triple/b on an Other → should panic.
(:wat::core::defn :user::probe-4 [] -> :wat::core::String
  (:wat::core::let [o (:myapp::Other :x 99)] (:myapp::Triple/b o)))

;; ─── Probe 5: predicate-gated pattern avoids panic ───────────────────────────
(:wat::core::defn :user::probe-5 [] -> :wat::core::f64
  (:wat::core::let [p (:myapp::Point :x 3 :y 4)]
    (:wat::core::if
      (:myapp::is-Voltage? p)
      
      (:myapp::Voltage/magnitude p)
      -1.0)))
