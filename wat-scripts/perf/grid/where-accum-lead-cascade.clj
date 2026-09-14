;; Twin of where-accum-lead-cascade.wat. A leading accumulator in a QUERY, under a cascade.
;;
;; Rows 1-3 differ ONLY in how many rounds the fixpoint must run: the S1 -> S2 -> S3 chain is
;; INERT — it derives facts the query never reads. Clara prints the same n= for all three, because
;; a query is answered against the world the fire ended in, not against each intermediate round.
;; That is the arbitration this file provides for RETE-FIX-LIST family A, where native leaked its
;; own round count into the answer: 1-rule chain -> 2 rows, 2-rule chain -> 3 rows, against 1 from
;; both references.
;;
;;   row 1  no-cascade   no chain, fixpoint settles immediately
;;   row 2  cascade-d1   one inert rule
;;   row 3  cascade-d2   two inert rules
;;   row 4  control-W    the accumulator's own facts are still there
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-accum-lead-cascade.wat
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-accum-lead-cascade.clj

(ns where-accum-lead-cascade
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord W  [k])
(defrecord S1 [k])
(defrecord S2 [k])
(defrecord S3 [k])

;; The inert chain. Neither rule reads or writes W.
(defrule r1 [S1 (= ?k k)] => (insert! (->S2 ?k)))
(defrule r2 [S2 (= ?k k)] => (insert! (->S3 ?k)))

(defquery q-lead []
  [?n <- (acc/count) :from [W]]
  [:test (>= ?n 2)])

(defquery q-W [] [W (= ?k k)])

(defn n [rules q]
  (count (query (fire-rules (insert (mk-session (conj rules q) :cache false)
                                    (->W 7) (->W 7) (->S1 1)))
                q)))

(defn -main [& _]
  (prn (str "row 1 no-cascade n=" (n []       q-lead)))
  (prn (str "row 2 cascade-d1 n=" (n [r1]     q-lead)))
  (prn (str "row 3 cascade-d2 n=" (n [r1 r2]  q-lead)))
  (prn (str "row 4 control-W n="  (n [r1 r2]  q-W))))

(-main)
