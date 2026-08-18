;; Twin of where-accum-from-left.wat. Clara 0.24.0.
;; Leftover ?c on accumulate :from: count Winds with kph > the Temp at that loc.

(ns where-accum-from-left
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Hit  [loc n])

(defrule count-winds-above-temp
  [Temp (= ?loc loc) (= ?c c)]
  [?n <- (acc/count) :from [Wind (= ?loc loc) (> kph ?c)]]
  => (insert! (->Hit ?loc ?n)))

(defquery hit-q [] [Hit (= ?loc loc) (= ?n n)])

(defn sum-n [facts]
  (let [s (mk-session [count-winds-above-temp hit-q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (reduce + 0 (map :?n (query (fire-rules s) hit-q)))))

(defn -main [& _]
  (prn (str "row 1 empty n=" (sum-n [])))
  (prn (str "row 2 temp-only n=" (sum-n [(->Temp 10 "MCI")])))
  (prn (str "row 3 below n=" (sum-n [(->Temp 10 "MCI") (->Wind 5 "MCI")])))
  (prn (str "row 4 above n=" (sum-n [(->Temp 10 "MCI") (->Wind 20 "MCI")])))
  (prn (str "row 5 equal n=" (sum-n [(->Temp 10 "MCI") (->Wind 10 "MCI")])))
  (prn (str "row 6 two-of-three n=" (sum-n [(->Temp 10 "MCI")
                                           (->Wind 5 "MCI")
                                           (->Wind 20 "MCI")
                                           (->Wind 30 "MCI")])))
  (prn (str "row 7 two-locs n=" (sum-n [(->Temp 10 "MCI") (->Wind 20 "MCI")
                                       (->Temp 10 "ORD") (->Wind 5 "ORD")]))))

(-main)
