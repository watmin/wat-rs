;; Twin of where-not-windy.wat. Clara 0.24.0 test-negation-with-other-conditions.
;; A cold Temp anywhere kills every windy loc.

(ns where-not-windy
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Hit  [loc])

(defrule windy-not-cold
  [Wind (= ?loc loc) (= ?w kph)]
  [:test (> ?w 30)]
  [:not [Temp (< c 20)]]
  => (insert! (->Hit ?loc)))

(defquery hit-q [] [Hit (= ?loc loc)])

(defn n-locs [facts]
  (let [s (mk-session [windy-not-cold hit-q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map :?loc (query (fire-rules s) hit-q))))))

(defn -main [& _]
  (prn (str "row 1 wind-only n=" (n-locs [(->Wind 40 "MCI")])))
  (prn (str "row 2 wind-hot n=" (n-locs [(->Wind 40 "MCI") (->Temp 80 "MCI")])))
  (prn (str "row 3 wind-cold n=" (n-locs [(->Wind 40 "MCI") (->Temp 10 "MCI")])))
  (prn (str "row 4 calm n=" (n-locs [(->Wind 20 "MCI")])))
  (prn (str "row 5 two-locs n=" (n-locs [(->Wind 40 "MCI") (->Wind 50 "ORD")])))
  (prn (str "row 6 two-locs-one-cold n=" (n-locs [(->Wind 40 "MCI") (->Wind 50 "ORD") (->Temp 10 "DFW")]))))

(-main)
