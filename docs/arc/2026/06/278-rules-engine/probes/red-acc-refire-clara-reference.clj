(ns d2
  (:require [clara.rules :refer [defrule defquery mk-session insert fire-rules query insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord A [x])
(defrecord B [x y])
(defrecord Seed [y])
(defrecord C [y])
(defrecord Out [x y])
(defrecord Tally [n])

;; round 1 derives C — so the accumulate's source appears mid-session
(defrule mk-c
  [Seed (= ?y y)]
  => (insert! (->C ?y)))

;; Out derives from C (derived), so the accumulate's count goes 0 -> 1 across the fixpoint
(defrule chain
  [C (= ?y y)]
  => (insert! (->Out 1 ?y)))

;; the accumulate, UNFENCED — exactly the wat fixture's shape
(defrule tally
  [Seed (= ?y y)]
  [?n <- (acc/count) :from [Out]]
  => (insert! (->Tally ?n)))

(defquery q-out   [] [?f <- Out])
(defquery q-tally [] [?f <- Tally])

(defn -main []
  (let [s (-> (mk-session 'd2 :cache false)
              (insert (->A 1) (->B 1 7) (->Seed 7))
              (fire-rules))
        outs   (query s q-out)
        tals   (query s q-tally)]
    (println "clara [Out Tally] =" [(count outs) (count tals)])
    (println "clara Tally values =" (vec (sort (map #(:n (:?f %)) tals))))))
