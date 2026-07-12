;; tests/wat_lang/probe_arc234_stone4_hash_destructure.wat
;; Arc 234 Stone 234.4 — let-binding hash-destructure probes (positive cases).
;; Probes 1, 2, 3, 4, 6. Probe 5 (unknown field) uses the .wat.bad fixture.

(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::core::defrecord :myapp::Triple
  [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool])
(:wat::core::defrecord :myapp::Counter [count <- :wat::core::i64])

;; Probe 1: single-field record destructure → f64(5.0)
(:wat::core::defn :t::probe1-single-field [] -> :wat::core::f64
  (:wat::core::let
      [{mag :magnitude} (:myapp::Voltage :magnitude 5.0)]
      mag))

;; Probe 2: multi-field record destructure → String("hello")
(:wat::core::defn :t::probe2-multi-field [] -> :wat::core::String
  (:wat::core::let
      [{x :a  y :b  z :c} (:myapp::Triple :a 7 :b "hello" :c true)]
      y))

;; Probe 3: HashMap destructure with present key (Some) → i64(8080)
(:wat::core::defn :t::probe3-hashmap-some [] -> :wat::core::i64
  (:wat::core::let
      [{p :port} {:port 8080}]
      (:wat::core::Option/expect p "probe 3: :port key present")))

;; Probe 4: HashMap destructure with missing key (None) → bool(true)
(:wat::core::defn :t::probe4-hashmap-none [] -> :wat::core::bool
  (:wat::core::let
      [{x :missing} {:host "localhost"}]
      (:wat::core::match x -> :wat::core::bool
        ((:wat::core::Some _) false)
        (:wat::core::None     true))))

;; Probe 6: multiple destructures in same let → f64(10.5)
(:wat::core::defn :t::probe6-multiple [] -> :wat::core::f64
  (:wat::core::let
      [{m :magnitude} (:myapp::Voltage :magnitude 3.5)
       {c :count}     (:myapp::Counter :count 7)]
      (:wat::core::+ m (:wat::core::i64/to-f64 c))))
