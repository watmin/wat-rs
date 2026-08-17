;; Twin of where-test-chain.wat. Clara 0.24.0. test-simple-test spoken vs joins-first.

(ns where-test-chain
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]))

(defrecord Temp [c loc])
(defrecord Pair [a b])

(defrule spoken
  [Temp (= ?t1 c) (= ?loc loc)]
  [:test (< ?t1 20)]
  [Temp (= ?t2 c) (= ?loc loc)]
  [:test (< ?t2 20)]
  [:test (< ?t1 ?t2)]
  => (insert! (->Pair ?t1 ?t2)))

(defrule join-first
  [Temp (= ?t1 c) (= ?loc loc)]
  [Temp (= ?t2 c) (= ?loc loc)]
  [:test (< ?t1 20)]
  [:test (< ?t2 20)]
  [:test (< ?t1 ?t2)]
  => (insert! (->Pair ?t1 ?t2)))

(defquery pair-q [] [Pair (= ?a a) (= ?b b)])

(def rows
  [[1 "spoken"     spoken]
   [2 "join-first" join-first]])

(def seeds [(->Temp 15 "MCI") (->Temp 10 "MCI") (->Temp 80 "MCI")])

(defn run-row [[row nm rule]]
  (let [session (apply insert (mk-session [rule pair-q] :cache false) seeds)
        pairs   (sort (map (fn [r] [(:?a r) (:?b r)]) (query (fire-rules session) pair-q)))]
    (str "row " row " " nm " n=" (count pairs) " ->"
         (apply str (map (fn [[a b]] (str " " a "," b)) pairs)))))

(defn -main [& _] (doseq [r rows] (prn (run-row r))))

(-main)
