;; Twin of where-not-and.wat. Clara 0.24.0 test-negated-conjunction.
;; n= is DISTINCT keys / locations.

(ns where-not-and
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Temp    [c loc])
(defrecord Wind    [kph loc])
(defrecord Station [loc])
(defrecord Hit     [k])
(defrecord At      [loc])

(defrule not-cold-and-windy
  [:not [:and
         [Wind (> kph 30)]
         [Temp (< c 20)]]]
  => (insert! (->Hit 1)))

(defrule station-not-both
  [Station (= ?loc loc)]
  [:not [:and
         [Wind (= ?loc loc) (> kph 30)]
         [Temp (= ?loc loc) (< c 20)]]]
  => (insert! (->At ?loc)))

(defquery hit-q [] [Hit (= ?k k)])
(defquery at-q  [] [At (= ?loc loc)])

(defn run [rule q n-fn facts]
  (let [s (mk-session [rule q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map n-fn (query (fire-rules s) q))))))

(defn n-hit [facts] (run not-cold-and-windy hit-q :?k facts))
(defn n-at  [facts] (run station-not-both at-q :?loc facts))

(defn -main [& _]
  (prn (str "row 1 empty n=" (n-hit [])))
  (prn (str "row 2 wind-only n=" (n-hit [(->Wind 40 "MCI")])))
  (prn (str "row 3 temp-only n=" (n-hit [(->Temp 10 "MCI")])))
  (prn (str "row 4 both n=" (n-hit [(->Wind 40 "MCI") (->Temp 10 "MCI")])))
  (prn (str "row 5 prefix-empty n=" (n-at [(->Station "MCI")])))
  (prn (str "row 6 prefix-wind n=" (n-at [(->Station "MCI") (->Wind 40 "MCI")])))
  (prn (str "row 7 prefix-temp n=" (n-at [(->Station "MCI") (->Temp 10 "MCI")])))
  (prn (str "row 8 prefix-both n=" (n-at [(->Station "MCI") (->Wind 40 "MCI") (->Temp 10 "MCI")]))))

(-main)
