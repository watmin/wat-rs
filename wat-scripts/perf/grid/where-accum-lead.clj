;; Twin of where-accum-lead.wat. Clara 0.24.0 test-count (leading acc).
;; Empty world: count = 0 and it fires. Three facts: count = 3.

(ns where-accum-lead
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Reading [v])
(defrecord Busy    [n])

(defrule count-zero
  [?n <- (acc/count) :from [Reading]]
  [:test (= ?n 0)]
  => (insert! (->Busy ?n)))

(defrule count-three
  [?n <- (acc/count) :from [Reading]]
  [:test (= ?n 3)]
  => (insert! (->Busy ?n)))

(defrule max-hi
  [?m <- (acc/max :v) :from [Reading]]
  [:test (> ?m 40)]
  => (insert! (->Busy ?m)))

(defquery busy-q [] [Busy (= ?n n)])

(defn n-busy [rule facts]
  (let [s (mk-session [rule busy-q] :cache false)
        s (if (seq facts) (apply insert s facts) s)]
    (count (set (map :?n (query (fire-rules s) busy-q))))))

(defn -main [& _]
  (prn (str "row 1 count0-empty n=" (n-busy count-zero [])))
  (prn (str "row 2 count0-three n=" (n-busy count-zero [(->Reading 1) (->Reading 2) (->Reading 3)])))
  (prn (str "row 3 count3-three n=" (n-busy count-three [(->Reading 1) (->Reading 2) (->Reading 3)])))
  (prn (str "row 4 count3-two n=" (n-busy count-three [(->Reading 1) (->Reading 2)])))
  (prn (str "row 5 max-empty n=" (n-busy max-hi [])))
  (prn (str "row 6 max-50 n=" (n-busy max-hi [(->Reading 50) (->Reading 40)]))))

(-main)
