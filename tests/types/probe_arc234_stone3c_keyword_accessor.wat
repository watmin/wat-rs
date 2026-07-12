;; tests/types/probe_arc234_stone3c_keyword_accessor.wat
;; Co-located fixture for probe_arc234_stone3c_keyword_accessor.rs (arc 234 Stone 234.3c).

(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::core::defrecord :myapp::Triple [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
(:wat::core::defstruct :myapp::Point
  [x <- :wat::core::i64
   y <- :wat::core::i64])

;; ─── Probe 1: keyword accessor on single-field record ────────────────────────
(:wat::core::defn :user::probe-1 [] -> :wat::core::f64
  (:wat::core::let [v (:myapp::Voltage :magnitude 5.0)] (:magnitude v)))

;; ─── Probe 2: keyword accessor on multi-field record ─────────────────────────
(:wat::core::defn :user::probe-2 [] -> :wat::core::String
  (:wat::core::let [t (:myapp::Triple :a 7 :b "hello" :c true)] (:b t)))

;; ─── Probe 3: unknown field on record errors ──────────────────────────────────
(:wat::core::defn :user::probe-3 [] -> :wat::core::f64
  (:wat::core::let [v (:myapp::Voltage :magnitude 5.0)] (:nonexistent v)))

;; ─── Probe 4: keyword accessor on hashmap key present ────────────────────────
(:wat::core::defn :user::probe-4 [] -> :wat::core::i64
  (:wat::core::let
    [m {:port 8080}
     v (:port m)]
    (:wat::core::Option/expect v "probe 4: expected :port key present")))

;; ─── Probe 5: keyword accessor on hashmap key missing ────────────────────────
(:wat::core::defn :user::probe-5 [] -> :wat::core::bool
  (:wat::core::let
    [m {:host "localhost"}
     v (:missing m)]
    (:wat::core::match v -> :wat::core::bool
      ((:wat::core::Some _) false)
      (:wat::core::None     true))))

;; ─── Probe 6: keyword accessor on struct ─────────────────────────────────────
(:wat::core::defn :user::probe-6 [] -> :wat::core::i64
  (:wat::core::let
    [p (:wat::core::struct-new :myapp::Point 3 4)]
    (:x p)))
