#!/usr/bin/env bash
# gen-min-finding.sh STATIONS THRESHOLD — emit the Clara translation of grid/min-finding.wat (axis
# A7: minimum-finding-set, "≥N findings to activate") to stdout. SAME workload: Station(loc) for
# loc in [0,STATIONS); (loc mod (2*THRESHOLD)) Reading(loc) findings per station; a rule that
# activates Busy(loc,n) iff count(Reading for loc) >= THRESHOLD. Both engines gate the SAME boolean
# (count(matching-facts) >= N) over the SAME static fact snapshot before firing — a straight
# composition of A5's acc/count fold with a :test predicate (CLARA-TRANSLATIONS.md A7: "no caveat
# beyond A5"; grounded in dsl.clj:139-144 :test parse + the repo's own >= ?n 3 differential test).
#
# Times Clara fire-rules only (mk-session compile + JIT warmed out first via 3 dry fires). Emits one
# #grid/Result EDN line with :derived canonically encoded exactly like the wat side
# (loc * 1,000,000 + n, sorted ascending) so the two outputs compare byte-for-byte.
set -euo pipefail
S="$1"; T="$2"

cat <<HEADER
(ns min-finding
  (:require [clara.rules :refer [mk-session insert fire-rules query defrule defquery]]
            [clara.rules.accumulators :as acc]))
(defrecord Station [loc])
(defrecord Reading [loc])
(defrecord Busy [loc n])

(defrule flag
  [Station (= ?loc loc)]
  [?n <- (acc/count) :from [Reading (= ?loc loc)]]
  [:test (>= ?n $T)]
  => (clara.rules/insert! (->Busy ?loc ?n)))

(defquery busy-q [] [Busy (= ?loc loc) (= ?n n)])

(def span (* 2 $T))
(defn seed-facts []
  (mapcat (fn [i] (cons (->Station i) (repeat (mod i span) (->Reading i))))
          (range $S)))
(defn all-codes [s]
  (sort (map (fn [r] (+ (* (:?loc r) 1000000) (:?n r))) (query s busy-q))))
(defn -main [& _]
  (let [seeds (seed-facts)
        build (fn [] (apply insert (mk-session 'min-finding :cache false) seeds))]
    (dotimes [_ 3] (count (all-codes (fire-rules (build)))))   ; JIT + compile warmup
    (let [s (build) t0 (System/nanoTime) f (fire-rules s) t1 (System/nanoTime)
          codes (all-codes f)]
      (println (str "#grid/Result {:axis \"min-finding\" :size [$S $T] :derived ["
                     (clojure.string/join " " codes) "] :clara-ns " (- t1 t0) "}")))))
HEADER
