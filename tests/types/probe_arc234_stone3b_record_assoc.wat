;; tests/types/probe_arc234_stone3b_record_assoc.wat
;; Co-located fixture for probe_arc234_stone3b_record_assoc.rs (arc 234 Stone 234.3b).

(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::core::defrecord :myapp::Triple [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])

;; ─── Probe 1: single-field update ────────────────────────────────────────────
(:wat::core::defn :user::probe-1 [] -> :wat::core::f64
  (:wat::core::let
    [r  (:myapp::Voltage :magnitude 5.0)
     r2 (:wat::core::Record/assoc r :magnitude 6.0)]
    (:myapp::Voltage/magnitude r2)))

;; ─── Probe 2: multi-field update one ─────────────────────────────────────────
(:wat::core::defn :user::probe-2 [] -> :wat::core::String
  (:wat::core::let
    [t  (:myapp::Triple :a 7 :b "hello" :c true)
     t2 (:wat::core::Record/assoc t :b "world")]
    (:myapp::Triple/b t2)))

;; ─── Probe 3: unknown field errors ───────────────────────────────────────────
(:wat::core::defn :user::probe-3 [] -> :wat::core::Record
  (:wat::core::let [t (:myapp::Triple :a 7 :b "hello" :c true)]
    (:wat::core::Record/assoc t :nonexistent 42)))

;; ─── Probe 4: type mismatch errors ───────────────────────────────────────────
(:wat::core::defn :user::probe-4 [] -> :wat::core::Record
  (:wat::core::let [r (:myapp::Voltage :magnitude 5.0)]
    (:wat::core::Record/assoc r :magnitude 42)))

;; ─── Probe 5: original record unchanged ──────────────────────────────────────
(:wat::core::defn :user::probe-5 [] -> :wat::core::f64
  (:wat::core::let
    [r1 (:myapp::Voltage :magnitude 5.0)
     r2 (:wat::core::Record/assoc r1 :magnitude 6.0)]
    (:myapp::Voltage/magnitude r1)))

;; ─── Probe 6: compose multiple assocs ────────────────────────────────────────
(:wat::core::defn :user::probe-6 [] -> :wat::core::String
  (:wat::core::let
    [t  (:myapp::Triple :a 7 :b "hello" :c true)
     t2 (:wat::core::Record/assoc
          (:wat::core::Record/assoc t :a 100)
          :b "world")]
    (:wat::string::concat
      (:wat::core::i64::to-string (:myapp::Triple/a t2))
      "|"
      (:myapp::Triple/b t2))))
