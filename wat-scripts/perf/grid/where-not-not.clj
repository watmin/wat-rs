;; Twin of where-not-not.wat. Clara 0.24.0 double negation.
;; n= is DISTINCT locs / keys.

(ns where-not-not
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Hit  [loc])
(defrecord Yes  [k])

(defrule wind-not-not-temp
  [Wind (= ?loc loc)]
  [:not [:not [Temp (= ?loc loc)]]]
  => (insert! (->Hit ?loc)))

(defrule lead-not-not
  [:not [:not [Temp]]]
  => (insert! (->Yes 1)))

(defquery hit-q [] [Hit (= ?loc loc)])
(defquery yes-q [] [Yes (= ?k k)])

(defn run [rule q n-fn facts]
  (let [s (mk-session [rule q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map n-fn (query (fire-rules s) q))))))

(defn -main [& _]
  (prn (str "row 1 wind-only n=" (run wind-not-not-temp hit-q :?loc [(->Wind 40 "MCI")])))
  (prn (str "row 2 same-loc n=" (run wind-not-not-temp hit-q :?loc [(->Wind 40 "MCI") (->Temp 10 "MCI")])))
  (prn (str "row 3 diff-loc n=" (run wind-not-not-temp hit-q :?loc [(->Wind 40 "MCI") (->Temp 10 "ORD")])))
  (prn (str "row 4 temp-only n=" (run wind-not-not-temp hit-q :?loc [(->Temp 10 "MCI")])))
  (prn (str "row 5 lead-empty n=" (run lead-not-not yes-q :?k [])))
  (prn (str "row 6 lead-temp n=" (run lead-not-not yes-q :?k [(->Temp 10 "MCI")]))))

(-main)
