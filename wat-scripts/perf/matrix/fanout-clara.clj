;; matrix dim: fan-out/selectivity — Clara side. Run: clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}} :paths ["<dir>"]}' -M -m fanbench K F
(ns fanbench (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]))
(defrecord Left [key lid]) (defrecord Right [key rid]) (defrecord Pair [key lid rid])
(defrule r-fan [Left (= ?k key) (= ?l lid)] [Right (= ?k key) (= ?r rid)]
  => (clara.rules/insert! (->Pair ?k ?l ?r)))
(defquery pairs-q [] [Pair (= ?k key)])
(defn -main [& args]
  (let [[K F] (map #(Integer/parseInt %) args)
        seeds (mapcat (fn [k] (mapcat (fn [f] [(->Left k f) (->Right k f)]) (range F))) (range K))
        build (fn [] (apply insert (mk-session 'fanbench :cache false) seeds))]
    (dotimes [_ 3] (count (query (fire-rules (build)) pairs-q)))
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)]
      (println (str "#clara/Fan {:keys " K " :fanout " F " :pairs " (count (query f pairs-q)) " :clara-ns " (- t1 t0) "}")))))
