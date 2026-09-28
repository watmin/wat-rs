;; tests/types/probe_arc234_stone3a_record_read_verbs.wat
;; Co-located fixture for probe_arc234_stone3a_record_read_verbs.rs (arc 234 Stone 234.3a).

(:wat::core::defrecord :myapp::Voltage [magnitude <- wat.type/f64])
(:wat::core::defrecord :myapp::Triple [a <- wat.type/i64  b <- wat.type/String  c <- wat.type/bool])
(:wat::core::defrecord :myapp::Tag [])

;; ─── Probe 1: record? true on a record ───────────────────────────────────────
(:wat::core::defn :user::probe-1 [] -> wat.type/bool
  (:wat::core::let [v (:myapp::Voltage :magnitude 5.0)] (:wat::core::record? v)))

;; ─── Probe 2: record? false on i64 ──────────────────────────────────────────
(:wat::core::defn :user::probe-2 [] -> wat.type/bool (:wat::core::record? 42))

;; ─── Probe 3: record->map single-field ───────────────────────────────────────
(:wat::core::defn :user::probe-3 [] -> wat.type/f64
  (:wat::core::let
    [v (:myapp::Voltage :magnitude 5.0)
     m (:wat::core::record->map v)]
    (:wat::core::Option/expect
      (:wat::core::get m :magnitude)
      "record->map probe 3: :magnitude key missing")))

;; ─── Probe 4: record->map multi-field heterogeneous ─────────────────────────
(:wat::core::defn :user::probe-4 [] -> wat.type/String
  (:wat::core::let
    [t (:myapp::Triple :a 7 :b "hello" :c true)
     m (:wat::core::record->map t)]
    (:wat::core::Option/expect
      (:wat::core::get m :b)
      "record->map probe 4: :b key missing")))

;; ─── Probe 5: record->map zero-field ─────────────────────────────────────────
(:wat::core::defn :user::probe-5 [] -> wat.type/bool
  (:wat::core::let
    [t (:myapp::Tag)
     m (:wat::core::record->map t)]
    (:wat::core::empty? m)))

;; ─── Probe 6: predicate-then-map composition ─────────────────────────────────
(:wat::core::defn :user::probe-6 [] -> wat.type/f64
  (:wat::core::let [v (:myapp::Voltage :magnitude 99.0)]
    (:wat::core::if
      (:wat::core::record? v)
      
      (:wat::core::Option/expect
        (:wat::core::get (:wat::core::record->map v) :magnitude)
        "probe 6: missing :magnitude")
      -1.0)))
