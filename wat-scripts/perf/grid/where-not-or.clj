;; Twin of where-not-or.wat. Clara 0.24.0 test-negated-disjunction.
;; n= is DISTINCT keys / locations.

(ns where-not-or
  (:require [clara.rules :refer [mk-session insert retract fire-rules query defrule defquery insert!]]))

(defrecord Temp    [c loc])
(defrecord Wind    [kph loc])
(defrecord Station [loc])
(defrecord Hit     [k])
(defrecord At      [loc])

(defrule not-cold-or-windy
  [:not [:or
         [Wind (> kph 30)]
         [Temp (< c 20)]]]
  => (insert! (->Hit 1)))

(defrule station-not-either
  [Station (= ?loc loc)]
  [:not [:or
         [Wind (= ?loc loc) (> kph 30)]
         [Temp (= ?loc loc) (< c 20)]]]
  => (insert! (->At ?loc)))

(defquery hit-q [] [Hit (= ?k k)])
(defquery at-q  [] [At (= ?loc loc)])

(defn run [rule q n-fn facts]
  (let [s (mk-session [rule q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map n-fn (query (fire-rules s) q))))))

(defn n-hit [facts] (run not-cold-or-windy hit-q :?k facts))
(defn n-at  [facts] (run station-not-either at-q :?loc facts))

(defn n-hit-retract []
  (let [w (->Wind 40 "MCI")
        s (-> (mk-session [not-cold-or-windy hit-q] :cache false)
              (insert w)
              (retract w))]
    (count (set (map :?k (query (fire-rules s) hit-q))))))

(defn -main [& _]
  (prn (str "row 1 empty n=" (n-hit [])))
  (prn (str "row 2 wind n=" (n-hit [(->Wind 40 "MCI")])))
  (prn (str "row 3 temp n=" (n-hit [(->Temp 10 "MCI")])))
  (prn (str "row 4 both n=" (n-hit [(->Wind 40 "MCI") (->Temp 10 "MCI")])))
  (prn (str "row 5 retract-wind n=" (n-hit-retract)))
  (prn (str "row 6 prefix-empty n=" (n-at [(->Station "MCI")])))
  (prn (str "row 7 prefix-wind n=" (n-at [(->Station "MCI") (->Wind 40 "MCI")])))
  (prn (str "row 8 prefix-temp n=" (n-at [(->Station "MCI") (->Temp 10 "MCI")]))))

(-main)
