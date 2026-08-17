;; Twin of where-not-where.wat. Clara 0.24.0 test-test-in-negation.
;; n= is DISTINCT (a,b) pairs.

(ns where-not-where
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Hit  [a b])

(defrule not-same-loc
  [Temp (= ?a loc)]
  [Wind (= ?b loc)]
  [:not [:test (= ?a ?b)]]
  => (insert! (->Hit ?a ?b)))

(defquery hit-q [] [Hit (= ?a a) (= ?b b)])

(defn n-pairs [facts]
  (let [s (mk-session [not-same-loc hit-q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map (juxt :?a :?b) (query (fire-rules s) hit-q))))))

(defn -main [& _]
  (prn (str "row 1 same-loc n=" (n-pairs [(->Temp 10 "MCI") (->Wind 10 "MCI")])))
  (prn (str "row 2 diff-loc n=" (n-pairs [(->Temp 10 "MCI") (->Wind 10 "ORD")])))
  (prn (str "row 3 temp-only n=" (n-pairs [(->Temp 10 "MCI")])))
  (prn (str "row 4 two-diff n=" (n-pairs [(->Temp 10 "MCI") (->Temp 12 "ORD")
                                          (->Wind 10 "DFW") (->Wind 20 "SEA")]))))

(-main)
