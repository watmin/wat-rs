;; Twin of where-fact-bind.wat. Clara 0.24.0 `[?t <- Temp]`.

(ns where-fact-bind
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Temp [c loc])
(defrecord Hit [c])

(defrule cool
  [?t <- Temp]
  [:test (< (:c ?t) 20)]
  => (insert! (->Hit (:c ?t))))

(defquery q-bound [] [?t <- Temp])
(defquery q-plain [] [Temp])
(defquery q-both [] [?t <- Temp (= ?c c)])
(defquery q-hit [] [Hit (= ?c c)])
(defquery q-from []
  [?n <- (acc/count) :from [Temp (= ?loc loc)]])

(defn has-key [answers k]
  (if (empty? answers)
    "empty"
    (if (contains? (first answers) k) "yes" "none")))

(defn -main [& _]
  (let [s (-> (mk-session [cool q-bound q-plain q-both q-hit] :cache false)
              (insert (->Temp 15 "MCI") (->Temp 80 "MCI"))
              fire-rules)
        bound (query s q-bound)
        plain (query s q-plain)
        both  (query s q-both)
        hits  (query s q-hit)]
    (prn (str "row 1 bound n=" (count bound) " has=?t " (has-key bound :?t)))
    (prn (str "row 2 plain n=" (count plain) " has=?t " (has-key plain :?t)))
    (prn (str "row 3 both n=" (count both)
              " has=?t " (has-key both :?t)
              " has=?c " (has-key both :?c)))
    (prn (str "row 4 cool n=" (count hits) " -> " (:?c (first hits))))
    (let [only (-> (mk-session [q-from] :cache false)
                   (insert (->Temp 15 "MCI") (->Temp 80 "MCI"))
                   fire-rules)]
      (prn (str "row 5 from n=" (count (query only q-from)))))))

(-main)
