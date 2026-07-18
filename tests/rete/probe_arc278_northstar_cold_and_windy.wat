;; tests/rete/probe_arc278_northstar_cold_and_windy.wat — co-located fixture for the sibling probe (.rs),
;; slurped via startup_beside(file!()). The north-star defrule: cold-and-windy end-to-end DSL spec.

(:wat::core::defrecord :weather::Temperature [celsius  <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed    [kph      <- :wat::core::i64  location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(:weather::Temperature
     (?loc <- :location)
     (?c   <- :celsius)
     (:wat::core::< ?c 20))
   (:weather::WindSpeed
     (?loc <- :location)
     (?k   <- :kph)
     (:wat::core::> ?k 30))]
  :then
  (:wat::rete::insert (:weather::ColdAndWindy :location ?loc)))

;; The lifecycle, value-threaded: collect → compile → insert → insert → fire → query, then COUNT the
;; derived facts (wrapped in `length` so the Rust driver just-evals to a scalar).
(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::length
    (:wat::core::let
      [rules    (:wat::rete::collect-rules :weather)
       session  (:wat::rete::compile rules)
       session  (:wat::rete::insert session (:weather::Temperature :celsius 15 :location "Oslo"))
       session  (:wat::rete::insert session (:weather::WindSpeed    :kph 45 :location "Oslo"))
       fired    (:wat::rete::fire-rules session)]
      (:wat::rete::query-by-type-string fired "weather::ColdAndWindy"))))

