;; Twin of where-not-bound.wat. Clara 0.24.0. test-accum-result-in-negation.

(ns where-not-bound
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Station [loc])
(defrecord Reading [loc v])
(defrecord Busy    [loc n])

(defrule max-not-below
  [Station (= ?loc loc)]
  [?m <- (acc/max :v) :from [Reading (= ?loc loc)]]
  [:not [Reading (= ?loc loc) (< v ?m)]]
  => (insert! (->Busy ?loc ?m)))

(defquery busy-q [] [Busy (= ?loc loc) (= ?n n)])

(defn n-busy [lo hi]
  (count (query
           (fire-rules
             (insert (mk-session [max-not-below busy-q] :cache false)
                     (->Station "OSL")
                     (->Reading "OSL" lo)
                     (->Reading "OSL" hi)))
           busy-q)))

(defn -main [& _]
  (prn (str "row 1 max-not-below-mixed n=" (n-busy 50 40)))
  (prn (str "row 2 max-not-below-tied n=" (n-busy 50 50))))

(-main)
