(ns chain (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord A [k]) (defrecord B [k]) (defrecord C [k])

;; R1: A -> B (single input match)
(defrule r1 [A (= ?k k)] => (clara.rules/insert! (->B ?k)))
;; R2: B JOIN A (derived B joined with input A, same k) -> C
(defrule r2 [B (= ?k k)] [A (= ?k k)] => (clara.rules/insert! (->C ?k)))

(defquery b-q [] [B (= ?k k)])
(defquery c-q [] [C (= ?k k)])

(defn -main [& _]
  (let [s (-> (mk-session 'chain :cache false) (insert (->A 1) (->A 2)) fire-rules)]
    (println (str "B (derived)  = " (count (query s b-q))))
    (println (str "C (B join A) = " (count (query s c-q))))))
