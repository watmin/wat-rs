;; Twin of where-not-fact.wat. Clara 0.24.0 test-simple-negation.

(ns where-not-fact
  (:require [clara.rules :refer [mk-session insert retract fire-rules query defrule defquery insert!]]))

(defrecord Temp [c loc])
(defrecord Hit  [k])

(defrule not-cold
  [:not [Temp (< c 20)]]
  => (insert! (->Hit 1)))

(defquery hit-q [] [Hit (= ?k k)])

(defn n-hit [facts]
  (let [s (mk-session [not-cold hit-q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map :?k (query (fire-rules s) hit-q))))))

(defn n-retract []
  (let [t (->Temp 10 "MCI")
        s (-> (mk-session [not-cold hit-q] :cache false)
              (insert t)
              (retract t))]
    (count (set (map :?k (query (fire-rules s) hit-q))))))

(defn n-partial []
  (let [a (->Temp 10 "MCI")
        b (->Temp 15 "MCI")
        s (-> (mk-session [not-cold hit-q] :cache false)
              (insert a b)
              (retract a))]
    (count (set (map :?k (query (fire-rules s) hit-q))))))

(defn -main [& _]
  (prn (str "row 1 empty n=" (n-hit [])))
  (prn (str "row 2 cold n=" (n-hit [(->Temp 10 "MCI")])))
  (prn (str "row 3 hot n=" (n-hit [(->Temp 80 "MCI")])))
  (prn (str "row 4 retract-cold n=" (n-retract)))
  (prn (str "row 5 partial-retract n=" (n-partial))))

(-main)
