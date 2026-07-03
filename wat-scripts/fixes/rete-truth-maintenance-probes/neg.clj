(ns neg (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord A [k]) (defrecord Bad [k]) (defrecord Ok [k])
(defrule mark-bad [A (= ?k k) (= ?k 2)] => (clara.rules/insert! (->Bad ?k)))
(defrule ok [A (= ?k k)] [:not [Bad (= ?k k)]] => (clara.rules/insert! (->Ok ?k)))
(defquery bad-q [] [Bad (= ?k k)])
(defquery ok-q  [] [Ok (= ?k k)])
(defn -main [& _]
  (let [s (-> (mk-session 'neg :cache false) (insert (->A 1) (->A 2)) fire-rules)]
    (println (str "Bad (Clara) = " (count (query s bad-q))))
    (println (str "Ok  (Clara) = " (count (query s ok-q))))))
