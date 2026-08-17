;; Twin of where-not-and-bound.wat. Clara 0.24.0 test-complex-negation.
;; n= is 1 when the :not fires (no matching Temp/Cold join), 0 when it does not.

(ns where-not-and-bound
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Wind [kph loc])
(defrecord Temp [c loc])
(defrecord Cold [c])
(defrecord Hit  [k])
(defrecord At   [loc])

(defrule not-match-temp
  [:not [:and
         [?t <- Temp]
         [Cold (= c (:c ?t))]]]
  => (insert! (->Hit 1)))

(defrule prior-not-match
  [Wind (= ?l loc)]
  [:not [:and
         [?t <- Temp (= ?l loc)]
         [Cold (= c (:c ?t))]]]
  => (insert! (->At ?l)))

(defquery hit-q [] [Hit (= ?k k)])
(defquery at-q  [] [At (= ?loc loc)])

(defn run [rule q n-fn facts]
  (let [s (mk-session [rule q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map n-fn (query (fire-rules s) q))))))

(defn n-hit [facts] (run not-match-temp hit-q :?k facts))
(defn n-at  [facts] (run prior-not-match at-q :?loc facts))

(defn -main [& _]
  (prn (str "row 1 empty n=" (n-hit [])))
  (prn (str "row 2 temp-only n=" (n-hit [(->Temp 10 "MCI")])))
  (prn (str "row 3 mismatch n=" (n-hit [(->Temp 10 "MCI") (->Cold 20)])))
  (prn (str "row 4 match n=" (n-hit [(->Temp 10 "MCI") (->Cold 10)])))
  (prn (str "row 5 prior-empty n=" (n-at [])))
  (prn (str "row 6 prior-wind n=" (n-at [(->Wind 10 "MCI")])))
  (prn (str "row 7 prior-other-loc n=" (n-at [(->Wind 10 "MCI") (->Temp 10 "ORD") (->Cold 10)])))
  (prn (str "row 8 prior-same-loc n=" (n-at [(->Wind 10 "MCI") (->Temp 10 "MCI") (->Cold 10)]))))

(-main)
