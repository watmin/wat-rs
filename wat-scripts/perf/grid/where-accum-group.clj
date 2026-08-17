;; Twin of where-accum-group.wat. Clara 0.24.0 unbound grouping + acc-first reorder.
;; n= is the number of distinct Busy locations.

(ns where-accum-group
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Busy [loc n])

(defrule count-by-loc
  [?n <- (acc/count) :from [Temp (= ?loc loc)]]
  => (insert! (->Busy ?loc ?n)))

(defrule acc-first-wind
  [?n <- (acc/count) :from [Temp (= ?loc loc)]]
  [Wind (> kph 10) (= ?loc loc)]
  => (insert! (->Busy ?loc ?n)))

(defquery busy-q [] [Busy (= ?loc loc) (= ?n n)])

(defn n-busy [rule facts]
  (let [s (mk-session [rule busy-q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map :?loc (query (fire-rules s) busy-q))))))

(defn -main [& _]
  (prn (str "row 1 two-locs n=" (n-busy count-by-loc [(->Temp 10 "MCI") (->Temp 20 "MCI") (->Temp 30 "ORD")])))
  (prn (str "row 2 empty-group n=" (n-busy count-by-loc [])))
  (prn (str "row 3 one-loc n=" (n-busy count-by-loc [(->Temp 10 "MCI") (->Temp 20 "MCI")])))
  (prn (str "row 4 acc-first-wind-empty-temp n=" (n-busy acc-first-wind [(->Wind 20 "MCI")])))
  (prn (str "row 5 acc-first-two-winds n=" (n-busy acc-first-wind [(->Wind 20 "MCI") (->Wind 20 "SFO") (->Temp 40 "SFO") (->Temp 50 "SFO")]))))

(-main)
