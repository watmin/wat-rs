;; Twin of where-or-and.wat. Clara 0.24.0 test-disjunction-with-nested-and.
;; n= is DISTINCT locations.

(ns where-or-and
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Hit  [loc])

(defrule really-cold-or-cold-and-windy
  [:or
   [Temp (= ?loc loc) (< c 0)]
   [:and
    [Temp (= ?loc loc) (< c 20)]
    [Wind (= ?loc loc) (> kph 30)]]]
  => (insert! (->Hit ?loc)))

(defquery hit-q [] [Hit (= ?loc loc)])

(defn n-locs [facts]
  (count (set (map :?loc (query (fire-rules (apply insert (mk-session [really-cold-or-cold-and-windy hit-q] :cache false) facts)) hit-q)))))

(defn -main [& _]
  (prn (str "row 1 really-cold n=" (n-locs [(->Temp -10 "MCI")])))
  (prn (str "row 2 cold-and-windy n=" (n-locs [(->Temp 15 "MCI") (->Wind 50 "MCI")])))
  (prn (str "row 3 mild-only n=" (n-locs [(->Temp 15 "MCI")])))
  (prn (str "row 4 wind-only n=" (n-locs [(->Wind 50 "MCI")])))
  (prn (str "row 5 really-cold-and-windy n=" (n-locs [(->Temp -10 "MCI") (->Wind 50 "MCI")])))
  (prn (str "row 6 hot-and-windy n=" (n-locs [(->Temp 25 "MCI") (->Wind 50 "MCI")]))))

(-main)
