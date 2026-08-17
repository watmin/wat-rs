;; Twin of where-accum-where.wat. Clara :test on an accumulator result.

(ns where-accum-where
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Station [loc])
(defrecord Reading [loc v])
(defrecord Busy    [loc n])

(defrule count-eq-3
  [Station (= ?loc loc)]
  [?n <- (acc/count) :from [Reading (= ?loc loc)]]
  [:test (= ?n 3)]
  => (insert! (->Busy ?loc ?n)))

(defrule max-gt-40
  [Station (= ?loc loc)]
  [?m <- (acc/max :v) :from [Reading (= ?loc loc)]]
  [:test (> ?m 40)]
  => (insert! (->Busy ?loc ?m)))

(defquery busy-q [] [Busy (= ?loc loc) (= ?n n)])

(defn n-busy [rule facts]
  (count (set (map :?loc
                   (query (fire-rules (apply insert (mk-session [rule busy-q] :cache false) facts))
                          busy-q)))))

(defn -main [& _]
  (prn (str "row 1 count-eq-3 n="
            (n-busy count-eq-3 [(->Station "OSL")
                                (->Reading "OSL" 1) (->Reading "OSL" 2) (->Reading "OSL" 3)])))
  (prn (str "row 2 count-eq-3-miss n="
            (n-busy count-eq-3 [(->Station "OSL")
                                (->Reading "OSL" 1) (->Reading "OSL" 2)])))
  (prn (str "row 3 max-gt-40 n="
            (n-busy max-gt-40 [(->Station "OSL")
                               (->Reading "OSL" 50) (->Reading "OSL" 40)])))
  (prn (str "row 4 max-le-40 n="
            (n-busy max-gt-40 [(->Station "OSL")
                               (->Reading "OSL" 40) (->Reading "OSL" 30)])))
  (prn (str "row 5 count-two-stations n="
            (n-busy count-eq-3 [(->Station "OSL") (->Station "BGO")
                                (->Reading "OSL" 1) (->Reading "OSL" 2) (->Reading "OSL" 3)
                                (->Reading "BGO" 1) (->Reading "BGO" 2) (->Reading "BGO" 3)]))))

(-main)
