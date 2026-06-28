;; tests/types/probe_arc234_stone3c_fix_narrow_fallthrough.wat
;; Co-located fixture for probe_arc234_stone3c_fix_narrow_fallthrough.rs (arc 234 Stone 234.3c.fix).
;; Positive cases (probes 2, 3, 4): startup must succeed.
;; Probe 1 (negative) uses the sibling _p1_bad.wat file.

(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])

;; Probe 2: record receiver keyword accessor
(:wat::core::defn :user::probe-2 [] -> :wat::core::f64
  (:wat::core::let [v (:myapp::Voltage 5.0)] (:magnitude v)))

;; Probe 3: hashmap receiver keyword accessor
(:wat::core::defn :user::probe-3 [] -> :wat::core::i64
  (:wat::core::let [p (:port {:port 8080})]
    (:wat::core::Option/expect p "probe 3: :port key present")))

;; Probe 4: polymorphic receiver (record-typed param)
(:wat::core::defn :user::pluck [v <- :wat::Record] -> :wat::core::f64 (:magnitude v))
(:wat::core::defn :user::probe-4 [] -> :wat::core::f64 (:user::pluck (:myapp::Voltage 7.0)))
