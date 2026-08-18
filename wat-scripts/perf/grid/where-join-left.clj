;; Twin of where-join-left.wat. Clara 0.24.0.
;; HashJoin whose right cond names a left-bound var: (> ?w ?c).

(ns where-join-left
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Hit  [loc])

(defrule wind-above-temp-inline
  [Temp (= ?loc loc) (= ?c c)]
  [Wind (= ?loc loc) (= ?w kph) (> ?w ?c)]
  => (insert! (->Hit ?loc)))

(defrule wind-above-temp-where
  [Temp (= ?loc loc) (= ?c c)]
  [Wind (= ?loc loc) (= ?w kph)]
  [:test (> ?w ?c)]
  => (insert! (->Hit ?loc)))

(defquery hit-q [] [Hit (= ?loc loc)])

(defn n-hit [rules facts]
  (let [s (mk-session (conj (vec rules) hit-q) :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (query (fire-rules s) hit-q))))

(defn -main [& _]
  (let [inline [wind-above-temp-inline]
        where  [wind-above-temp-where]]
    (prn (str "row 1 empty-inline n=" (n-hit inline [])))
    (prn (str "row 2 temp-only-inline n=" (n-hit inline [(->Temp 10 "MCI")])))
    (prn (str "row 3 wind-only-inline n=" (n-hit inline [(->Wind 20 "MCI")])))
    (prn (str "row 4 below-inline n=" (n-hit inline [(->Temp 10 "MCI") (->Wind 5 "MCI")])))
    (prn (str "row 5 above-inline n=" (n-hit inline [(->Temp 10 "MCI") (->Wind 20 "MCI")])))
    (prn (str "row 6 equal-inline n=" (n-hit inline [(->Temp 10 "MCI") (->Wind 10 "MCI")])))
    (prn (str "row 7 below-where n=" (n-hit where [(->Temp 10 "MCI") (->Wind 5 "MCI")])))
    (prn (str "row 8 above-where n=" (n-hit where [(->Temp 10 "MCI") (->Wind 20 "MCI")])))
    (prn (str "row 9 two-locs-inline n=" (n-hit inline [(->Temp 10 "MCI") (->Wind 20 "MCI")
                                                       (->Temp 10 "ORD") (->Wind 5 "ORD")])))))

(-main)
