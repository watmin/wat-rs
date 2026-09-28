;; tests/wat_lang/probe_arc257_keys_destructure.wat
;; Arc 257.2 — {:keys [x y z]} destructure in binder position (positive cases).
;; Probe 3 (bare-symbol brace rejected) uses the .wat.bad fixture.

(:wat::core::defstruct :myapp::Voltage [magnitude <- wat.type/f64])
(:wat::core::defstruct :myapp::Triple
  [a <- wat.type/i64  b <- wat.type/String  c <- wat.type/bool])

;; Probe 1: single-field keys-destructure → f64(5.0)
(:wat::core::defn :t::probe1-single-field [] -> wat.type/f64
  (:wat::core::let
      [v (:myapp::Voltage :magnitude 5.0)
       {:keys [magnitude]} v]
      magnitude))

;; Probe 2: multi-field keys-destructure → String("hello")
(:wat::core::defn :t::probe2-multi-field [] -> wat.type/String
  (:wat::core::let
      [t (:myapp::Triple :a 7 :b "hello" :c true)
       {:keys [a b c]} t]
      b))
