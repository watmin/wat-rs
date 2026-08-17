;; Twin of where-query-compat.wat. Clara 0.24.0 query mouth.
;; Binding maps, sorted scalars. A bound record is presence, not EDN.

(ns where-query-compat
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Temp [c loc])
(defrecord Wind [kph loc])
(defrecord Hit [loc])

(defrule mark
  [Wind (= ?loc loc) (> kph 10)]
  => (insert! (->Hit ?loc)))

(defquery q-fields [] [Temp (= ?c c) (= ?loc loc)])
(defquery q-plain [] [Temp])
(defquery q-bound [] [?t <- Temp (= ?c c)])
(defquery q-at [:?loc] [Temp (= ?c c) (= ?loc loc)])
(defquery q-join []
  [Temp (= ?c c) (= ?loc loc)]
  [Wind (= ?w kph) (= ?loc loc)])
(defquery q-count-at [:?loc]
  [?n <- (acc/count) :from [Temp (= ?loc loc)]])
(defquery q-count-wind [:?loc]
  [?n <- (acc/count) :from [Temp (= ?loc loc)]]
  [Wind (= ?loc loc)])
(defquery q-no-wind []
  [Temp (= ?c c) (= ?loc loc)]
  [:not [Wind (= loc ?loc)]])
(defquery q-has-wind []
  [Temp (= ?c c) (= ?loc loc)]
  [:exists [Wind (= loc ?loc)]])
(defquery q-cool []
  [Temp (= ?c c) (= ?loc loc)]
  [:test (< ?c 20)])
(defquery q-hit [] [Hit (= ?loc loc)])

(def facts
  [(->Temp 15 "MCI") (->Temp 80 "MCI") (->Temp 40 "SFO") (->Temp 10 "ORD")
   (->Wind 20 "MCI") (->Wind 5 "SFO") (->Wind 20 "LAX")])

(defn insert-all [s fs]
  (apply insert s fs))

(defn world []
  (-> (mk-session [q-fields q-plain q-bound q-at q-join
                   q-count-at q-count-wind q-no-wind q-has-wind q-cool]
                  :cache false)
      (insert-all facts)
      fire-rules))

(defn world-hits []
  (-> (mk-session [mark q-hit] :cache false)
      (insert-all facts)
      fire-rules))

(defn world-empty []
  (fire-rules
    (mk-session [q-fields q-plain q-bound q-at q-join
                 q-count-at q-count-wind q-no-wind q-has-wind q-cool]
                :cache false)))

(defn has-key [answers k]
  (if (empty? answers)
    "empty"
    (if (contains? (first answers) k) "yes" "none")))

(defn pairs-c-loc [answers]
  (->> answers
       (map #(str (:?c %) "," (:?loc %)))
       sort
       (map #(str " " %))
       (apply str)))

(defn pairs-join [answers]
  (->> answers
       (map #(str (:?c %) "," (:?w %) "," (:?loc %)))
       sort
       (map #(str " " %))
       (apply str)))

(defn vals-c [answers]
  (->> answers (map :?c) sort (map #(str " " %)) (apply str)))

(defn vals-loc [answers]
  (->> answers (map :?loc) sort (map #(str " " %)) (apply str)))

(defn one-n [answers]
  (if (empty? answers) "" (str " " (:?n (first answers)))))

(defn -main [& _]
  (let [s (world)
        fields (query s q-fields)
        plain  (query s q-plain)
        bound  (query s q-bound)
        at-mci (query s q-at :?loc "MCI")
        at-xxx (query s q-at :?loc "XXX")
        join   (query s q-join)
        n-mci  (query s q-count-at :?loc "MCI")
        n-lax  (query s q-count-wind :?loc "LAX")
        none   (query s q-no-wind)
        some   (query s q-has-wind)
        cool   (query s q-cool)
        hits   (query (world-hits) q-hit)
        empty  (query (world-empty) q-fields)]
    (prn (str "row 1 fields n=" (count fields) " ->" (pairs-c-loc fields)))
    (prn (str "row 2 plain n=" (count plain) " has=?c " (has-key plain :?c)))
    (prn (str "row 3 fact-bind n=" (count bound)
              " has=?t " (has-key bound :?t) " ->" (vals-c bound)))
    (prn (str "row 4 params-mci n=" (count at-mci) " ->" (pairs-c-loc at-mci)))
    (prn (str "row 5 params-xxx n=" (count at-xxx)))
    (prn (str "row 6 join n=" (count join) " ->" (pairs-join join)))
    (prn (str "row 7 count-mci n=" (count n-mci) " ->" (one-n n-mci)))
    (prn (str "row 8 count-zero n=" (count n-lax) " ->" (one-n n-lax)))
    (prn (str "row 9 not-wind n=" (count none) " ->" (pairs-c-loc none)))
    (prn (str "row 10 exists-wind n=" (count some) " ->" (pairs-c-loc some)))
    (prn (str "row 11 where-cool n=" (count cool) " ->" (pairs-c-loc cool)))
    (prn (str "row 12 derived n=" (count hits) " ->" (vals-loc hits)))
    (prn (str "row 13 empty n=" (count empty)))))

(-main)
