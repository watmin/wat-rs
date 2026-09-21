;; 255.6 — rete clause heads and fact-bind types accept both spellings.
;; Special forms stay rust-scheme (this is a parser stone, not 8d-ii).
;; :when clauses are the converted dialect: (Type/Name …) and (?fact :- ns/Type).
(:wat::core::defrecord :weather::Temperature [celsius <- :wat::core::i64 location <- :wat::core::String])
(:wat::core::defrecord :weather::WindSpeed [kph <- :wat::core::i64 location <- :wat::core::String])
(:wat::core::defrecord :weather::ColdAndWindy [location <- :wat::core::String])

(:wat::rete::defrule :weather::cold-and-windy
  :when
  [(weather/Temperature
     (?loc :- :location)
     (?c :- :celsius)
     (wat.rete.i64/< ?c 20))
   (weather/WindSpeed
     (?loc :- :location)
     (?k :- :kph)
     (wat.rete.i64/> ?k 30))]
  :then
  [(:weather::ColdAndWindy :location ?loc)])

(:wat::rete::defquery :weather::q-ColdAndWindy
  :params []
  :when [(?fact :- weather/ColdAndWindy)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "ok"))
