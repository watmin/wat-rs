;; Twin of where-query-params.wat. Clara 0.24.0 test-count-some-empty.

(ns where-query-params
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Hit  [loc])

(defrule mark
  [Wind (= ?loc loc) (> kph 10)]
  => (insert! (->Hit ?loc)))

(defquery temps-at [:?loc]
  [?n <- (acc/count) :from [Temp (= ?loc loc)]]
  [Wind (> kph 10) (= ?loc loc)])

(defquery all-wind []
  [Wind (= ?loc loc)])

(defquery hit-q [] [Hit (= ?loc loc)])

(defn world [facts]
  (let [s (mk-session [mark temps-at all-wind hit-q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (fire-rules s)))

(def facts
  [(->Wind 20 "MCI") (->Wind 20 "SFO") (->Temp 40 "SFO") (->Temp 50 "SFO")])

(defn -main [& _]
  (let [s (world facts)
        e (world [])]
    (prn (str "row 1 hits n=" (count (query s hit-q))))
    (prn (str "row 2 mci-empty-temps n=" (count (query s temps-at :?loc "MCI"))))
    (prn (str "row 3 sfo-two-temps n=" (count (query s temps-at :?loc "SFO"))))
    (prn (str "row 4 missing-loc n=" (count (query s temps-at :?loc "XXX"))))
    (prn (str "row 5 all-wind n=" (count (query s all-wind))))
    (prn (str "row 6 empty-world n=" (count (query e temps-at :?loc "MCI"))))))

(-main)
