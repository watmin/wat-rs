;; 255.7 — :then insert heads accept both spellings. Special forms stay rust-scheme.
(wat.core/defrecord weather/Temperature [celsius :- wat.type/i64 location :- wat.type/String])
(wat.core/defrecord weather/WindSpeed [kph :- wat.type/i64 location :- wat.type/String])
(wat.core/defrecord weather/ColdAndWindy [location :- wat.type/String])

(wat.rete/defrule weather/cold-and-windy
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
  [(weather/ColdAndWindy :location ?loc)])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.kernel/println "ok"))
