;; tests/types/probe_arc234_stone2b_defrecord_macro.wat
;; Co-located fixture for probe_arc234_stone2b_defrecord_macro.rs (arc 234 Stone 234.2b).

(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::core::defrecord :myapp::Counter [count <- :wat::core::i64])
(:wat::core::defrecord :myapp::Triple [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
(:wat::core::defrecord :myapp::Tag [])

;; ─── Probe 1: single-field construction returns :wat::core::Record ─────────────────
(:wat::core::defn :user::probe-1 [] -> :wat::core::Record (:myapp::Voltage :magnitude 5.0))

;; ─── Probe 2: per-field accessor returns value ───────────────────────────────
(:wat::core::defn :user::probe-2 [] -> :wat::core::f64
  (:wat::core::let [v (:myapp::Voltage :magnitude 42.5)] (:myapp::Voltage/magnitude v)))

;; ─── Probe 3: predicate true on matching class ────────────────────────────────
(:wat::core::defn :user::probe-3 [] -> :wat::core::bool
  (:wat::core::let [v (:myapp::Voltage :magnitude 5.0)] (:myapp::is-Voltage? v)))

;; ─── Probe 4: predicate false on non-matching class ──────────────────────────
(:wat::core::defn :user::probe-4 [] -> :wat::core::bool
  (:wat::core::let [c (:myapp::Counter :count 42)] (:myapp::is-Voltage? c)))

;; ─── Probe 5: multi-field accessors in order ─────────────────────────────────
(:wat::core::defn :user::probe-5-a [] -> :wat::core::i64
  (:wat::core::let [t (:myapp::Triple :a 7 :b "hello" :c true)] (:myapp::Triple/a t)))
(:wat::core::defn :user::probe-5-b [] -> :wat::core::String
  (:wat::core::let [t (:myapp::Triple :a 7 :b "hello" :c true)] (:myapp::Triple/b t)))
(:wat::core::defn :user::probe-5-c [] -> :wat::core::bool
  (:wat::core::let [t (:myapp::Triple :a 7 :b "hello" :c true)] (:myapp::Triple/c t)))
(:wat::core::defn :user::probe-5 [] -> :wat::core::String
  (:wat::core::let
    [a (:user::probe-5-a)
     b (:user::probe-5-b)
     c (:user::probe-5-c)]
    (:wat::string::concat
      (:wat::i64::to-string a)
      "|"
      b
      "|"
      (:wat::core::bool::to-string c))))

;; ─── Probe 6: zero-field defrecord ───────────────────────────────────────────
(:wat::core::defn :user::probe-6 [] -> :wat::core::bool
  (:wat::core::let [t (:myapp::Tag)] (:myapp::is-Tag? t)))
