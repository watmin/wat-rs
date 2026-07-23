;; tests/wat_lang/probe_arc234_stone4_match_hash_destructure.wat
;; Arc 234 Stone 234.4.match — match-arm hash-destructure probes (all positive).

(:wat::core::defrecord :myapp::Voltage [magnitude <- :wat::core::f64])
(:wat::core::defrecord :myapp::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :myapp::Tag [label <- :wat::core::String])
(:wat::core::defrecord :myapp::Sensor [reading <- :wat::core::f64])

;; Probe 1: match record with single {var :field} → f64(7.5)
(:wat::core::defn :t::probe1-match-record-single [] -> :wat::core::f64
  (:wat::core::let
      [rec (:myapp::Voltage :magnitude 7.5)]
      (:wat::core::match rec 
        ({mag :magnitude} mag)
        (_ 0.0))))

;; Probe 2: match record with multi {var1 :f1 var2 :f2} → i64(7)
(:wat::core::defn :t::probe2-match-record-multi [] -> :wat::core::i64
  (:wat::core::let
      [pt (:myapp::Point :x 3 :y 4)]
      (:wat::core::match pt 
        ({px :x  py :y} (:wat::core::+ px py))
        (_ 0))))

;; Probe 3: HashMap match with single key (present → Some) → i64(9000)
(:wat::core::defn :t::probe3-hashmap-single-some [] -> :wat::core::i64
  (:wat::core::let
      [m {:port 9000}]
      (:wat::core::match m 
        ({p :port} (:wat::core::Option/expect p "probe 3: :port key present"))
        (_ 0))))

;; Probe 4: HashMap match multi-key → bool(true) when h=Some, mv=None
(:wat::core::defn :t::probe4-hashmap-multi [] -> :wat::core::bool
  (:wat::core::let
      [m {:host "localhost"  :user "admin"}]
      (:wat::core::match m 
        ({h :host  mv :missing}
         (:wat::core::match h 
           ((:wat::core::Some _)
            (:wat::core::match mv 
              ((:wat::core::Some _) false)
              (:wat::core::None     true)))
           (:wat::core::None false)))
        (_ false))))

;; Probe 5: fall-through — i64 scrutinee with hash-destructure first arm falls to wildcard → i64(99)
(:wat::core::defn :t::probe5-fall-through [] -> :wat::core::i64
  (:wat::core::let
      [v 42]
      (:wat::core::match v 
        ({lbl :label} 0)
        (_ 99))))

;; Probe 6: mixed match — Sensor record → "record-matched"
(:wat::core::defn :t::probe6-compute-from-record [] -> :wat::core::String
  (:wat::core::let
      [s (:myapp::Sensor :reading 3.14)]
      (:wat::core::match s 
        ({r :reading} "record-matched")
        (_ "wildcard"))))

(:wat::core::defn :t::probe6-mixed [] -> :wat::core::String (:t::probe6-compute-from-record))
