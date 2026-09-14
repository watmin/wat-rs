;; Twin of where-accum-where-chain.wat. TWO `:test`s after an accumulator.
;;
;; Rows 1 and 2 differ by exactly ONE trailing, trivially-true `:test`. Clara 0.24.0 prints the
;; same n= for both — a `:test` after a `:test` after an accumulator is an ordinary chain, and a
;; tautology cannot subtract a match. That is the arbitration this file exists to provide:
;; wat's `$oracle` says the two rows agree, and Clara is the third reference that says so too.
;;
;; Run both and compare by eye (the byte-diff instrument is where-shapes.{wat,clj}):
;;
;;     ./target/release/wat  wat-scripts/perf/grid/where-accum-where-chain.wat
;;     clojure -Sdeps '{:deps {com.cerner/clara-rules {:mvn/version "0.24.0"}}}' \
;;             -M wat-scripts/perf/grid/where-accum-where-chain.clj

(ns where-accum-where-chain
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery insert!]]
            [clara.rules.accumulators :as acc]))

(defrecord Station [loc])
(defrecord Reading [loc v])
(defrecord Busy    [loc n])

;; ROW 1 — the agreeing control: station, accumulator, ONE test.
(defrule one-where
  [Station (= ?loc loc)]
  [?n <- (acc/count) :from [Reading (= ?loc loc)]]
  [:test (>= ?n 2)]
  => (insert! (->Busy ?loc ?n)))

;; ROW 2 — the same rule plus one trailing tautology.
(defrule two-wheres
  [Station (= ?loc loc)]
  [?n <- (acc/count) :from [Reading (= ?loc loc)]]
  [:test (>= ?n 2)]
  [:test (> 1 0)]
  => (insert! (->Busy ?loc ?n)))

(defquery busy-q [] [Busy (= ?loc loc) (= ?n n)])

;; SUM of Busy.n, matching the wat side's `sum-n` — so the two outputs are line-comparable.
(defn sum-n [rule facts]
  (reduce + 0 (map :?n
                   (query (fire-rules (apply insert (mk-session [rule busy-q] :cache false) facts))
                          busy-q))))

(def mci [(->Station "MCI")
          (->Reading "MCI" 1) (->Reading "MCI" 2) (->Reading "MCI" 3)])

(defn -main [& _]
  (prn (str "row 1 one-where n=" (sum-n one-where mci)))
  (prn (str "row 2 two-wheres n=" (sum-n two-wheres mci))))

(-main)
