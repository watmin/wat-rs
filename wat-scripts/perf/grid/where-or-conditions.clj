;; Twin of where-or-conditions.wat. Clara 0.24.0 condition :or.
;; n= is DISTINCT locations (Clara insert!s twice when both arms match).
;;
;; Rows 1–3: trailing :or. 4–7: prefix then :or. 8–11: :or then fact.
;; 12–14: prefix + :or + :test (wat :where).

(ns where-or-conditions
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Temp    [c loc])
(defrecord Wind    [kph loc])
(defrecord Station [loc])
(defrecord Reading [loc v])
(defrecord Hit     [loc])

(defrule or-hit
  [:or
   [Temp (= ?loc loc) (< c 20)]
   [Wind (= ?loc loc) (> kph 30)]]
  => (insert! (->Hit ?loc)))

(defrule prefix-then-or
  [Station (= ?loc loc)]
  [:or
   [Temp (= ?loc loc) (< c 20)]
   [Wind (= ?loc loc) (> kph 30)]]
  => (insert! (->Hit ?loc)))

(defrule or-then-fact
  [:or
   [Temp (= ?loc loc) (< c 20)]
   [Wind (= ?loc loc) (> kph 30)]]
  [Station (= ?loc loc)]
  => (insert! (->Hit ?loc)))

(defrule or-then-where
  [Reading (= ?loc loc) (= ?v v)]
  [:or
   [Temp (= ?loc loc)]
   [Wind (= ?loc loc)]]
  [:test (> ?v 10)]
  => (insert! (->Hit ?loc)))

(defquery hit-q [] [Hit (= ?loc loc)])

(defn n-locs [rule facts]
  (count (set (map :?loc (query (fire-rules (apply insert (mk-session [rule hit-q] :cache false) facts)) hit-q)))))

(defn -main [& _]
  (prn (str "row 1 or-cold-only n=" (n-locs or-hit [(->Temp 15 "MCI")])))
  (prn (str "row 2 or-wind-only n=" (n-locs or-hit [(->Wind 50 "MCI")])))
  (prn (str "row 3 or-both n=" (n-locs or-hit [(->Temp 15 "MCI") (->Wind 50 "MCI")])))
  (prn (str "row 4 prefix-or-cold n=" (n-locs prefix-then-or [(->Station "MCI") (->Temp 15 "MCI")])))
  (prn (str "row 5 prefix-or-wind n=" (n-locs prefix-then-or [(->Station "MCI") (->Wind 50 "MCI")])))
  (prn (str "row 6 prefix-or-both n=" (n-locs prefix-then-or [(->Station "MCI") (->Temp 15 "MCI") (->Wind 50 "MCI")])))
  (prn (str "row 7 prefix-or-no-station n=" (n-locs prefix-then-or [(->Temp 15 "MCI")])))
  (prn (str "row 8 or-fact-cold n=" (n-locs or-then-fact [(->Station "MCI") (->Temp 15 "MCI")])))
  (prn (str "row 9 or-fact-wind n=" (n-locs or-then-fact [(->Station "MCI") (->Wind 50 "MCI")])))
  (prn (str "row 10 or-fact-both n=" (n-locs or-then-fact [(->Station "MCI") (->Temp 15 "MCI") (->Wind 50 "MCI")])))
  (prn (str "row 11 or-fact-no-station n=" (n-locs or-then-fact [(->Temp 15 "MCI")])))
  (prn (str "row 12 or-where-pass n=" (n-locs or-then-where [(->Reading "MCI" 15) (->Temp 15 "MCI")])))
  (prn (str "row 13 or-where-fail n=" (n-locs or-then-where [(->Reading "MCI" 5) (->Temp 15 "MCI")])))
  (prn (str "row 14 or-where-both n=" (n-locs or-then-where [(->Reading "MCI" 15) (->Temp 15 "MCI") (->Wind 50 "MCI")]))))

(-main)
