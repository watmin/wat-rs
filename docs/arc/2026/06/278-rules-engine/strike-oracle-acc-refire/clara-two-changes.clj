(ns t2
  (:require [clara.rules :refer [defrule defquery mk-session insert fire-rules query insert!]]
            [clara.rules.accumulators :as acc]))
(defrecord Seed [y])
(defrecord Out [y])
(defrecord D [y])
(defrecord Tally [n])
(defrule a [Seed (= ?y y)] => (insert! (->Out ?y)))
(defrule b [Out (= ?y y)] [:test (= ?y 1)] => (insert! (->D 2)))
(defrule c [D (= ?y y)] => (insert! (->Out ?y)))
(defrule tally
  [Seed (= ?s y)]
  [?n <- (acc/count) :from [Out]]
  => (insert! (->Tally ?n)))
(defquery q [] [?f <- Tally])
(defn -main []
  (let [s (-> (mk-session 't2 :cache false) (insert (->Seed 1)) (fire-rules))
        t (query s q)]
    (println "clara Tally count =" (count t)
             " values =" (vec (sort (map #(:n (:?f %)) t))))))
